use crate::blueprints::util::SecurifiedAccessRules;
use crate::errors::*;
use crate::kernel::kernel_api::{KernelApi, KernelSubstateApi};
use crate::system::node_init::type_info_partition;
use crate::system::node_modules::metadata::MetadataEntrySubstate;
use crate::system::node_modules::type_info::TypeInfoSubstate;
use crate::system::system_modules::costing::{apply_royalty_cost, RoyaltyRecipient};
use crate::track::interface::NodeSubstates;
use crate::types::*;
use crate::vm::wasm::PrepareError;
use native_sdk::modules::access_rules::AccessRules;
use native_sdk::modules::metadata::Metadata;
use native_sdk::modules::royalty::ComponentRoyalty;
use native_sdk::resource::NativeVault;
use native_sdk::resource::ResourceManager;
use radix_engine_interface::api::node_modules::metadata::MetadataInit;
use radix_engine_interface::api::{
    ClientApi, ClientObjectApi, KVEntry, LockFlags, ObjectModuleId, OBJECT_HANDLE_SELF,
};
pub use radix_engine_interface::blueprints::package::*;
use radix_engine_interface::blueprints::resource::{require, Bucket};
use radix_engine_interface::schema::{
    BlueprintCollectionSchema, BlueprintEventSchemaInit, BlueprintFunctionsSchemaInit,
    BlueprintKeyValueStoreSchema, BlueprintSchemaInit, BlueprintStateSchemaInit, FieldSchema,
    FunctionSchemaInit, TypeRef,
};
use sbor::LocalTypeIndex;

// Import and re-export substate types
use crate::roles_template;
use crate::system::node_modules::access_rules::AccessRulesNativePackage;
use crate::system::node_modules::royalty::RoyaltyUtil;
use crate::system::system::{KeyValueEntrySubstate, SubstateMutability, SystemService};
use crate::system::system_callback::{SystemConfig, SystemLockData};
use crate::system::system_callback_api::SystemCallbackObject;
use crate::system::system_modules::auth::{AuthError, ResolvedPermission};
use crate::vm::VmPackageValidation;
pub use radix_engine_interface::blueprints::package::{
    PackageInstrumentedCodeSubstate, PackageOriginalCodeSubstate, PackageRoyaltyAccumulatorSubstate,
};

pub const PACKAGE_ROYALTY_FEATURE: &str = "package-royalty";

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum PackageError {
    InvalidWasm(PrepareError),

    InvalidBlueprintWasm(SchemaValidationError),
    TooManySubstateSchemas,

    FailedToResolveLocalSchema {
        local_type_index: LocalTypeIndex,
    },
    EventNameMismatch {
        expected: String,
        actual: Option<String>,
    },
    InvalidEventSchema,
    InvalidSystemFunction,
    InvalidTypeParent,
    MissingOuterBlueprint,
    WasmUnsupported(String),
    InvalidGenericId(u8),

    InvalidAuthSetup,
    DefiningReservedRoleKey(String, RoleKey),
    MissingRole(RoleKey),
    UnexpectedNumberOfMethodAuth {
        blueprint: String,
        expected: usize,
        actual: usize,
    },
    MissingMethodPermission {
        blueprint: String,
        ident: String,
    },

    UnexpectedNumberOfFunctionAuth {
        blueprint: String,
        expected: usize,
        actual: usize,
    },
    MissingFunctionPermission {
        blueprint: String,
        ident: String,
    },

    UnexpectedNumberOfFunctionRoyalties {
        blueprint: String,
        expected: usize,
        actual: usize,
    },
    MissingFunctionRoyalty {
        blueprint: String,
        ident: String,
    },
    RoyaltyAmountIsGreaterThanAllowed {
        max: RoyaltyAmount,
        actual: RoyaltyAmount,
    },

    InvalidMetadataKey(String),

    RoyaltiesNotEnabled,
}

fn validate_package_schema<'a, I: Iterator<Item = &'a BlueprintSchemaInit>>(
    blueprints: I,
) -> Result<(), PackageError> {
    for bp_init in blueprints {
        validate_schema(&bp_init.schema).map_err(|e| PackageError::InvalidBlueprintWasm(e))?;

        if bp_init.state.fields.len() > 0xff {
            return Err(PackageError::TooManySubstateSchemas);
        }

        // FIXME: Add validation for valid local_type_index of all schema'd things

        let num_generics = bp_init.generics.len() as u8;

        for field in &bp_init.state.fields {
            match field.field {
                TypeRef::Static(..) => {}
                TypeRef::Generic(generic_id) => {
                    if generic_id >= num_generics {
                        return Err(PackageError::InvalidGenericId(generic_id));
                    }
                }
            }
        }

        for collection in &bp_init.state.collections {
            match collection {
                BlueprintCollectionSchema::KeyValueStore(kv_store_schema) => {
                    match kv_store_schema.key {
                        TypeRef::Static(..) => {}
                        TypeRef::Generic(generic_id) => {
                            if generic_id >= num_generics {
                                return Err(PackageError::InvalidGenericId(generic_id));
                            }
                        }
                    }

                    match kv_store_schema.value {
                        TypeRef::Static(..) => {}
                        TypeRef::Generic(generic_id) => {
                            if generic_id >= num_generics {
                                return Err(PackageError::InvalidGenericId(generic_id));
                            }
                        }
                    }
                }
                BlueprintCollectionSchema::SortedIndex(..) => {}
                BlueprintCollectionSchema::Index(..) => {}
            }
        }

        for (_name, event) in &bp_init.events.event_schema {
            match event {
                TypeRef::Static(..) => {}
                TypeRef::Generic(generic_id) => {
                    if *generic_id >= num_generics {
                        return Err(PackageError::InvalidGenericId(*generic_id));
                    }
                }
            }
        }

        for (_name, function) in &bp_init.functions.functions {
            match function.input {
                TypeRef::Static(..) => {}
                TypeRef::Generic(generic_id) => {
                    if generic_id >= num_generics {
                        return Err(PackageError::InvalidGenericId(generic_id));
                    }
                }
            }
            match function.output {
                TypeRef::Static(..) => {}
                TypeRef::Generic(generic_id) => {
                    if generic_id >= num_generics {
                        return Err(PackageError::InvalidGenericId(generic_id));
                    }
                }
            }
        }
    }

    Ok(())
}

