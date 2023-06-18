use crate::blueprints::util::SecurifiedAccessRules;
use crate::errors::*;
use crate::kernel::kernel_api::{KernelApi, KernelNodeApi, KernelSubstateApi};
use crate::system::node_init::type_info_partition;
use crate::system::node_modules::type_info::TypeInfoSubstate;
use crate::system::system_modules::costing::{
    apply_royalty_cost, RoyaltyRecipient, FIXED_HIGH_FEE, FIXED_MEDIUM_FEE,
};
use crate::track::interface::NodeSubstates;
use crate::types::*;
use crate::vm::wasm::{PrepareError, WasmValidator};
use native_sdk::modules::access_rules::AccessRules;
use native_sdk::resource::NativeVault;
use native_sdk::resource::ResourceManager;
use radix_engine_interface::api::node_modules::metadata::MetadataValue;
use radix_engine_interface::api::{ClientApi, LockFlags, ObjectModuleId, OBJECT_HANDLE_SELF};
pub use radix_engine_interface::blueprints::package::*;
use radix_engine_interface::blueprints::resource::{require, Bucket};
use radix_engine_interface::schema::{
    BlueprintCollectionSchema, BlueprintEventSchemaInit, BlueprintFunctionsSchemaInit,
    BlueprintKeyValueStoreSchema, BlueprintSchemaInit, BlueprintStateSchemaInit, FieldSchema,
    FunctionSchemaInit, RefTypes, TypeRef,
};
use resources_tracker_macro::trace_resources;
use sbor::LocalTypeIndex;

// Import and re-export substate types
use crate::method_auth_template;
use crate::system::system::{KeyValueEntrySubstate, SystemService};
use crate::system::system_callback::{SystemConfig, SystemLockData};
use crate::system::system_callback_api::SystemCallbackObject;
use crate::system::system_modules::auth::{AuthError, ResolvedPermission};
pub use radix_engine_interface::blueprints::package::{
    PackageCodeSubstate, PackageRoyaltyAccumulatorSubstate,
};

pub const PACKAGE_ROYALTY_AUTHORITY: &str = "package_royalty";

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

    InvalidMetadataKey(String),
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

struct SecurifiedPackage;

impl SecurifiedAccessRules for SecurifiedPackage {
    const OWNER_BADGE: ResourceAddress = PACKAGE_OWNER_BADGE;
}

