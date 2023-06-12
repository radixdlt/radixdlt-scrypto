use crate::blueprints::util::{SecurifiedAccessRules, SecurifiedRoleEntry};
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
use radix_engine_interface::api::node_modules::metadata::{
    METADATA_GET_IDENT, METADATA_REMOVE_IDENT, METADATA_SET_IDENT,
};
use radix_engine_interface::api::{ClientApi, LockFlags, ObjectModuleId, OBJECT_HANDLE_SELF};
pub use radix_engine_interface::blueprints::package::*;
use radix_engine_interface::blueprints::resource::{require, Bucket};
use radix_engine_interface::schema::{
    BlueprintCollectionSchema, BlueprintEventSchemaInit, BlueprintFunctionsTemplateInit,
    BlueprintKeyValueStoreSchema, BlueprintSchemaInit, BlueprintStateSchemaInit, FieldSchema,
    FunctionTemplateInit, RefTypes, TypeRef,
};
use resources_tracker_macro::trace_resources;
use sbor::LocalTypeIndex;

// Import and re-export substate types
use crate::method_auth_template;
use crate::system::system::{KeyValueEntrySubstate, SystemService};
use crate::system::system_callback::{SystemConfig, SystemLockData};
use crate::system::system_callback_api::SystemCallbackObject;
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
    WasmUnsupported(String),

    InvalidMetadataKey(String),
}