fn validate_package_event_schema<'a, I: Iterator<Item = &'a BlueprintDefinitionInit>>(
    blueprints: I,
) -> Result<(), PackageError> {
    for BlueprintDefinitionInit {
        schema: BlueprintSchemaInit { schema, events, .. },
        ..
    } in blueprints
    {
        // Package schema validation happens when the package is published. No need to redo
        // it here again.

        for (expected_event_name, local_type_index) in events.event_schema.iter() {
            let local_type_index = match local_type_index {
                TypeRef::Static(type_index) => type_index,
                TypeRef::Generic(..) => {
                    return Err(PackageError::WasmUnsupported(
                        "Generics not supported".to_string(),
                    ));
                }
            };

            // Checking that the event is either a struct or an enum
            let type_kind = schema.resolve_type_kind(*local_type_index).map_or(
                Err(PackageError::FailedToResolveLocalSchema {
                    local_type_index: *local_type_index,
                }),
                Ok,
            )?;
            match type_kind {
                // Structs and Enums are allowed
                TypeKind::Enum { .. } | TypeKind::Tuple { .. } => Ok(()),
                _ => Err(PackageError::InvalidEventSchema),
            }?;

            // Checking that the event name is indeed what the user claims it to be
            let actual_event_name = schema.resolve_type_metadata(*local_type_index).map_or(
                Err(PackageError::FailedToResolveLocalSchema {
                    local_type_index: *local_type_index,
                }),
                |metadata| Ok(metadata.get_name_string()),
            )?;

            if Some(expected_event_name) != actual_event_name.as_ref() {
                Err(PackageError::EventNameMismatch {
                    expected: expected_event_name.to_string(),
                    actual: actual_event_name,
                })?
            }
        }
    }

    Ok(())
}

fn validate_royalties<Y>(definition: &PackageDefinition, api: &mut Y) -> Result<(), RuntimeError>
where
    Y: ClientApi<RuntimeError>,
{
    for (blueprint, definition_init) in &definition.blueprints {
        match &definition_init.royalty_config {
            PackageRoyaltyConfig::Disabled => {}
            PackageRoyaltyConfig::Enabled(function_royalties) => {
                let num_functions = definition_init.schema.functions.functions.len();

                if num_functions != function_royalties.len() {
                    return Err(RuntimeError::ApplicationError(
                        ApplicationError::PackageError(
                            PackageError::UnexpectedNumberOfFunctionRoyalties {
                                blueprint: blueprint.clone(),
                                expected: num_functions,
                                actual: function_royalties.len(),
                            },
                        ),
                    ));
                }

                for name in definition_init.schema.functions.functions.keys() {
                    if !function_royalties.contains_key(name) {
                        return Err(RuntimeError::ApplicationError(
                            ApplicationError::PackageError(PackageError::MissingFunctionRoyalty {
                                blueprint: blueprint.clone(),
                                ident: name.clone(),
                            }),
                        ));
                    }
                }

                RoyaltyUtil::verify_royalty_amounts(function_royalties.values(), false, api)?;
            }
        }
    }

    Ok(())
}

fn validate_auth(definition: &PackageDefinition) -> Result<(), PackageError> {
    for (blueprint, definition_init) in &definition.blueprints {
        match &definition_init.auth_config.function_auth {
            FunctionAuth::AllowAll | FunctionAuth::RootOnly => {}
            FunctionAuth::AccessRules(functions) => {
                let num_functions = definition_init
                    .schema
                    .functions
                    .functions
                    .values()
                    .filter(|schema| schema.receiver.is_none())
                    .count();

                if num_functions != functions.len() {
                    return Err(PackageError::UnexpectedNumberOfFunctionAuth {
                        blueprint: blueprint.clone(),
                        expected: num_functions,
                        actual: functions.len(),
                    });
                }

                for (name, schema_init) in &definition_init.schema.functions.functions {
                    if schema_init.receiver.is_none() && !functions.contains_key(name) {
                        return Err(PackageError::MissingFunctionPermission {
                            blueprint: blueprint.clone(),
                            ident: name.clone(),
                        });
                    }
                }
            }
        }

        match (
            &definition_init.blueprint_type,
            &definition_init.auth_config.method_auth,
        ) {
            (_, MethodAuthTemplate::AllowAll) => {}
            (blueprint_type, MethodAuthTemplate::StaticRoles(StaticRoles { roles, methods })) => {
                let role_specification = match (blueprint_type, roles) {
                    (_, RoleSpecification::Normal(roles)) => roles,
                    (BlueprintType::Inner { outer_blueprint }, RoleSpecification::UseOuter) => {
                        if let Some(blueprint) = definition.blueprints.get(outer_blueprint) {
                            match &blueprint.auth_config.method_auth {
                                MethodAuthTemplate::StaticRoles(StaticRoles {
                                    roles: RoleSpecification::Normal(roles),
                                    ..
                                }) => roles,
                                _ => return Err(PackageError::InvalidAuthSetup),
                            }
                        } else {
                            return Err(PackageError::InvalidAuthSetup);
                        }
                    }
                    _ => {
                        return Err(PackageError::InvalidAuthSetup);
                    }
                };

                let check_list = |list: &RoleList| {
                    for role_key in &list.list {
                        if AccessRulesNativePackage::is_reserved_role_key(role_key) {
                            continue;
                        }
                        if !role_specification.contains_key(role_key) {
                            return Err(PackageError::MissingRole(role_key.clone()));
                        }
                    }
                    Ok(())
                };

                if let RoleSpecification::Normal(roles) = roles {
                    for (role_key, role_list) in roles {
                        check_list(role_list)?;
                        if AccessRulesNativePackage::is_reserved_role_key(role_key) {
                            return Err(PackageError::DefiningReservedRoleKey(
                                blueprint.to_string(),
                                role_key.clone(),
                            ));
                        }
                    }
                }

                for (_method, accessibility) in methods {
                    match accessibility {
                        MethodAccessibility::RoleProtected(role_list) => {
                            check_list(role_list)?;
                        }
                        MethodAccessibility::Public | MethodAccessibility::OuterObjectOnly => {}
                    }
                }

                let num_methods = definition_init
                    .schema
                    .functions
                    .functions
                    .values()
                    .filter(|schema| schema.receiver.is_some())
                    .count();

                if num_methods != methods.len() {
                    return Err(PackageError::UnexpectedNumberOfMethodAuth {
                        blueprint: blueprint.clone(),
                        expected: num_methods,
                        actual: methods.len(),
                    });
                }

                for (name, schema_init) in &definition_init.schema.functions.functions {
                    if schema_init.receiver.is_some()
                        && !methods.contains_key(&MethodKey::new(name))
                    {
                        return Err(PackageError::MissingMethodPermission {
                            blueprint: blueprint.clone(),
                            ident: name.clone(),
                        });
                    }
                }
            }
        }
    }

    Ok(())
}