fn globalize_package<Y, L: Default>(
    package_address_reservation: Option<GlobalAddressReservation>,
    blueprints: BTreeMap<String, BlueprintDefinition>,
    blueprint_dependencies: BTreeMap<String, BlueprintDependencies>,

    schemas: BTreeMap<Hash, ScryptoSchema>,
    code: PackageCodeSubstate,
    code_hash: Hash,

    package_royalties: BTreeMap<String, RoyaltyConfig>,

    auth_configs: BTreeMap<String, AuthConfig>,

    metadata: BTreeMap<String, MetadataValue>,
    access_rules: Option<AccessRules>,
    api: &mut Y,
) -> Result<PackageAddress, RuntimeError>
where
    Y: KernelNodeApi + KernelSubstateApi<L> + ClientApi<RuntimeError>,
{
    // Use kernel API to commit substates directly.
    // Can't use the ClientApi because of chicken-and-egg issue.

    let mut partitions: NodeSubstates = BTreeMap::new();

    let royalty = PackageRoyaltyAccumulatorSubstate {
        royalty_vault: None,
    };

    // Prepare node init.
    {
        let main_partition = btreemap!(
            PackageField::Royalty.into() => IndexedScryptoValue::from_typed(&royalty),
        );
        partitions.insert(
            MAIN_BASE_PARTITION
                .at_offset(PACKAGE_FIELDS_PARTITION_OFFSET)
                .unwrap(),
            main_partition,
        );
    }

    {
        let blueprints_partition = blueprints
            .into_iter()
            .map(|(blueprint, definition)| {
                let key = BlueprintVersionKey {
                    blueprint,
                    version: BlueprintVersion::default(),
                };
                let value = KeyValueEntrySubstate::immutable_entry(definition);
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
        let minor_version_configs = blueprint_dependencies
            .into_iter()
            .map(|(blueprint, minor_version_config)| {
                let key = BlueprintVersionKey {
                    blueprint,
                    version: BlueprintVersion::default(),
                };

                let value = KeyValueEntrySubstate::immutable_entry(minor_version_config);
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
        let schemas_partition = schemas
            .into_iter()
            .map(|(hash, schema)| {
                let value = KeyValueEntrySubstate::immutable_entry(schema);

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
        let value = KeyValueEntrySubstate::immutable_entry(code);
        partitions.insert(
            MAIN_BASE_PARTITION
                .at_offset(PACKAGE_CODE_PARTITION_OFFSET)
                .unwrap(),
            btreemap! (
                SubstateKey::Map(scrypto_encode(&code_hash).unwrap()) => IndexedScryptoValue::from_typed(&value),
            ),
        );
    };

    {
        let royalty_partition = package_royalties
            .into_iter()
            .map(|(blueprint, royalty)| {
                let key = BlueprintVersionKey {
                    blueprint,
                    version: BlueprintVersion::default(),
                };
                let value = KeyValueEntrySubstate::immutable_entry(royalty);
                (
                    SubstateKey::Map(scrypto_encode(&key).unwrap()),
                    IndexedScryptoValue::from_typed(&value),
                )
            })
            .collect();

        partitions.insert(
            MAIN_BASE_PARTITION
                .at_offset(PACKAGE_ROYALTY_PARTITION_OFFSET)
                .unwrap(),
            royalty_partition,
        );
    };

    {
        let auth_partition = auth_configs
            .into_iter()
            .map(|(blueprint, auth_template)| {
                let key = BlueprintVersionKey {
                    blueprint,
                    version: BlueprintVersion::default(),
                };
                let value = KeyValueEntrySubstate::immutable_entry(auth_template);
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

    partitions.insert(
        TYPE_INFO_FIELD_PARTITION,
        type_info_partition(TypeInfoSubstate::Object(ObjectInfo {
            global: true,

            blueprint_id: BlueprintId::new(&PACKAGE_PACKAGE, PACKAGE_BLUEPRINT),
            version: BlueprintVersion::default(),

            blueprint_info: ObjectBlueprintInfo::default(),
            instance_schema: None,
        })),
    );
    let metadata_partition = {
        let mut metadata_partition = BTreeMap::new();
        for (key, value) in metadata {
            let value = KeyValueEntrySubstate::entry(value);
            metadata_partition.insert(
                SubstateKey::Map(scrypto_encode(&key).unwrap()),
                IndexedScryptoValue::from_typed(&value),
            );
        }
        metadata_partition
    };
    partitions.insert(METADATA_KV_STORE_PARTITION, metadata_partition);

    let package_address = if let Some(address_reservation) = package_address_reservation {
        // TODO: Can we use `global_object` API?

        // Check global address reservation
        let global_address = {
            let substates = api.kernel_drop_node(address_reservation.0.as_node_id())?;

            let type_info: Option<TypeInfoSubstate> = substates
                .get(&TYPE_INFO_FIELD_PARTITION)
                .and_then(|x| x.get(&TypeInfoField::TypeInfo.into()))
                .and_then(|x| x.as_typed().ok());

            match type_info {
                Some(TypeInfoSubstate::GlobalAddressReservation(x)) => x,
                _ => {
                    return Err(RuntimeError::SystemError(
                        SystemError::InvalidGlobalAddressReservation,
                    ));
                }
            }
        };

        // Check blueprint id
        let reserved_blueprint_id = {
            let lock_handle = api.kernel_lock_substate(
                global_address.as_node_id(),
                TYPE_INFO_FIELD_PARTITION,
                &TypeInfoField::TypeInfo.into(),
                LockFlags::MUTABLE, // This is to ensure the substate is lock free!
                L::default(),
            )?;
            let type_info: TypeInfoSubstate =
                api.kernel_read_substate(lock_handle)?.as_typed().unwrap();
            api.kernel_drop_lock(lock_handle)?;
            match type_info {
                TypeInfoSubstate::GlobalAddressPhantom(GlobalAddressPhantom { blueprint_id }) => {
                    blueprint_id
                }
                _ => unreachable!(),
            }
        };

        if reserved_blueprint_id.package_address != PACKAGE_PACKAGE
            || reserved_blueprint_id.blueprint_name != PACKAGE_BLUEPRINT
        {
            return Err(RuntimeError::SystemError(SystemError::CannotGlobalize(
                CannotGlobalizeError::InvalidBlueprintId,
            )));
        }
        PackageAddress::new_or_panic(global_address.into())
    } else {
        PackageAddress::new_or_panic(
            api.kernel_allocate_node_id(EntityType::GlobalPackage)?
                .into(),
        )
    };

    api.kernel_create_node(package_address.into_node_id(), partitions)?;

    if let Some(access_rules) = access_rules {
        let module_base_partition = ObjectModuleId::AccessRules.base_partition_num();
        for offset in 0u8..2u8 {
            let src = MAIN_BASE_PARTITION
                .at_offset(PartitionOffset(offset))
                .unwrap();
            let dest = module_base_partition
                .at_offset(PartitionOffset(offset))
                .unwrap();

            api.kernel_move_module(
                access_rules.0.as_node_id(),
                src,
                package_address.as_node_id(),
                dest,
            )?;
        }

        api.kernel_drop_node(access_rules.0.as_node_id())?;
    }

    Ok(package_address)
}

pub struct PackageNativePackage;

impl PackageNativePackage {
    pub fn definition() -> PackageDefinition {
        let mut aggregator = TypeAggregator::<ScryptoCustomTypeKind>::new();

        let mut fields = Vec::new();
        fields.push(FieldSchema::static_field(
            aggregator.add_child_type_and_descendents::<PackageRoyaltyAccumulatorSubstate>(),
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
                key: TypeRef::Static(aggregator.add_child_type_and_descendents::<Hash>()),
                value: TypeRef::Static(
                    aggregator.add_child_type_and_descendents::<PackageCodeSubstate>(),
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
                    aggregator.add_child_type_and_descendents::<RoyaltyConfig>(),
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
                receiver: Some(schema::ReceiverInfo::normal_ref_mut()),
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

                royalty_config: RoyaltyConfig::default(),
                auth_config: AuthConfig {
                    function_auth: btreemap!(
                        PACKAGE_PUBLISH_WASM_IDENT.to_string() => rule!(allow_all),
                        PACKAGE_PUBLISH_WASM_ADVANCED_IDENT.to_string() => rule!(allow_all),
                        PACKAGE_PUBLISH_NATIVE_IDENT.to_string() => rule!(require(SYSTEM_TRANSACTION_BADGE)),
                    ),
                    method_auth: MethodAuthTemplate::Static(
                        method_auth_template! {
                            PACKAGE_CLAIM_ROYALTIES_IDENT => [OWNER_ROLE];
                        },
                    ),
                },
            }
        );

        PackageDefinition { blueprints }
    }

    #[trace_resources(log=export_name)]
    pub fn invoke_export<Y, L: Default>(
        export_name: &str,
        input: &IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi<L> + ClientApi<RuntimeError>,
    {
        match export_name {
            PACKAGE_PUBLISH_NATIVE_IDENT => {
                api.consume_cost_units(FIXED_HIGH_FEE, ClientCostingReason::RunNative)?;

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
                api.consume_cost_units(FIXED_HIGH_FEE, ClientCostingReason::RunNative)?;

                let input: PackagePublishWasmInput = input.as_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;

                let rtn = Self::publish_wasm(input.code, input.setup, input.metadata, api)?;

                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            PACKAGE_PUBLISH_WASM_ADVANCED_IDENT => {
                api.consume_cost_units(FIXED_HIGH_FEE, ClientCostingReason::RunNative)?;

                let input: PackagePublishWasmAdvancedInput = input.as_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;

                let rtn = Self::publish_wasm_advanced(
                    input.package_address,
                    input.code,
                    input.setup,
                    input.metadata,
                    input.owner_rule,
                    api,
                )?;

                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            PACKAGE_CLAIM_ROYALTIES_IDENT => {
                api.consume_cost_units(FIXED_MEDIUM_FEE, ClientCostingReason::RunNative)?;
                let _input: PackageClaimRoyaltiesInput = input.as_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;
                let rtn = PackageRoyaltyNativeBlueprint::claim_royalty(api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            _ => Err(RuntimeError::ApplicationError(
                ApplicationError::ExportDoesNotExist(export_name.to_string()),
            )),
        }
    }

    pub(crate) fn publish_native<Y, L: Default>(
        package_address: Option<GlobalAddressReservation>,
        native_package_code_id: u8,
        setup: PackageDefinition,
        metadata: BTreeMap<String, MetadataValue>,
        api: &mut Y,
    ) -> Result<PackageAddress, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi<L> + ClientApi<RuntimeError>,
    {
        // Validate schema
        validate_package_schema(setup.blueprints.values().map(|s| &s.schema))
            .map_err(|e| RuntimeError::ApplicationError(ApplicationError::PackageError(e)))?;
        validate_package_event_schema(setup.blueprints.values())
            .map_err(|e| RuntimeError::ApplicationError(ApplicationError::PackageError(e)))?;

        // Build node init
        let mut auth_configs = BTreeMap::new();
        let mut schemas = BTreeMap::new();
        let mut blueprints = BTreeMap::new();
        let mut blueprint_dependencies = BTreeMap::new();

        let code = PackageCodeSubstate {
            vm_type: VmType::Native,
            code: vec![native_package_code_id],
        };

        let code_hash = hash(scrypto_encode(&code).unwrap());

        {
            for (blueprint, definition_init) in setup.blueprints {
                match (
                    &definition_init.blueprint_type,
                    &definition_init.auth_config.method_auth,
                ) {
                    (_, MethodAuthTemplate::Static(..)) => {}
                    (_, MethodAuthTemplate::NoAuth) => {}
                    (BlueprintType::Inner { .. }, MethodAuthTemplate::StaticUseOuterAuth(..)) => {}
                    _ => {
                        return Err(RuntimeError::ApplicationError(
                            ApplicationError::PackageError(PackageError::InvalidAuthSetup),
                        ));
                    }
                }

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

                let (feature_set, outer_blueprint) = match definition_init.blueprint_type {
                    BlueprintType::Outer { feature_set } => (feature_set, None),
                    BlueprintType::Inner { outer_blueprint } => {
                        (BTreeSet::new(), Some(outer_blueprint))
                    }
                };

                let definition = BlueprintDefinition {
                    interface: BlueprintInterface {
                        outer_blueprint,
                        generics: definition_init.schema.generics,
                        feature_set,
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
                blueprints.insert(blueprint.clone(), definition);

                let minor_version_config = BlueprintDependencies {
                    dependencies: definition_init.dependencies,
                };
                blueprint_dependencies.insert(blueprint.clone(), minor_version_config);
            }
        };

        globalize_package(
            package_address,
            blueprints,
            blueprint_dependencies,
            schemas,
            code,
            code_hash,
            btreemap!(),
            auth_configs,
            metadata,
            None,
            api,
        )
    }

    pub(crate) fn publish_wasm<Y, L: Default>(
        code: Vec<u8>,
        definition: PackageDefinition,
        metadata: BTreeMap<String, MetadataValue>,
        api: &mut Y,
    ) -> Result<(PackageAddress, Bucket), RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi<L> + ClientApi<RuntimeError>,
    {
        let (access_rules, bucket) = SecurifiedPackage::create_securified(api)?;
        let address =
            Self::publish_wasm_internal(None, code, definition, metadata, access_rules, api)?;

        Ok((address, bucket))
    }

    pub(crate) fn publish_wasm_advanced<Y, L: Default>(
        package_address: Option<GlobalAddressReservation>,
        code: Vec<u8>,
        definition: PackageDefinition,
        metadata: BTreeMap<String, MetadataValue>,
        owner_rule: OwnerRole,
        api: &mut Y,
    ) -> Result<PackageAddress, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi<L> + ClientApi<RuntimeError>,
    {
        let access_rules = SecurifiedPackage::create_advanced(owner_rule, api)?;
        let address = Self::publish_wasm_internal(
            package_address,
            code,
            definition,
            metadata,
            access_rules,
            api,
        )?;

        Ok(address)
    }

    fn publish_wasm_internal<Y, L: Default>(
        package_address: Option<GlobalAddressReservation>,
        code: Vec<u8>,
        setup: PackageDefinition,
        metadata: BTreeMap<String, MetadataValue>,
        access_rules: AccessRules,
        api: &mut Y,
    ) -> Result<PackageAddress, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi<L> + ClientApi<RuntimeError>,
    {
        // Validate schema
        validate_package_schema(setup.blueprints.values().map(|s| &s.schema))
            .map_err(|e| RuntimeError::ApplicationError(ApplicationError::PackageError(e)))?;
        validate_package_event_schema(setup.blueprints.values())
            .map_err(|e| RuntimeError::ApplicationError(ApplicationError::PackageError(e)))?;

        for BlueprintDefinitionInit {
            blueprint_type,
            schema:
                BlueprintSchemaInit {
                    generics,
                    state: BlueprintStateSchemaInit { collections, .. },
                    functions,
                    ..
                },
            ..
        } in setup.blueprints.values()
        {
            match blueprint_type {
                BlueprintType::Outer { feature_set } => {
                    if !feature_set.is_empty() {
                        return Err(RuntimeError::ApplicationError(
                            ApplicationError::PackageError(PackageError::WasmUnsupported(
                                "Feature set not supported".to_string(),
                            )),
                        ));
                    }
                }
                BlueprintType::Inner { .. } => {
                    return Err(RuntimeError::ApplicationError(
                        ApplicationError::PackageError(PackageError::WasmUnsupported(
                            "Inner blueprints not supported".to_string(),
                        )),
                    ));
                }
            }

            if !collections.is_empty() {
                return Err(RuntimeError::ApplicationError(
                    ApplicationError::PackageError(PackageError::WasmUnsupported(
                        "Static collections not supported".to_string(),
                    )),
                ));
            }

            if !functions.virtual_lazy_load_functions.is_empty() {
                return Err(RuntimeError::ApplicationError(
                    ApplicationError::PackageError(PackageError::WasmUnsupported(
                        "Lazy load functions not supported".to_string(),
                    )),
                ));
            }

            for (_name, schema) in &functions.functions {
                if let Some(info) = &schema.receiver {
                    if info.ref_types != RefTypes::NORMAL {
                        return Err(RuntimeError::ApplicationError(
                            ApplicationError::PackageError(PackageError::WasmUnsupported(
                                "Irregular ref types not supported".to_string(),
                            )),
                        ));
                    }
                }
            }

            if !generics.is_empty() {
                return Err(RuntimeError::ApplicationError(
                    ApplicationError::PackageError(PackageError::WasmUnsupported(
                        "Generics not supported".to_string(),
                    )),
                ));
            }
        }

        // Validate WASM
        WasmValidator::default()
            .validate(&code, setup.blueprints.values())
            .map_err(|e| {
                RuntimeError::ApplicationError(ApplicationError::PackageError(
                    PackageError::InvalidWasm(e),
                ))
            })?;

        let code = PackageCodeSubstate {
            vm_type: VmType::ScryptoV1,
            code,
        };

        let code_hash = hash(scrypto_encode(&code).unwrap());

        let mut auth_templates = BTreeMap::new();

        let mut blueprints = BTreeMap::new();
        let mut schemas = BTreeMap::new();
        let mut royalties = BTreeMap::new();
        let mut blueprint_dependencies = BTreeMap::new();

        // Build node init
        {
            for (blueprint, definition_init) in setup.blueprints {
                match (
                    &definition_init.blueprint_type,
                    &definition_init.auth_config.method_auth,
                ) {
                    (_, MethodAuthTemplate::Static(..)) => {}
                    (_, MethodAuthTemplate::NoAuth) => {}
                    (BlueprintType::Inner { .. }, MethodAuthTemplate::StaticUseOuterAuth(..)) => {}
                    _ => {
                        return Err(RuntimeError::ApplicationError(
                            ApplicationError::PackageError(PackageError::InvalidAuthSetup),
                        ));
                    }
                }
                auth_templates.insert(blueprint.clone(), definition_init.auth_config);

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

                let (feature_set, outer_blueprint) = match definition_init.blueprint_type {
                    BlueprintType::Outer { feature_set } => (feature_set, None),
                    BlueprintType::Inner { outer_blueprint } => {
                        (BTreeSet::new(), Some(outer_blueprint))
                    }
                };

                let definition = BlueprintDefinition {
                    interface: BlueprintInterface {
                        outer_blueprint,
                        generics: definition_init.schema.generics,
                        feature_set: feature_set,
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
                blueprints.insert(blueprint.clone(), definition);
                royalties.insert(blueprint.clone(), definition_init.royalty_config);

                let dependencies = BlueprintDependencies {
                    dependencies: definition_init.dependencies,
                };
                blueprint_dependencies.insert(blueprint.clone(), dependencies);
            }
        }

        globalize_package(
            package_address,
            blueprints,
            blueprint_dependencies,
            schemas,
            code,
            code_hash,
            royalties,
            auth_templates,
            metadata,
            Some(access_rules),
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
        let handle = api.kernel_lock_substate_with_default(
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

        let substate: KeyValueEntrySubstate<RoyaltyConfig> =
            api.kernel_read_substate(handle)?.as_typed().unwrap();
        api.kernel_drop_lock(handle)?;

        let royalty_charge = substate
            .value
            .and_then(|c| c.rules.get(ident).cloned())
            .unwrap_or(RoyaltyAmount::Free);

        if royalty_charge.is_non_zero() {
            let handle = api.kernel_lock_substate(
                receiver,
                MAIN_BASE_PARTITION,
                &PackageField::Royalty.into(),
                LockFlags::MUTABLE,
                SystemLockData::default(),
            )?;

            let mut substate: PackageRoyaltyAccumulatorSubstate =
                api.kernel_read_substate(handle)?.as_typed().unwrap();

            let vault_id = if let Some(vault) = substate.royalty_vault {
                vault
            } else {
                let mut system = SystemService::new(api);
                let new_vault = ResourceManager(RADIX_TOKEN).new_empty_vault(&mut system)?;
                substate.royalty_vault = Some(new_vault);
                api.kernel_write_substate(handle, IndexedScryptoValue::from_typed(&substate))?;
                new_vault
            };
            let package_address = PackageAddress::new_or_panic(receiver.0);
            apply_royalty_cost(
                api,
                royalty_charge,
                RoyaltyRecipient::Package(package_address),
                vault_id.0,
            )?;

            api.kernel_drop_lock(handle)?;
        }

        Ok(())
    }

    pub(crate) fn claim_royalty<Y>(api: &mut Y) -> Result<Bucket, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let handle = api.actor_lock_field(
            OBJECT_HANDLE_SELF,
            PackageField::Royalty.into(),
            LockFlags::read_only(),
        )?;

        let substate: PackageRoyaltyAccumulatorSubstate = api.field_lock_read_typed(handle)?;
        let bucket = match substate.royalty_vault.clone() {
            Some(vault) => Vault(vault).take_all(api)?,
            None => ResourceManager(RADIX_TOKEN).new_empty_bucket(api)?,
        };

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
        let auth_template = Self::get_bp_auth_template(receiver, bp_version_key, api)?;
        let access_rule = auth_template.function_auth.get(ident);
        if let Some(access_rule) = access_rule {
            Ok(ResolvedPermission::AccessRule(access_rule.clone()))
        } else {
            let package_address = PackageAddress::new_or_panic(receiver.0.clone());
            let blueprint_id = BlueprintId::new(&package_address, &bp_version_key.blueprint);
            Err(RuntimeError::SystemModuleError(
                SystemModuleError::AuthError(AuthError::NoFunction(FnIdentifier {
                    blueprint_id,
                    ident: FnIdent::Application(ident.to_string()),
                })),
            ))
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

        let handle = api.kernel_lock_substate_with_default(
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
        api.kernel_drop_lock(handle)?;

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