fn validate_package_schema<'a, I: Iterator<Item = &'a BlueprintDefinitionInit>>(
    blueprints: I,
) -> Result<(), PackageError> {
    for bp_init in blueprints {
        validate_schema(&bp_init.schema.schema)
            .map_err(|e| PackageError::InvalidBlueprintWasm(e))?;

        if bp_init.schema.state.fields.len() > 0xff {
            return Err(PackageError::TooManySubstateSchemas);
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

    fn role_definitions() -> BTreeMap<RoleKey, SecurifiedRoleEntry> {
        btreemap!()
    }
}

fn globalize_package<Y, L: Default>(
    package_address_reservation: Option<GlobalAddressReservation>,
    blueprints: BTreeMap<String, BlueprintDefinition>,
    blueprint_config: BTreeMap<String, BlueprintDependencies>,

    schemas: BTreeMap<Hash, ScryptoSchema>,
    code: PackageCodeSubstate,
    code_hash: Hash,

    package_royalties: BTreeMap<String, RoyaltyConfig>,

    blueprint_auth_template: BTreeMap<String, AuthTemplate>,

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
        let minor_version_configs = blueprint_config
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
        let auth_partition = blueprint_auth_template
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

            outer_object: None,
            instance_schema: None,
            features: btreeset!(),
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
        /*
        for (partition, substates) in node_substates {
            // TODO: Cleanup
            let offset = partition.0 - MAIN_BASE_PARTITION.0;
            let partition_num = ACCESS_RULES_BASE_PARTITION.at_offset(PartitionOffset(offset)).unwrap();
            partitions.insert(partition_num, substates);
        }
         */
    }

    Ok(package_address)
}

pub struct PackageNativePackage;

impl PackageNativePackage {
    pub fn definition() -> PackageSetup {
        let mut aggregator = TypeAggregator::<ScryptoCustomTypeKind>::new();

        let mut fields = Vec::new();
        fields.push(FieldSchema::normal(
            aggregator.add_child_type_and_descendents::<PackageRoyaltyAccumulatorSubstate>(),
        ));

        let mut collections = Vec::new();
        collections.push(BlueprintCollectionSchema::KeyValueStore(
            BlueprintKeyValueStoreSchema {
                key: TypeRef::Blueprint(
                    aggregator.add_child_type_and_descendents::<BlueprintVersionKey>(),
                ),
                value: TypeRef::Blueprint(
                    aggregator.add_child_type_and_descendents::<BlueprintDefinition>(),
                ),
                can_own: false,
            },
        ));
        collections.push(BlueprintCollectionSchema::KeyValueStore(
            BlueprintKeyValueStoreSchema {
                key: TypeRef::Blueprint(
                    aggregator.add_child_type_and_descendents::<BlueprintVersionKey>(),
                ),
                value: TypeRef::Blueprint(
                    aggregator.add_child_type_and_descendents::<BlueprintDependencies>(),
                ),
                can_own: false,
            },
        ));
        collections.push(BlueprintCollectionSchema::KeyValueStore(
            BlueprintKeyValueStoreSchema {
                key: TypeRef::Blueprint(aggregator.add_child_type_and_descendents::<Hash>()),
                value: TypeRef::Blueprint(
                    aggregator.add_child_type_and_descendents::<ScryptoSchema>(),
                ),
                can_own: false,
            },
        ));
        collections.push(BlueprintCollectionSchema::KeyValueStore(
            BlueprintKeyValueStoreSchema {
                key: TypeRef::Blueprint(aggregator.add_child_type_and_descendents::<Hash>()),
                value: TypeRef::Blueprint(
                    aggregator.add_child_type_and_descendents::<PackageCodeSubstate>(),
                ),
                can_own: false,
            },
        ));
        collections.push(BlueprintCollectionSchema::KeyValueStore(
            BlueprintKeyValueStoreSchema {
                key: TypeRef::Blueprint(
                    aggregator.add_child_type_and_descendents::<BlueprintVersionKey>(),
                ),
                value: TypeRef::Blueprint(
                    aggregator.add_child_type_and_descendents::<RoyaltyConfig>(),
                ),
                can_own: false,
            },
        ));
        collections.push(BlueprintCollectionSchema::KeyValueStore(
            BlueprintKeyValueStoreSchema {
                key: TypeRef::Blueprint(
                    aggregator.add_child_type_and_descendents::<BlueprintVersionKey>(),
                ),
                value: TypeRef::Blueprint(
                    aggregator.add_child_type_and_descendents::<AuthTemplate>(),
                ),
                can_own: false,
            },
        ));

        let mut functions = BTreeMap::new();
        functions.insert(
            PACKAGE_PUBLISH_WASM_IDENT.to_string(),
            FunctionTemplateInit {
                receiver: None,
                input: aggregator.add_child_type_and_descendents::<PackagePublishWasmInput>(),
                output: aggregator.add_child_type_and_descendents::<PackagePublishWasmOutput>(),
                export: PACKAGE_PUBLISH_WASM_IDENT.to_string(),
            },
        );
        functions.insert(
            PACKAGE_PUBLISH_WASM_ADVANCED_IDENT.to_string(),
            FunctionTemplateInit {
                receiver: None,
                input: aggregator
                    .add_child_type_and_descendents::<PackagePublishWasmAdvancedInput>(),
                output: aggregator
                    .add_child_type_and_descendents::<PackagePublishWasmAdvancedOutput>(),
                export: PACKAGE_PUBLISH_WASM_ADVANCED_IDENT.to_string(),
            },
        );
        functions.insert(
            PACKAGE_PUBLISH_NATIVE_IDENT.to_string(),
            FunctionTemplateInit {
                receiver: None,
                input: aggregator.add_child_type_and_descendents::<PackagePublishNativeInput>(),
                output: aggregator.add_child_type_and_descendents::<PackagePublishNativeOutput>(),
                export: PACKAGE_PUBLISH_NATIVE_IDENT.to_string(),
            },
        );
        functions.insert(
            PACKAGE_CLAIM_ROYALTIES_IDENT.to_string(),
            FunctionTemplateInit {
                receiver: Some(schema::ReceiverInfo::normal_ref_mut()),
                input: aggregator.add_child_type_and_descendents::<PackageClaimRoyaltiesInput>(),
                output: aggregator.add_child_type_and_descendents::<PackageClaimRoyaltiesOutput>(),
                export: PACKAGE_CLAIM_ROYALTIES_IDENT.to_string(),
            },
        );

        let schema = generate_full_schema(aggregator);
        let blueprints = btreemap!(
            PACKAGE_BLUEPRINT.to_string() => BlueprintDefinitionInit {
                outer_blueprint: None,
                dependencies: btreeset!(
                    PACKAGE_OF_DIRECT_CALLER_VIRTUAL_BADGE.into(),
                    PACKAGE_OWNER_BADGE.into(),
                ),
                feature_set: btreeset!(),

                schema: BlueprintSchemaInit {
                    schema,
                    state: BlueprintStateSchemaInit {
                        fields,
                        collections,
                    },
                    events: BlueprintEventSchemaInit::default(),
                    functions: BlueprintFunctionsTemplateInit {
                        virtual_lazy_load_functions: btreemap!(),
                        functions,
                    },
                },

                royalty_config: RoyaltyConfig::default(),
                auth_template: AuthTemplate {
                    function_auth: btreemap!(
                        PACKAGE_PUBLISH_WASM_IDENT.to_string() => rule!(allow_all),
                        PACKAGE_PUBLISH_WASM_ADVANCED_IDENT.to_string() => rule!(allow_all),
                        PACKAGE_PUBLISH_NATIVE_IDENT.to_string() => rule!(require(SYSTEM_TRANSACTION_BADGE)),
                    ),
                    method_auth:  method_auth_template! {
                        SchemaMethodKey::metadata(METADATA_SET_IDENT) => [OWNER_ROLE];
                        SchemaMethodKey::metadata(METADATA_REMOVE_IDENT) => [OWNER_ROLE];
                        SchemaMethodKey::metadata(METADATA_GET_IDENT) => SchemaMethodPermission::Public;

                        SchemaMethodKey::main(PACKAGE_CLAIM_ROYALTIES_IDENT) => [OWNER_ROLE];
                    },
                    outer_method_auth_template: btreemap!(),
                },
            }
        );

        PackageSetup { blueprints }
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
        setup: PackageSetup,
        metadata: BTreeMap<String, MetadataValue>,
        api: &mut Y,
    ) -> Result<PackageAddress, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi<L> + ClientApi<RuntimeError>,
    {
        // Validate schema
        validate_package_schema(setup.blueprints.values())
            .map_err(|e| RuntimeError::ApplicationError(ApplicationError::PackageError(e)))?;
        validate_package_event_schema(setup.blueprints.values())
            .map_err(|e| RuntimeError::ApplicationError(ApplicationError::PackageError(e)))?;

        // Build node init
        let mut blueprint_auth_templates = BTreeMap::new();
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
                blueprint_auth_templates.insert(blueprint.clone(), definition_init.auth_template);

                let blueprint_schema = definition_init.schema.schema.clone();
                let schema_hash = hash(scrypto_encode(&blueprint_schema).unwrap());
                schemas.insert(schema_hash, blueprint_schema);

                let mut functions = BTreeMap::new();
                let mut function_exports = BTreeMap::new();
                for (function, setup) in definition_init.schema.functions.functions {
                    functions.insert(
                        function.clone(),
                        FunctionSchema {
                            receiver: setup.receiver,
                            input: SchemaPointer::Package(schema_hash, setup.input),
                            output: SchemaPointer::Package(schema_hash, setup.output),
                        },
                    );
                    let export = PackageExport {
                        code_hash,
                        export_name: setup.export.clone(),
                    };
                    function_exports.insert(function, export);
                }

                let events = definition_init
                    .schema
                    .events
                    .event_schema
                    .into_iter()
                    .map(|(key, index)| (key, SchemaPointer::Package(schema_hash, index)))
                    .collect();

                let definition = BlueprintDefinition {
                    outer_blueprint: definition_init.outer_blueprint,
                    features: definition_init.feature_set,
                    functions,
                    events,
                    state_schema: IndexedBlueprintStateSchema::from_schema(
                        schema_hash,
                        definition_init.schema.state,
                    ),
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
            blueprint_auth_templates,
            metadata,
            None,
            api,
        )
    }

    pub(crate) fn publish_wasm<Y, L: Default>(
        code: Vec<u8>,
        definition: PackageSetup,
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
        definition: PackageSetup,
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
        setup: PackageSetup,
        metadata: BTreeMap<String, MetadataValue>,
        access_rules: AccessRules,
        api: &mut Y,
    ) -> Result<PackageAddress, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi<L> + ClientApi<RuntimeError>,
    {
        // Validate schema
        validate_package_schema(setup.blueprints.values())
            .map_err(|e| RuntimeError::ApplicationError(ApplicationError::PackageError(e)))?;
        validate_package_event_schema(setup.blueprints.values())
            .map_err(|e| RuntimeError::ApplicationError(ApplicationError::PackageError(e)))?;

        for BlueprintDefinitionInit {
            outer_blueprint: parent,
            feature_set: features,
            schema:
                BlueprintSchemaInit {
                    state: BlueprintStateSchemaInit { collections, .. },
                    functions,
                    ..
                },
            ..
        } in setup.blueprints.values()
        {
            if parent.is_some() {
                return Err(RuntimeError::ApplicationError(
                    ApplicationError::PackageError(PackageError::InvalidTypeParent),
                ));
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

            if !features.is_empty() {
                return Err(RuntimeError::ApplicationError(
                    ApplicationError::PackageError(PackageError::WasmUnsupported(
                        "Features not supported".to_string(),
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
                auth_templates.insert(blueprint.clone(), definition_init.auth_template);

                let blueprint_schema = definition_init.schema.schema.clone();
                let schema_hash = hash(scrypto_encode(&blueprint_schema).unwrap());
                schemas.insert(schema_hash, blueprint_schema);

                let mut functions = BTreeMap::new();
                let mut function_exports = BTreeMap::new();
                for (function, setup) in definition_init.schema.functions.functions {
                    functions.insert(
                        function.clone(),
                        FunctionSchema {
                            receiver: setup.receiver,
                            input: SchemaPointer::Package(schema_hash, setup.input),
                            output: SchemaPointer::Package(schema_hash, setup.output),
                        },
                    );
                    let export = PackageExport {
                        code_hash,
                        export_name: setup.export.clone(),
                    };
                    function_exports.insert(function, export);
                }

                let events = definition_init
                    .schema
                    .events
                    .event_schema
                    .into_iter()
                    .map(|(key, index)| (key, SchemaPointer::Package(schema_hash, index)))
                    .collect();

                let definition = BlueprintDefinition {
                    outer_blueprint: definition_init.outer_blueprint,
                    features: definition_init.feature_set,
                    functions,
                    events,
                    state_schema: IndexedBlueprintStateSchema::from_schema(
                        schema_hash,
                        definition_init.schema.state,
                    ),
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

    pub fn get_schema_hash<Y>(
        receiver: &NodeId,
        hash: &Hash,
        api: &mut Y,
    ) -> Result<ScryptoSchema, RuntimeError>
    where
        Y: KernelSubstateApi<SystemLockData>,
    {
        let handle = api.kernel_lock_substate_with_default(
            receiver,
            MAIN_BASE_PARTITION
                .at_offset(PACKAGE_SCHEMAS_PARTITION_OFFSET)
                .unwrap(),
            &SubstateKey::Map(scrypto_encode(hash).unwrap()),
            LockFlags::read_only(),
            Some(|| {
                let kv_entry = KeyValueEntrySubstate::<()>::default();
                IndexedScryptoValue::from_typed(&kv_entry)
            }),
            SystemLockData::default(),
        )?;

        let substate: KeyValueEntrySubstate<ScryptoSchema> =
            api.kernel_read_substate(handle)?.as_typed().unwrap();
        api.kernel_drop_lock(handle)?;

        let schema = substate.value.unwrap();

        Ok(schema)
    }

    pub fn get_blueprint_definition<Y>(
        receiver: &NodeId,
        bp_version_key: &BlueprintVersionKey,
        api: &mut Y,
    ) -> Result<BlueprintDefinition, RuntimeError>
    where
        Y: KernelSubstateApi<SystemLockData>,
    {
        let handle = api.kernel_lock_substate_with_default(
            receiver,
            MAIN_BASE_PARTITION
                .at_offset(PACKAGE_BLUEPRINTS_PARTITION_OFFSET)
                .unwrap(),
            &SubstateKey::Map(scrypto_encode(bp_version_key).unwrap()),
            LockFlags::read_only(),
            Some(|| {
                let kv_entry = KeyValueEntrySubstate::<()>::default();
                IndexedScryptoValue::from_typed(&kv_entry)
            }),
            SystemLockData::default(),
        )?;

        let substate: KeyValueEntrySubstate<BlueprintDefinition> =
            api.kernel_read_substate(handle)?.as_typed().unwrap();
        api.kernel_drop_lock(handle)?;

        let definition = substate.value.ok_or_else(|| {
            let package_address = PackageAddress::new_or_panic(receiver.0.clone());
            let blueprint_id = BlueprintId::new(&package_address, &bp_version_key.blueprint);
            RuntimeError::SystemError(SystemError::BlueprintDoesNotExist(blueprint_id))
        })?;

        Ok(definition)
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
    pub fn get_bp_auth_template<Y>(
        receiver: &NodeId,
        bp_version_key: &BlueprintVersionKey,
        api: &mut Y,
    ) -> Result<AuthTemplate, RuntimeError>
    where
        Y: KernelSubstateApi<SystemLockData>,
    {
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

        let auth_template: KeyValueEntrySubstate<AuthTemplate> =
            api.kernel_read_substate(handle)?.as_typed().unwrap();
        api.kernel_drop_lock(handle)?;

        let template = auth_template.value.ok_or_else(|| {
            let package_address = PackageAddress::new_or_panic(receiver.0.clone());
            let blueprint_id = BlueprintId::new(&package_address, &bp_version_key.blueprint);
            RuntimeError::SystemError(SystemError::BlueprintTemplateDoesNotExist(blueprint_id))
        })?;

        Ok(template)
    }
}