const SECURIFY_OWNER_ROLE: &str = "securify_owner";

struct SecurifiedPackage;

impl SecurifiedAccessRules for SecurifiedPackage {
    type OwnerBadgeNonFungibleData = PackageOwnerBadgeData;
    const OWNER_BADGE: ResourceAddress = PACKAGE_OWNER_BADGE;
}

pub fn create_bootstrap_package_partitions(
    package_structure: PackageStructure,
    metadata: MetadataInit,
) -> NodeSubstates {
    let mut partitions: NodeSubstates = BTreeMap::new();

    {
        let blueprints_partition = package_structure
            .definitions
            .into_iter()
            .map(|(blueprint, definition)| {
                let key = BlueprintVersionKey {
                    blueprint,
                    version: BlueprintVersion::default(),
                };
                let value = KeyValueEntrySubstate::locked_entry(definition);
                (
                    SubstateKey::Map(scrypto_encode(&key).unwrap()),
                    IndexedScryptoValue::from_typed(&value),
                )
            })
            .collect();

        partitions.insert(
            MAIN_BASE_PARTITION
                .at_offset(PACKAGE_BLUEPRINTS_PARTITION_OFFSET)
                .unwrap(),
            blueprints_partition,
        );
    };

    {
        let minor_version_configs = package_structure
            .dependencies
            .into_iter()
            .map(|(blueprint, minor_version_config)| {
                let key = BlueprintVersionKey {
                    blueprint,
                    version: BlueprintVersion::default(),
                };

                let value = KeyValueEntrySubstate::locked_entry(minor_version_config);
                (
                    SubstateKey::Map(scrypto_encode(&key).unwrap()),
                    IndexedScryptoValue::from_typed(&value),
                )
            })
            .collect();

        partitions.insert(
            MAIN_BASE_PARTITION
                .at_offset(PACKAGE_BLUEPRINT_DEPENDENCIES_PARTITION_OFFSET)
                .unwrap(),
            minor_version_configs,
        );
    };

    {
        let schemas_partition = package_structure
            .schemas
            .into_iter()
            .map(|(hash, schema)| {
                let value = KeyValueEntrySubstate::locked_entry(schema);

                (
                    SubstateKey::Map(scrypto_encode(&hash).unwrap()),
                    IndexedScryptoValue::from_typed(&value),
                )
            })
            .collect();

        partitions.insert(
            MAIN_BASE_PARTITION
                .at_offset(PACKAGE_SCHEMAS_PARTITION_OFFSET)
                .unwrap(),
            schemas_partition,
        );
    }

    {
        let vm_type_partition = package_structure
            .vm_type
            .into_iter()
            .map(|(hash, code_substate)| {
                let value = KeyValueEntrySubstate::locked_entry(code_substate);
                (
                    SubstateKey::Map(scrypto_encode(&hash).unwrap()),
                    IndexedScryptoValue::from_typed(&value),
                )
            })
            .collect();

        partitions.insert(
            MAIN_BASE_PARTITION
                .at_offset(PACKAGE_VM_TYPE_PARTITION_OFFSET)
                .unwrap(),
            vm_type_partition,
        );
    }

    {
        let original_code_partition = package_structure
            .original_code
            .into_iter()
            .map(|(hash, code_substate)| {
                let value = KeyValueEntrySubstate::locked_entry(code_substate);
                (
                    SubstateKey::Map(scrypto_encode(&hash).unwrap()),
                    IndexedScryptoValue::from_typed(&value),
                )
            })
            .collect();

        partitions.insert(
            MAIN_BASE_PARTITION
                .at_offset(PACKAGE_ORIGINAL_CODE_PARTITION_OFFSET)
                .unwrap(),
            original_code_partition,
        );
    }

    {
        let instrumented_code_partition = package_structure
            .instrumented_code
            .into_iter()
            .map(|(hash, code_substate)| {
                let value = KeyValueEntrySubstate::locked_entry(code_substate);
                (
                    SubstateKey::Map(scrypto_encode(&hash).unwrap()),
                    IndexedScryptoValue::from_typed(&value),
                )
            })
            .collect();

        partitions.insert(
            MAIN_BASE_PARTITION
                .at_offset(PACKAGE_INSTRUMENTED_CODE_PARTITION_OFFSET)
                .unwrap(),
            instrumented_code_partition,
        );
    }

    {
        let auth_partition = package_structure
            .auth_configs
            .into_iter()
            .map(|(blueprint, auth_template)| {
                let key = BlueprintVersionKey {
                    blueprint,
                    version: BlueprintVersion::default(),
                };
                let value = KeyValueEntrySubstate::locked_entry(auth_template);
                (
                    SubstateKey::Map(scrypto_encode(&key).unwrap()),
                    IndexedScryptoValue::from_typed(&value),
                )
            })
            .collect();

        partitions.insert(
            MAIN_BASE_PARTITION
                .at_offset(PACKAGE_AUTH_TEMPLATE_PARTITION_OFFSET)
                .unwrap(),
            auth_partition,
        );
    }

    {
        let mut metadata_partition = BTreeMap::new();
        for (key, value) in metadata.data {
            let mutability = if value.lock {
                SubstateMutability::Immutable
            } else {
                SubstateMutability::Mutable
            };
            let value = MetadataEntrySubstate {
                value: value.value,
                mutability,
            };

            metadata_partition.insert(
                SubstateKey::Map(scrypto_encode(&key).unwrap()),
                IndexedScryptoValue::from_typed(&value),
            );
        }
        partitions.insert(METADATA_KV_STORE_PARTITION, metadata_partition);
    }

    {
        partitions.insert(
            TYPE_INFO_FIELD_PARTITION,
            type_info_partition(TypeInfoSubstate::Object(ObjectInfo {
                global: true,
                blueprint_id: BlueprintId::new(&PACKAGE_PACKAGE, PACKAGE_BLUEPRINT),
                version: BlueprintVersion::default(),
                blueprint_info: ObjectBlueprintInfo::default(),
                features: btreeset!(),
                instance_schema: None,
            })),
        );
    }

    partitions
}

