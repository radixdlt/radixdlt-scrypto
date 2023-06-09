use crate::blueprints::util::{SecurifiedAccessRules, SecurifiedRoleEntry};
use crate::errors::*;
use crate::kernel::kernel_api::{KernelNodeApi, KernelSubstateApi};
use crate::system::node_init::type_info_partition;
use crate::system::node_modules::type_info::TypeInfoSubstate;
use crate::system::system_modules::costing::{FIXED_HIGH_FEE, FIXED_MEDIUM_FEE};
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
    BlueprintCollectionSchema, BlueprintKeyValueStoreSchema, BlueprintSchema, FeaturedSchema,
    FieldSchema, RefTypes, SchemaMethodKey, SchemaMethodPermission, TypeRef,
};
use resources_tracker_macro::trace_resources;
use sbor::LocalTypeIndex;

// Import and re-export substate types
pub use super::substates::PackageCodeTypeSubstate;
use crate::method_auth_template;
use crate::system::system::{SubstateMutability, SubstateWrapper};
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

fn validate_package_schema<'a, I: Iterator<Item = &'a BlueprintSetup>>(
    blueprints: I,
) -> Result<(), PackageError> {
    for setup in blueprints {
        validate_schema(&setup.schema).map_err(|e| PackageError::InvalidBlueprintWasm(e))?;

        if setup.blueprint.fields.len() > 0xff {
            return Err(PackageError::TooManySubstateSchemas);
        }
    }
    Ok(())
}