fn globalize_package<Y>(
    package_address_reservation: Option<GlobalAddressReservation>,
    package_structure: PackageStructure,
    metadata: Own,
    access_rules: AccessRules,
    api: &mut Y,
) -> Result<PackageAddress, RuntimeError>
where
    Y: ClientApi<RuntimeError>,
{
    let mut kv_entries: BTreeMap<u8, BTreeMap<Vec<u8>, KVEntry>> = BTreeMap::new();

    let vault = ResourceManager(XRD).new_empty_vault(api)?;
    let royalty = PackageRoyaltyAccumulatorSubstate {
        royalty_vault: Vault(vault),
    };

    {
        let mut definition_partition = BTreeMap::new();
        for (blueprint, definition) in package_structure.definitions {
            let key = BlueprintVersionKey::new_default(blueprint);
            let entry = KVEntry {
                value: Some(scrypto_encode(&definition).unwrap()),
                locked: true,
            };
            definition_partition.insert(scrypto_encode(&key).unwrap(), entry);
        }
        kv_entries.insert(0u8, definition_partition);
    }

    {
        let mut dependency_partition = BTreeMap::new();
        for (blueprint, dependencies) in package_structure.dependencies {
            let key = BlueprintVersionKey::new_default(blueprint);
            let entry = KVEntry {
                value: Some(scrypto_encode(&dependencies).unwrap()),
                locked: true,
            };
            dependency_partition.insert(scrypto_encode(&key).unwrap(), entry);
        }
        kv_entries.insert(1u8, dependency_partition);
    }

    {
        let mut schemas_partition = BTreeMap::new();
        for (hash, schema) in package_structure.schemas {
            let entry = KVEntry {
                value: Some(scrypto_encode(&schema).unwrap()),
                locked: true,
            };
            schemas_partition.insert(scrypto_encode(&hash).unwrap(), entry);
        }
        kv_entries.insert(2u8, schemas_partition);
    }

    {
        let mut package_royalties_partition = BTreeMap::new();
        for (blueprint, package_royalty) in package_structure.package_royalties {
            let key = BlueprintVersionKey::new_default(blueprint);
            let entry = KVEntry {
                value: Some(scrypto_encode(&package_royalty).unwrap()),
                locked: true,
            };
            package_royalties_partition.insert(scrypto_encode(&key).unwrap(), entry);
        }
        kv_entries.insert(3u8, package_royalties_partition);
    }

    {
        let mut auth_partition = BTreeMap::new();
        for (blueprint, auth_config) in package_structure.auth_configs {
            let key = BlueprintVersionKey::new_default(blueprint);
            let entry = KVEntry {
                value: Some(scrypto_encode(&auth_config).unwrap()),
                locked: true,
            };
            auth_partition.insert(scrypto_encode(&key).unwrap(), entry);
        }
        kv_entries.insert(4u8, auth_partition);
    }

    {
        let mut vm_type_partition = BTreeMap::new();
        for (hash, code_substate) in package_structure.vm_type {
            let entry = KVEntry {
                value: Some(scrypto_encode(&code_substate).unwrap()),
                locked: true,
            };
            vm_type_partition.insert(scrypto_encode(&hash).unwrap(), entry);
        }
        kv_entries.insert(5u8, vm_type_partition);
    }

    {
        let mut original_code_partition = BTreeMap::new();
        for (hash, code_substate) in package_structure.original_code {
            let entry = KVEntry {
                value: Some(scrypto_encode(&code_substate).unwrap()),
                locked: true,
            };
            original_code_partition.insert(scrypto_encode(&hash).unwrap(), entry);
        }
        kv_entries.insert(6u8, original_code_partition);
    }

    {
        let mut instrumented_code_partition = BTreeMap::new();
        for (hash, code_substate) in package_structure.instrumented_code {
            let entry = KVEntry {
                value: Some(scrypto_encode(&code_substate).unwrap()),
                locked: true,
            };
            instrumented_code_partition.insert(scrypto_encode(&hash).unwrap(), entry);
        }
        kv_entries.insert(7u8, instrumented_code_partition);
    }

    let package_object = api.new_object(
        PACKAGE_BLUEPRINT,
        vec![PACKAGE_ROYALTY_FEATURE],
        None,
        vec![scrypto_encode(&royalty).unwrap()],
        kv_entries,
    )?;

    let royalty = ComponentRoyalty::create(ComponentRoyaltyConfig::Disabled, api)?;

    let address = api.globalize(
        btreemap!(
            ObjectModuleId::Main => package_object,
            ObjectModuleId::Metadata => metadata.0,
            ObjectModuleId::Royalty => royalty.0,
            ObjectModuleId::AccessRules => access_rules.0.0,
        ),
        package_address_reservation,
    )?;

    Ok(PackageAddress::new_or_panic(address.into_node_id().0))
}

pub struct PackageStructure {
    pub definitions: BTreeMap<String, BlueprintDefinition>,
    pub dependencies: BTreeMap<String, BlueprintDependencies>,
    pub schemas: BTreeMap<Hash, ScryptoSchema>,
    pub vm_type: BTreeMap<Hash, PackageVmTypeSubstate>,
    pub original_code: BTreeMap<Hash, PackageOriginalCodeSubstate>,
    pub instrumented_code: BTreeMap<Hash, PackageInstrumentedCodeSubstate>,
    pub auth_configs: BTreeMap<String, AuthConfig>,
    pub package_royalties: BTreeMap<String, PackageRoyaltyConfig>,
}

pub struct PackageNativePackage;

impl PackageNativePackage {
    pub fn definition() -> PackageDefinition {
        let mut aggregator = TypeAggregator::<ScryptoCustomTypeKind>::new();

        let mut fields = Vec::new();
        fields.push(FieldSchema::if_feature(
            aggregator.add_child_type_and_descendents::<PackageRoyaltyAccumulatorSubstate>(),
            PACKAGE_ROYALTY_FEATURE,
        ));

        let mut collections = Vec::new();
        collections.push(BlueprintCollectionSchema::KeyValueStore(
            BlueprintKeyValueStoreSchema {
                key: TypeRef::Static(
                    aggregator.add_child_type_and_descendents::<BlueprintVersionKey>(),
                ),
                value: TypeRef::Static(
                    aggregator.add_child_type_and_descendents::<BlueprintDefinition>(),
                ),
                can_own: false,
            },
        ));
        collections.push(BlueprintCollectionSchema::KeyValueStore(
            BlueprintKeyValueStoreSchema {
                key: TypeRef::Static(
                    aggregator.add_child_type_and_descendents::<BlueprintVersionKey>(),
                ),
                value: TypeRef::Static(
                    aggregator.add_child_type_and_descendents::<BlueprintDependencies>(),
                ),
                can_own: false,
            },
        ));
        collections.push(BlueprintCollectionSchema::KeyValueStore(
            BlueprintKeyValueStoreSchema {
                key: TypeRef::Static(aggregator.add_child_type_and_descendents::<Hash>()),
                value: TypeRef::Static(
                    aggregator.add_child_type_and_descendents::<ScryptoSchema>(),
                ),
                can_own: false,
            },
        ));
        collections.push(BlueprintCollectionSchema::KeyValueStore(
            BlueprintKeyValueStoreSchema {
                key: TypeRef::Static(
                    aggregator.add_child_type_and_descendents::<BlueprintVersionKey>(),
                ),
                value: TypeRef::Static(
                    aggregator.add_child_type_and_descendents::<PackageRoyaltyConfig>(),
                ),
                can_own: false,
            },
        ));
        collections.push(BlueprintCollectionSchema::KeyValueStore(
            BlueprintKeyValueStoreSchema {
                key: TypeRef::Static(
                    aggregator.add_child_type_and_descendents::<BlueprintVersionKey>(),
                ),
                value: TypeRef::Static(aggregator.add_child_type_and_descendents::<AuthConfig>()),
                can_own: false,
            },
        ));
        collections.push(BlueprintCollectionSchema::KeyValueStore(
            BlueprintKeyValueStoreSchema {
                key: TypeRef::Static(aggregator.add_child_type_and_descendents::<Hash>()),
                value: TypeRef::Static(
                    aggregator.add_child_type_and_descendents::<PackageVmTypeSubstate>(),
                ),
                can_own: false,
            },
        ));
        collections.push(BlueprintCollectionSchema::KeyValueStore(
            BlueprintKeyValueStoreSchema {
                key: TypeRef::Static(aggregator.add_child_type_and_descendents::<Hash>()),
                value: TypeRef::Static(
                    aggregator.add_child_type_and_descendents::<PackageOriginalCodeSubstate>(),
                ),
                can_own: false,
            },
        ));
        collections.push(BlueprintCollectionSchema::KeyValueStore(
            BlueprintKeyValueStoreSchema {
                key: TypeRef::Static(aggregator.add_child_type_and_descendents::<Hash>()),
                value: TypeRef::Static(
                    aggregator.add_child_type_and_descendents::<PackageInstrumentedCodeSubstate>(),
                ),
                can_own: false,
            },
        ));

        let mut functions = BTreeMap::new();
        functions.insert(
            PACKAGE_PUBLISH_WASM_IDENT.to_string(),
            FunctionSchemaInit {
                receiver: None,
                input: TypeRef::Static(
                    aggregator.add_child_type_and_descendents::<PackagePublishWasmInput>(),
                ),
                output: TypeRef::Static(
                    aggregator.add_child_type_and_descendents::<PackagePublishWasmOutput>(),
                ),
                export: PACKAGE_PUBLISH_WASM_IDENT.to_string(),
            },
        );
        functions.insert(
            PACKAGE_PUBLISH_WASM_ADVANCED_IDENT.to_string(),
            FunctionSchemaInit {
                receiver: None,
                input: TypeRef::Static(
                    aggregator.add_child_type_and_descendents::<PackagePublishWasmAdvancedInput>(),
                ),
                output: TypeRef::Static(
                    aggregator.add_child_type_and_descendents::<PackagePublishWasmAdvancedOutput>(),
                ),
                export: PACKAGE_PUBLISH_WASM_ADVANCED_IDENT.to_string(),
            },
        );
        functions.insert(
            PACKAGE_PUBLISH_NATIVE_IDENT.to_string(),
            FunctionSchemaInit {
                receiver: None,
                input: TypeRef::Static(
                    aggregator.add_child_type_and_descendents::<PackagePublishNativeInput>(),
                ),
                output: TypeRef::Static(
                    aggregator.add_child_type_and_descendents::<PackagePublishNativeOutput>(),
                ),
                export: PACKAGE_PUBLISH_NATIVE_IDENT.to_string(),
            },
        );
        functions.insert(
            PACKAGE_CLAIM_ROYALTIES_IDENT.to_string(),
            FunctionSchemaInit {
                receiver: Some(ReceiverInfo::normal_ref_mut()),
                input: TypeRef::Static(
                    aggregator.add_child_type_and_descendents::<PackageClaimRoyaltiesInput>(),
                ),
                output: TypeRef::Static(
                    aggregator.add_child_type_and_descendents::<PackageClaimRoyaltiesOutput>(),
                ),
                export: PACKAGE_CLAIM_ROYALTIES_IDENT.to_string(),
            },
        );

        let schema = generate_full_schema(aggregator);
        let blueprints = btreemap!(
            PACKAGE_BLUEPRINT.to_string() => BlueprintDefinitionInit {
                blueprint_type: BlueprintType::default(),
                feature_set: btreeset!(
                    PACKAGE_ROYALTY_FEATURE.to_string(),
                ),
                dependencies: btreeset!(
                    PACKAGE_OF_DIRECT_CALLER_VIRTUAL_BADGE.into(),
                    PACKAGE_OWNER_BADGE.into(),
                ),

                schema: BlueprintSchemaInit {
                    generics: vec![],
                    schema,
                    state: BlueprintStateSchemaInit {
                        fields,
                        collections,
                    },
                    events: BlueprintEventSchemaInit::default(),
                    functions: BlueprintFunctionsSchemaInit {
                        virtual_lazy_load_functions: btreemap!(),
                        functions,
                    },
                },

                royalty_config: PackageRoyaltyConfig::default(),
                auth_config: AuthConfig {
                    function_auth: FunctionAuth::AccessRules(
                        btreemap!(
                            PACKAGE_PUBLISH_WASM_IDENT.to_string() => rule!(require(package_of_direct_caller(TRANSACTION_PROCESSOR_PACKAGE))),
                            PACKAGE_PUBLISH_WASM_ADVANCED_IDENT.to_string() => rule!(require(package_of_direct_caller(TRANSACTION_PROCESSOR_PACKAGE))),
                            PACKAGE_PUBLISH_NATIVE_IDENT.to_string() => rule!(require(SYSTEM_TRANSACTION_BADGE)),
                        )
                    ),
                    method_auth: MethodAuthTemplate::StaticRoles(
                        roles_template! {
                            roles {
                                SECURIFY_OWNER_ROLE;
                            },
                            methods {
                                PACKAGE_CLAIM_ROYALTIES_IDENT => [SECURIFY_OWNER_ROLE];
                            }
                        },
                    ),
                },
            }
        );

        PackageDefinition { blueprints }
    }