fn validate_package_event_schema<'a, I: Iterator<Item = &'a BlueprintSetup>>(
    blueprints: I,
) -> Result<(), PackageError> {
    for BlueprintSetup {
        schema,
        event_schema,
        ..
    } in blueprints
    {
        // Package schema validation happens when the package is published. No need to redo
        // it here again.

        for (expected_event_name, local_type_index) in event_schema.iter() {
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
    blueprint_config: BTreeMap<String, BlueprintImpl>,
    code_type: PackageCodeTypeSubstate,
    code: PackageCodeSubstate,
    royalty: PackageRoyaltyAccumulatorSubstate,
    fn_royalty: BTreeMap<FnKey, RoyaltyAmount>,
    function_access_rules: BTreeMap<FnKey, AccessRule>,
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

    // Prepare node init.
    {
        let main_partition = btreemap!(
            PackageField::CodeType.into() => IndexedScryptoValue::from_typed(&code_type),
            PackageField::Code.into() => IndexedScryptoValue::from_typed(&code),
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
                let value = SubstateWrapper {
                    value: Some(definition),
                    mutability: SubstateMutability::Immutable,
                };
                (
                    SubstateKey::Map(scrypto_encode(&blueprint).unwrap()),
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
                let key = BlueprintMinorVersionConfigKey {
                    blueprint,
                    version: BlueprintMinorVersion::default(),
                };

                let value = SubstateWrapper {
                    value: Some(minor_version_config),
                    mutability: SubstateMutability::Immutable,
                };

                (
                    SubstateKey::Map(scrypto_encode(&key).unwrap()),
                    IndexedScryptoValue::from_typed(&value),
                )
            })
            .collect();

        partitions.insert(
            MAIN_BASE_PARTITION
                .at_offset(PACKAGE_BLUEPRINT_MINOR_VERSION_CONFIG_OFFSET)
                .unwrap(),
            minor_version_configs,
        );
    };

    {
        let fn_royalty_partition = fn_royalty
            .into_iter()
            .map(|(k, royalty)| {
                let value = SubstateWrapper {
                    value: Some(royalty),
                    mutability: SubstateMutability::Mutable,
                };
                (
                    SubstateKey::Map(scrypto_encode(&k).unwrap()),
                    IndexedScryptoValue::from_typed(&value),
                )
            })
            .collect();

        partitions.insert(
            MAIN_BASE_PARTITION
                .at_offset(PACKAGE_ROYALTY_PARTITION_OFFSET)
                .unwrap(),
            fn_royalty_partition,
        );
    };

    {
        let function_access_rules_partition = function_access_rules
            .into_iter()
            .map(|(k, rule)| {
                let value = SubstateWrapper {
                    value: Some(rule),
                    mutability: SubstateMutability::Immutable,
                };
                (
                    SubstateKey::Map(scrypto_encode(&k).unwrap()),
                    IndexedScryptoValue::from_typed(&value),
                )
            })
            .collect();

        partitions.insert(
            MAIN_BASE_PARTITION
                .at_offset(PACKAGE_FUNCTION_ACCESS_RULES_PARTITION_OFFSET)
                .unwrap(),
            function_access_rules_partition,
        );
    }

    partitions.insert(
        TYPE_INFO_FIELD_PARTITION,
        type_info_partition(TypeInfoSubstate::Object(ObjectInfo {
            blueprint: BlueprintId::new(&PACKAGE_PACKAGE, PACKAGE_BLUEPRINT),
            global: true,
            outer_object: None,
            instance_schema: None,
            features: btreeset!(),
        })),
    );
    let metadata_partition = {
        let mut metadata_partition = BTreeMap::new();
        for (key, value) in metadata {
            let value = SubstateWrapper {
                value: Some(value),
                mutability: SubstateMutability::Mutable,
            };
            metadata_partition.insert(
                SubstateKey::Map(scrypto_encode(&key).unwrap()),
                IndexedScryptoValue::from_typed(&value),
            );
        }
        metadata_partition
    };
    partitions.insert(METADATA_KV_STORE_PARTITION, metadata_partition);

    let node_id = if let Some(reservation) = package_address_reservation {
        // TODO: Can we use `global_object` API?

        // Check global address reservation
        let global_address = {
            let substates = api.kernel_drop_node(reservation.0.as_node_id())?;

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

        global_address.as_node_id().clone()
    } else {
        api.kernel_allocate_node_id(EntityType::GlobalPackage)?
    };

    api.kernel_create_node(node_id, partitions)?;

    if let Some(access_rules) = access_rules {
        let module_base_partition = ObjectModuleId::AccessRules.base_partition_num();
        for offset in 0u8..2u8 {
            let src = MAIN_BASE_PARTITION
                .at_offset(PartitionOffset(offset))
                .unwrap();
            let dest = module_base_partition
                .at_offset(PartitionOffset(offset))
                .unwrap();

            api.kernel_move_module(access_rules.0.as_node_id(), src, &node_id, dest)?;
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

    let package_address = PackageAddress::new_or_panic(node_id.into());
    Ok(package_address)
}

pub struct PackageNativePackage;

impl PackageNativePackage {
    pub fn definition() -> PackageSetup {
        let mut aggregator = TypeAggregator::<ScryptoCustomTypeKind>::new();

        let mut fields = Vec::new();
        fields.push(FieldSchema::normal(
            aggregator.add_child_type_and_descendents::<PackageCodeTypeSubstate>(),
        ));
        fields.push(FieldSchema::normal(
            aggregator.add_child_type_and_descendents::<PackageCodeSubstate>(),
        ));
        fields.push(FieldSchema::normal(
            aggregator.add_child_type_and_descendents::<PackageRoyaltyAccumulatorSubstate>(),
        ));

        let mut collections = Vec::new();
        collections.push(BlueprintCollectionSchema::KeyValueStore(
            BlueprintKeyValueStoreSchema {
                key: TypeRef::Blueprint(aggregator.add_child_type_and_descendents::<String>()),
                value: TypeRef::Blueprint(
                    aggregator.add_child_type_and_descendents::<BlueprintDefinition>(),
                ),
                can_own: false,
            },
        ));
        collections.push(BlueprintCollectionSchema::KeyValueStore(
            BlueprintKeyValueStoreSchema {
                key: TypeRef::Blueprint(aggregator.add_child_type_and_descendents::<BlueprintMinorVersionConfigKey>()),
                value: TypeRef::Blueprint(
                    aggregator.add_child_type_and_descendents::<BlueprintImpl>(),
                ),
                can_own: false,
            },
        ));
        collections.push(BlueprintCollectionSchema::KeyValueStore(
            BlueprintKeyValueStoreSchema {
                key: TypeRef::Blueprint(aggregator.add_child_type_and_descendents::<FnKey>()),
                value: TypeRef::Blueprint(
                    aggregator.add_child_type_and_descendents::<RoyaltyAmount>(),
                ),
                can_own: false,
            },
        ));
        collections.push(BlueprintCollectionSchema::KeyValueStore(
            BlueprintKeyValueStoreSchema {
                key: TypeRef::Blueprint(aggregator.add_child_type_and_descendents::<FnKey>()),
                value: TypeRef::Blueprint(
                    aggregator.add_child_type_and_descendents::<AccessRule>(),
                ),
                can_own: false,
            },
        ));

        let mut functions = BTreeMap::new();
        functions.insert(
            PACKAGE_PUBLISH_WASM_IDENT.to_string(),
            FunctionSetup {
                receiver: None,
                input: aggregator.add_child_type_and_descendents::<PackagePublishWasmInput>(),
                output: aggregator.add_child_type_and_descendents::<PackagePublishWasmOutput>(),
                export: FeaturedSchema::normal(PACKAGE_PUBLISH_WASM_IDENT),
            },
        );
        functions.insert(
            PACKAGE_PUBLISH_WASM_ADVANCED_IDENT.to_string(),
            FunctionSetup {
                receiver: None,
                input: aggregator
                    .add_child_type_and_descendents::<PackagePublishWasmAdvancedInput>(),
                output: aggregator
                    .add_child_type_and_descendents::<PackagePublishWasmAdvancedOutput>(),
                export: FeaturedSchema::normal(PACKAGE_PUBLISH_WASM_ADVANCED_IDENT),
            },
        );
        functions.insert(
            PACKAGE_PUBLISH_NATIVE_IDENT.to_string(),
            FunctionSetup {
                receiver: None,
                input: aggregator.add_child_type_and_descendents::<PackagePublishNativeInput>(),
                output: aggregator.add_child_type_and_descendents::<PackagePublishNativeOutput>(),
                export: FeaturedSchema::normal(PACKAGE_PUBLISH_NATIVE_IDENT),
            },
        );
        functions.insert(
            PACKAGE_SET_ROYALTY_IDENT.to_string(),
            FunctionSetup {
                receiver: Some(schema::ReceiverInfo::normal_ref_mut()),
                input: aggregator.add_child_type_and_descendents::<PackageSetRoyaltyInput>(),
                output: aggregator.add_child_type_and_descendents::<PackageSetRoyaltyOutput>(),
                export: FeaturedSchema::normal(PACKAGE_SET_ROYALTY_IDENT),
            },
        );
        functions.insert(
            PACKAGE_CLAIM_ROYALTIES_IDENT.to_string(),
            FunctionSetup {
                receiver: Some(schema::ReceiverInfo::normal_ref_mut()),
                input: aggregator.add_child_type_and_descendents::<PackageClaimRoyaltiesInput>(),
                output: aggregator.add_child_type_and_descendents::<PackageClaimRoyaltiesOutput>(),
                export: FeaturedSchema::normal(PACKAGE_CLAIM_ROYALTIES_IDENT),
            },
        );

        let schema = generate_full_schema(aggregator);
        let blueprints = btreemap!(
            PACKAGE_BLUEPRINT.to_string() => BlueprintSetup {
                outer_blueprint: None,
                dependencies: btreeset!(
                    PACKAGE_OF_DIRECT_CALLER_VIRTUAL_BADGE.into(),
                    PACKAGE_OWNER_BADGE.into(),
                ),
                features: btreeset!(),
                schema,
                blueprint: BlueprintSchema {
                    fields,
                    collections,
                },
                event_schema: [].into(),
                function_auth: btreemap!(
                    PACKAGE_PUBLISH_WASM_IDENT.to_string() => rule!(allow_all),
                    PACKAGE_PUBLISH_WASM_ADVANCED_IDENT.to_string() => rule!(allow_all),
                    PACKAGE_PUBLISH_NATIVE_IDENT.to_string() => rule!(require(SYSTEM_TRANSACTION_BADGE)),
                ),
                royalty_config: RoyaltyConfig::default(),
                template: MethodAuthTemplate {
                    method_auth_template:  method_auth_template! {
                        SchemaMethodKey::metadata(METADATA_SET_IDENT) => [OWNER_ROLE];
                        SchemaMethodKey::metadata(METADATA_REMOVE_IDENT) => [OWNER_ROLE];
                        SchemaMethodKey::metadata(METADATA_GET_IDENT) => SchemaMethodPermission::Public;

                        SchemaMethodKey::main(PACKAGE_CLAIM_ROYALTIES_IDENT) => [OWNER_ROLE];
                        SchemaMethodKey::main(PACKAGE_SET_ROYALTY_IDENT) => [OWNER_ROLE];
                    },
                    outer_method_auth_template: btreemap!(),
                },
                virtual_lazy_load_functions: btreemap!(),
                functions,
            }
        );

        PackageSetup { blueprints }
    }

    #[trace_resources(log=export_name)]
    pub fn invoke_export<Y, L: Default>(
        export_name: &str,
        receiver: Option<&NodeId>,
        input: &IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi<L> + ClientApi<RuntimeError>,
    {
        match export_name {
            PACKAGE_PUBLISH_NATIVE_IDENT => {
                api.consume_cost_units(FIXED_HIGH_FEE, ClientCostingReason::RunNative)?;

                if receiver.is_some() {
                    return Err(RuntimeError::SystemUpstreamError(
                        SystemUpstreamError::NativeUnexpectedReceiver(export_name.to_string()),
                    ));
                }

                let input: PackagePublishNativeInput = input.as_typed().map_err(|e| {
                    RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
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

                if receiver.is_some() {
                    return Err(RuntimeError::SystemUpstreamError(
                        SystemUpstreamError::NativeUnexpectedReceiver(export_name.to_string()),
                    ));
                }
                let input: PackagePublishWasmInput = input.as_typed().map_err(|e| {
                    RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
                })?;

                let rtn = Self::publish_wasm(input.code, input.setup, input.metadata, api)?;

                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            PACKAGE_PUBLISH_WASM_ADVANCED_IDENT => {
                api.consume_cost_units(FIXED_HIGH_FEE, ClientCostingReason::RunNative)?;

                if receiver.is_some() {
                    return Err(RuntimeError::SystemUpstreamError(
                        SystemUpstreamError::NativeUnexpectedReceiver(export_name.to_string()),
                    ));
                }
                let input: PackagePublishWasmAdvancedInput = input.as_typed().map_err(|e| {
                    RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
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

            PACKAGE_SET_ROYALTY_IDENT => {
                api.consume_cost_units(FIXED_MEDIUM_FEE, ClientCostingReason::RunNative)?;

                let input: PackageSetRoyaltyInput = input.as_typed().map_err(|e| {
                    RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
                })?;
                let rtn = Self::set_royalty(input.blueprint, input.fn_name, input.royalty, api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            PACKAGE_CLAIM_ROYALTIES_IDENT => {
                api.consume_cost_units(FIXED_MEDIUM_FEE, ClientCostingReason::RunNative)?;
                let _input: PackageClaimRoyaltiesInput = input.as_typed().map_err(|e| {
                    RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
                })?;
                let rtn = Self::claim_royalty(api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            _ => Err(RuntimeError::SystemUpstreamError(
                SystemUpstreamError::NativeExportDoesNotExist(export_name.to_string()),
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
        let (function_access_rules, blueprints, blueprint_config) = {
            let mut access_rules = BTreeMap::new();
            let mut blueprints = BTreeMap::new();
            let mut blueprint_config = BTreeMap::new();

            for (blueprint, setup) in setup.blueprints {
                for (ident, rule) in setup.function_auth {
                    access_rules.insert(FnKey::new(blueprint.clone(), ident), rule);
                }

                let mut functions = BTreeMap::new();
                let mut function_exports = BTreeMap::new();
                for (function, setup) in setup.functions {
                    functions.insert(
                        function.clone(),
                        FunctionSchema {
                            receiver: setup.receiver,
                            input: setup.input,
                            output: setup.output,
                        },
                    );
                    function_exports.insert(function, setup.export);
                }

                let definition = BlueprintDefinition {
                    outer_blueprint: setup.outer_blueprint,
                    features: setup.features,
                    functions,
                    events: setup.event_schema,
                    schema: setup.schema,
                    state_schema: setup.blueprint.into(),
                    template: setup.template,
                };
                blueprints.insert(blueprint.clone(), definition);

                let minor_version_config = BlueprintImpl {
                    dependencies: setup.dependencies,
                    function_exports,
                    virtual_lazy_load_functions: setup.virtual_lazy_load_functions,
                };
                blueprint_config.insert(blueprint.clone(), minor_version_config);
            }

            (access_rules, blueprints, blueprint_config)
        };

        let code_type = PackageCodeTypeSubstate::Native;
        let code = PackageCodeSubstate {
            code: vec![native_package_code_id],
        };
        let royalty = PackageRoyaltyAccumulatorSubstate {
            royalty_vault: None,
        };

        globalize_package(
            package_address,
            blueprints,
            blueprint_config,
            code_type,
            code,
            royalty,
            btreemap!(),
            function_access_rules,
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
        for BlueprintSetup {
            outer_blueprint: parent,
            features,
            blueprint: BlueprintSchema { collections, .. },
            virtual_lazy_load_functions,
            functions,
            ..
        } in setup.blueprints.values()
        {
            if parent.is_some() {
                return Err(RuntimeError::ApplicationError(
                    ApplicationError::PackageError(PackageError::InvalidTypeParent),
                ));
            }

            if !virtual_lazy_load_functions.is_empty() {
                return Err(RuntimeError::ApplicationError(
                    ApplicationError::PackageError(PackageError::WasmUnsupported(
                        "Lazy load functions not supported".to_string(),
                    )),
                ));
            }

            if !collections.is_empty() {
                return Err(RuntimeError::ApplicationError(
                    ApplicationError::PackageError(PackageError::WasmUnsupported(
                        "Static collections not supported".to_string(),
                    )),
                ));
            }

            for (_name, schema) in functions {
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

        // Build node init
        let (function_access_rules, blueprints, blueprint_config, royalty_accumulator, fn_royalty) = {
            let mut access_rules = BTreeMap::new();
            let mut blueprints = BTreeMap::new();
            let mut royalties = BTreeMap::new();
            let mut blueprint_config = BTreeMap::new();

            for (blueprint, setup) in setup.blueprints {
                for (ident, rule) in setup.function_auth {
                    access_rules.insert(FnKey::new(blueprint.clone(), ident), rule);
                }

                let mut functions = BTreeMap::new();
                let mut function_exports = BTreeMap::new();
                for (function, setup) in setup.functions {
                    functions.insert(
                        function.clone(),
                        FunctionSchema {
                            receiver: setup.receiver,
                            input: setup.input,
                            output: setup.output,
                        },
                    );
                    function_exports.insert(function, setup.export);
                }

                let definition = BlueprintDefinition {
                    outer_blueprint: setup.outer_blueprint,
                    features: setup.features,
                    functions,
                    events: setup.event_schema,
                    schema: setup.schema,
                    state_schema: setup.blueprint.into(),
                    template: setup.template,
                };
                blueprints.insert(blueprint.clone(), definition);
                for (ident, amount) in setup.royalty_config.rules {
                    royalties.insert(FnKey::new(blueprint.clone(), ident), amount);
                }

                let minor_version_config = BlueprintImpl {
                    dependencies: setup.dependencies,
                    function_exports,
                    virtual_lazy_load_functions: setup.virtual_lazy_load_functions,
                };
                blueprint_config.insert(blueprint.clone(), minor_version_config);
            }

            (
                access_rules,
                blueprints,
                blueprint_config,
                PackageRoyaltyAccumulatorSubstate {
                    royalty_vault: None,
                },
                royalties,
            )
        };

        let code_type = PackageCodeTypeSubstate::Wasm;
        let code = PackageCodeSubstate { code };

        globalize_package(
            package_address,
            blueprints,
            blueprint_config,
            code_type,
            code,
            royalty_accumulator,
            fn_royalty,
            function_access_rules,
            metadata,
            Some(access_rules),
            api,
        )
    }

    pub(crate) fn set_royalty<Y>(
        blueprint: String,
        fn_name: String,
        amount: RoyaltyAmount,
        api: &mut Y,
    ) -> Result<(), RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        // FIXME: double check if auth is set up for any package

        let handle = api.actor_lock_key_value_entry(
            OBJECT_HANDLE_SELF,
            PACKAGE_ROYALTY_COLLECTION_INDEX,
            &scrypto_encode(&FnKey::new(blueprint, fn_name)).unwrap(),
            LockFlags::MUTABLE,
        )?;
        api.key_value_entry_set_typed(handle, amount)?;
        api.key_value_entry_release(handle)?;

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