    pub fn invoke_export<Y>(
        export_name: &str,
        input: &IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        match export_name {
            PACKAGE_PUBLISH_NATIVE_IDENT => {
                let input: PackagePublishNativeInput = input.as_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;

                let rtn = Self::publish_native(
                    input.package_address,
                    input.native_package_code_id,
                    input.setup,
                    input.metadata,
                    api,
                )?;

                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            PACKAGE_PUBLISH_WASM_IDENT => {
                let input: PackagePublishWasmInput = input.as_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;

                let rtn = Self::publish_wasm(input.code, input.setup, input.metadata, api)?;

                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            PACKAGE_PUBLISH_WASM_ADVANCED_IDENT => {
                let input: PackagePublishWasmAdvancedInput = input.as_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;

                let rtn = Self::publish_wasm_advanced(
                    input.package_address,
                    input.code,
                    input.setup,
                    input.metadata,
                    input.owner_role,
                    api,
                )?;

                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            PACKAGE_CLAIM_ROYALTIES_IDENT => {
                let _input: PackageClaimRoyaltiesInput = input.as_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;
                let rtn = PackageRoyaltyNativeBlueprint::claim_royalties(api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            _ => Err(RuntimeError::ApplicationError(
                ApplicationError::ExportDoesNotExist(export_name.to_string()),
            )),
        }
    }

    pub fn validate_and_build_package_structure(
        definition: PackageDefinition,
        vm_type: VmType,
        original_code: Vec<u8>,
    ) -> Result<PackageStructure, RuntimeError> {
        // Validate schema
        validate_package_schema(definition.blueprints.values().map(|s| &s.schema))
            .map_err(|e| RuntimeError::ApplicationError(ApplicationError::PackageError(e)))?;
        validate_package_event_schema(definition.blueprints.values())
            .map_err(|e| RuntimeError::ApplicationError(ApplicationError::PackageError(e)))?;
        validate_auth(&definition)
            .map_err(|e| RuntimeError::ApplicationError(ApplicationError::PackageError(e)))?;

        // Validate VM specific properties
        let instrumented_code =
            VmPackageValidation::validate(&definition, vm_type, &original_code)?;

        // Build Package structure
        let mut definitions = BTreeMap::new();
        let mut dependencies = BTreeMap::new();
        let mut schemas = BTreeMap::new();
        let mut package_royalties = BTreeMap::new();
        let mut auth_configs = BTreeMap::new();
        let mut vm_type_substates = BTreeMap::new();
        let mut original_code_substates = BTreeMap::new();
        let mut instrumented_code_substates = BTreeMap::new();

        let code_hash = hash(&original_code);
        vm_type_substates.insert(code_hash, PackageVmTypeSubstate { vm_type });
        original_code_substates.insert(
            code_hash,
            PackageOriginalCodeSubstate {
                code: original_code,
            },
        );
        if let Some(code) = instrumented_code {
            instrumented_code_substates
                .insert(code_hash, PackageInstrumentedCodeSubstate { code: code });
        };

        {
            for (blueprint, definition_init) in definition.blueprints {
                auth_configs.insert(blueprint.clone(), definition_init.auth_config);

                let blueprint_schema = definition_init.schema.schema.clone();
                let schema_hash = hash(scrypto_encode(&blueprint_schema).unwrap());
                schemas.insert(schema_hash, blueprint_schema);

                let mut functions = BTreeMap::new();
                let mut function_exports = BTreeMap::new();
                for (function, function_schema_init) in definition_init.schema.functions.functions {
                    let input = match function_schema_init.input {
                        TypeRef::Static(input_type_index) => input_type_index,
                        TypeRef::Generic(..) => {
                            return Err(RuntimeError::ApplicationError(
                                ApplicationError::PackageError(PackageError::WasmUnsupported(
                                    "Generics not supported".to_string(),
                                )),
                            ))
                        }
                    };
                    let output = match function_schema_init.output {
                        TypeRef::Static(output_type_index) => output_type_index,
                        TypeRef::Generic(..) => {
                            return Err(RuntimeError::ApplicationError(
                                ApplicationError::PackageError(PackageError::WasmUnsupported(
                                    "Generics not supported".to_string(),
                                )),
                            ))
                        }
                    };
                    functions.insert(
                        function.clone(),
                        FunctionSchema {
                            receiver: function_schema_init.receiver,
                            input: TypePointer::Package(schema_hash, input),
                            output: TypePointer::Package(schema_hash, output),
                        },
                    );
                    let export = PackageExport {
                        code_hash,
                        export_name: function_schema_init.export.clone(),
                    };
                    function_exports.insert(function, export);
                }

                let mut events = BTreeMap::new();
                for (key, type_ref) in definition_init.schema.events.event_schema {
                    let index = match type_ref {
                        TypeRef::Static(index) => TypePointer::Package(schema_hash, index),
                        TypeRef::Generic(index) => TypePointer::Instance(index),
                    };
                    events.insert(key, index);
                }

                let definition = BlueprintDefinition {
                    interface: BlueprintInterface {
                        blueprint_type: definition_init.blueprint_type,
                        generics: definition_init.schema.generics,
                        feature_set: definition_init.feature_set,
                        functions,
                        events,
                        state: IndexedStateSchema::from_schema(
                            schema_hash,
                            definition_init.schema.state,
                        ),
                    },
                    function_exports,
                    virtual_lazy_load_functions: definition_init
                        .schema
                        .functions
                        .virtual_lazy_load_functions
                        .into_iter()
                        .map(|(key, export_name)| {
                            (
                                key,
                                PackageExport {
                                    code_hash,
                                    export_name,
                                },
                            )
                        })
                        .collect(),
                };
                definitions.insert(blueprint.clone(), definition);

                let minor_version_config = BlueprintDependencies {
                    dependencies: definition_init.dependencies,
                };
                dependencies.insert(blueprint.clone(), minor_version_config);

                package_royalties.insert(blueprint.clone(), definition_init.royalty_config);
            }
        };

        let package_structure = PackageStructure {
            definitions,
            dependencies,
            schemas,
            vm_type: vm_type_substates,
            original_code: original_code_substates,
            instrumented_code: instrumented_code_substates,
            auth_configs,
            package_royalties,
        };

        Ok(package_structure)
    }

    pub(crate) fn publish_native<Y>(
        package_address: Option<GlobalAddressReservation>,
        native_package_code_id: u64,
        definition: PackageDefinition,
        metadata_init: MetadataInit,
        api: &mut Y,
    ) -> Result<PackageAddress, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        validate_royalties(&definition, api)?;
        let package_structure = Self::validate_and_build_package_structure(
            definition,
            VmType::Native,
            native_package_code_id.to_be_bytes().to_vec(),
        )?;
        let access_rules = AccessRules::create(OwnerRole::None, btreemap!(), api)?;
        let metadata = Metadata::create_with_data(metadata_init, api)?;

        globalize_package(
            package_address,
            package_structure,
            metadata,
            access_rules,
            api,
        )
    }

    pub(crate) fn publish_wasm<Y>(
        code: Vec<u8>,
        definition: PackageDefinition,
        metadata_init: MetadataInit,
        api: &mut Y,
    ) -> Result<(PackageAddress, Bucket), RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        validate_royalties(&definition, api)?;
        let package_structure =
            Self::validate_and_build_package_structure(definition, VmType::ScryptoV1, code)?;

        let (address_reservation, address) = api.allocate_global_address(BlueprintId {
            package_address: PACKAGE_PACKAGE,
            blueprint_name: PACKAGE_BLUEPRINT.to_string(),
        })?;

        let (access_rules, bucket) = SecurifiedPackage::create_securified(
            PackageOwnerBadgeData {
                name: "Package Owner Badge".to_owned(),
                package: address.try_into().expect("Impossible Case"),
            },
            None,
            api,
        )?;
        let metadata = Metadata::create_with_data(metadata_init, api)?;

        let address = globalize_package(
            Some(address_reservation),
            package_structure,
            metadata,
            access_rules,
            api,
        )?;

        Ok((address, bucket))
    }

    pub(crate) fn publish_wasm_advanced<Y>(
        package_address: Option<GlobalAddressReservation>,
        code: Vec<u8>,
        definition: PackageDefinition,
        metadata_init: MetadataInit,
        owner_rule: OwnerRole,
        api: &mut Y,
    ) -> Result<PackageAddress, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        validate_royalties(&definition, api)?;
        let package_structure =
            Self::validate_and_build_package_structure(definition, VmType::ScryptoV1, code)?;
        let metadata = Metadata::create_with_data(metadata_init, api)?;
        let access_rules = SecurifiedPackage::create_advanced(owner_rule, api)?;

        globalize_package(
            package_address,
            package_structure,
            metadata,
            access_rules,
            api,
        )
    }
}

pub struct PackageRoyaltyNativeBlueprint;

impl PackageRoyaltyNativeBlueprint {
    pub fn charge_package_royalty<Y, V>(
        receiver: &NodeId,
        bp_version_key: &BlueprintVersionKey,
        ident: &str,
        api: &mut Y,
    ) -> Result<(), RuntimeError>
    where
        V: SystemCallbackObject,
        Y: KernelApi<SystemConfig<V>>,
    {
        {
            let mut service = SystemService::new(api);
            let object_info = service.get_object_info(receiver)?;
            if !object_info.features.contains(PACKAGE_ROYALTY_FEATURE) {
                return Ok(());
            }
        }

        let handle = api.kernel_open_substate_with_default(
            receiver,
            MAIN_BASE_PARTITION
                .at_offset(PACKAGE_ROYALTY_PARTITION_OFFSET)
                .unwrap(),
            &SubstateKey::Map(scrypto_encode(&bp_version_key).unwrap()),
            LockFlags::read_only(),
            Some(|| {
                let kv_entry = KeyValueEntrySubstate::<()>::default();
                IndexedScryptoValue::from_typed(&kv_entry)
            }),
            SystemLockData::default(),
        )?;

        let substate: KeyValueEntrySubstate<PackageRoyaltyConfig> =
            api.kernel_read_substate(handle)?.as_typed().unwrap();
        api.kernel_close_substate(handle)?;

        let royalty_charge = substate
            .value
            .and_then(|royalty_config| match royalty_config {
                PackageRoyaltyConfig::Enabled(royalty_amounts) => {
                    royalty_amounts.get(ident).cloned()
                }
                PackageRoyaltyConfig::Disabled => Some(RoyaltyAmount::Free),
            })
            .unwrap_or(RoyaltyAmount::Free);

        if royalty_charge.is_non_zero() {
            let handle = api.kernel_open_substate(
                receiver,
                MAIN_BASE_PARTITION,
                &PackageField::Royalty.into(),
                LockFlags::MUTABLE,
                SystemLockData::default(),
            )?;

            let substate: PackageRoyaltyAccumulatorSubstate =
                api.kernel_read_substate(handle)?.as_typed().unwrap();

            let vault_id = substate.royalty_vault.0;
            let package_address = PackageAddress::new_or_panic(receiver.0);
            apply_royalty_cost(
                api,
                royalty_charge,
                RoyaltyRecipient::Package(package_address),
                vault_id.0,
            )?;

            api.kernel_close_substate(handle)?;
        }

        Ok(())
    }

    pub(crate) fn claim_royalties<Y>(api: &mut Y) -> Result<Bucket, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        if !api.actor_is_feature_enabled(OBJECT_HANDLE_SELF, PACKAGE_ROYALTY_FEATURE)? {
            return Err(RuntimeError::ApplicationError(
                ApplicationError::PackageError(PackageError::RoyaltiesNotEnabled),
            ));
        }

        let handle = api.actor_open_field(
            OBJECT_HANDLE_SELF,
            PackageField::Royalty.into(),
            LockFlags::read_only(),
        )?;

        let mut substate: PackageRoyaltyAccumulatorSubstate = api.field_lock_read_typed(handle)?;
        let bucket = substate.royalty_vault.take_all(api)?;

        Ok(bucket)
    }
}

pub struct PackageAuthNativeBlueprint;

impl PackageAuthNativeBlueprint {
    pub fn resolve_function_permission<Y, V>(
        receiver: &NodeId,
        bp_version_key: &BlueprintVersionKey,
        ident: &str,
        api: &mut Y,
    ) -> Result<ResolvedPermission, RuntimeError>
    where
        Y: KernelSubstateApi<SystemLockData> + KernelApi<SystemConfig<V>>,
        V: SystemCallbackObject,
    {
        let auth_config = Self::get_bp_auth_template(receiver, bp_version_key, api)?;
        match auth_config.function_auth {
            FunctionAuth::AllowAll => Ok(ResolvedPermission::AllowAll),
            FunctionAuth::RootOnly => {
                if api.kernel_get_current_depth() == 0 {
                    Ok(ResolvedPermission::AllowAll)
                } else {
                    Ok(ResolvedPermission::AccessRule(AccessRule::DenyAll))
                }
            }
            FunctionAuth::AccessRules(access_rules) => {
                let access_rule = access_rules.get(ident);
                if let Some(access_rule) = access_rule {
                    Ok(ResolvedPermission::AccessRule(access_rule.clone()))
                } else {
                    let package_address = PackageAddress::new_or_panic(receiver.0.clone());
                    let blueprint_id =
                        BlueprintId::new(&package_address, &bp_version_key.blueprint);
                    Err(RuntimeError::SystemModuleError(
                        SystemModuleError::AuthError(AuthError::NoFunction(FnIdentifier {
                            blueprint_id,
                            ident: FnIdent::Application(ident.to_string()),
                        })),
                    ))
                }
            }
        }
    }

    pub fn get_bp_auth_template<Y, V>(
        receiver: &NodeId,
        bp_version_key: &BlueprintVersionKey,
        api: &mut Y,
    ) -> Result<AuthConfig, RuntimeError>
    where
        Y: KernelSubstateApi<SystemLockData> + KernelApi<SystemConfig<V>>,
        V: SystemCallbackObject,
    {
        let package_bp_version_id = CanonicalBlueprintId {
            address: PackageAddress::new_or_panic(receiver.0.clone()),
            blueprint: bp_version_key.blueprint.to_string(),
            version: bp_version_key.version.clone(),
        };

        let auth_template = api
            .kernel_get_system_state()
            .system
            .auth_cache
            .get(&package_bp_version_id);
        if let Some(auth_template) = auth_template {
            return Ok(auth_template.clone());
        }

        let handle = api.kernel_open_substate_with_default(
            receiver,
            MAIN_BASE_PARTITION
                .at_offset(PACKAGE_AUTH_TEMPLATE_PARTITION_OFFSET)
                .unwrap(),
            &SubstateKey::Map(scrypto_encode(&bp_version_key).unwrap()),
            LockFlags::read_only(),
            Some(|| {
                let kv_entry = KeyValueEntrySubstate::<()>::default();
                IndexedScryptoValue::from_typed(&kv_entry)
            }),
            SystemLockData::default(),
        )?;

        let auth_template: KeyValueEntrySubstate<AuthConfig> =
            api.kernel_read_substate(handle)?.as_typed().unwrap();
        api.kernel_close_substate(handle)?;

        let template = match auth_template.value {
            Some(template) => template,
            None => {
                return Err(RuntimeError::SystemError(
                    SystemError::AuthTemplateDoesNotExist(package_bp_version_id),
                ))
            }
        };

        api.kernel_get_system_state()
            .system
            .auth_cache
            .insert(package_bp_version_id, template.clone());

        Ok(template)
    }
}

#[derive(ScryptoSbor)]
pub struct PackageOwnerBadgeData {
    pub name: String,
    pub package: PackageAddress,
}

impl NonFungibleData for PackageOwnerBadgeData {
    const MUTABLE_FIELDS: &'static [&'static str] = &[];
}
