use crate::blueprints::util::{SecurifiedAccessRules, SecurifiedRoleEntry};
use crate::errors::*;
use crate::kernel::kernel_api::KernelNodeApi;
use crate::system::node_init::ModuleInit;
use crate::system::node_modules::access_rules::{
    FunctionAccessRulesSubstate, MethodAccessRulesSubstate,
};
use crate::system::node_modules::type_info::TypeInfoSubstate;
use crate::system::system_modules::costing::{FIXED_HIGH_FEE, FIXED_MEDIUM_FEE};
use crate::track::interface::NodeSubstates;
use crate::types::*;
use crate::vm::wasm::{PrepareError, WasmValidator};
use native_sdk::modules::access_rules::AccessRules;
use native_sdk::resource::NativeVault;
use native_sdk::resource::ResourceManager;
use radix_engine_interface::api::component::{
    ComponentRoyaltyAccumulatorSubstate, ComponentRoyaltyConfigSubstate,
};
use radix_engine_interface::api::node_modules::metadata::MetadataValue;
use radix_engine_interface::api::node_modules::metadata::{
    METADATA_GET_IDENT, METADATA_REMOVE_IDENT, METADATA_SET_IDENT,
};
use radix_engine_interface::api::{ClientApi, LockFlags, OBJECT_HANDLE_SELF};
pub use radix_engine_interface::blueprints::package::*;
use radix_engine_interface::blueprints::resource::{require, Bucket};
use radix_engine_interface::schema::{BlueprintSchema, ExportNameMapping, FunctionSchema, RefTypes, SchemaMethodKey, SchemaMethodPermission};
use resources_tracker_macro::trace_resources;

// Import and re-export substate types
pub use super::substates::PackageCodeTypeSubstate;
use crate::method_auth_template;
pub use crate::system::node_modules::access_rules::FunctionAccessRulesSubstate as PackageFunctionAccessRulesSubstate;
pub use radix_engine_interface::blueprints::package::{
    PackageCodeSubstate, PackageInfoSubstate, PackageRoyaltySubstate,
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

fn validate_package_schema<'a, I: Iterator<Item = &'a BlueprintSchema>>(
    blueprints: I,
) -> Result<(), PackageError> {
    for blueprint in blueprints {
        validate_schema(&blueprint.schema).map_err(|e| PackageError::InvalidBlueprintWasm(e))?;

        if blueprint.fields.len() > 0xff {
            return Err(PackageError::TooManySubstateSchemas);
        }
    }
    Ok(())
}

fn validate_package_event_schema<'a, I: Iterator<Item = &'a BlueprintSchema>>(
    blueprints: I,
) -> Result<(), PackageError> {
    for BlueprintSchema {
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

fn globalize_package<Y>(
    package_address: Option<[u8; NodeId::LENGTH]>,
    info: PackageInfoSubstate,
    code_type: PackageCodeTypeSubstate,
    code: PackageCodeSubstate,
    royalty: PackageRoyaltySubstate,
    function_access_rules: FunctionAccessRulesSubstate,
    metadata: BTreeMap<String, MetadataValue>,
    access_rules: Option<AccessRules>,
    api: &mut Y,
) -> Result<PackageAddress, RuntimeError>
where
    Y: KernelNodeApi + ClientApi<RuntimeError>,
{
    // Use kernel API to commit substates directly.
    // Can't use the ClientApi because of chicken-and-egg issue.

    // Prepare node init.
    let node_init = btreemap!(
        PackageField::Info.into() => IndexedScryptoValue::from_typed(&info),
        PackageField::CodeType.into() => IndexedScryptoValue::from_typed(&code_type),
        PackageField::Code.into() => IndexedScryptoValue::from_typed(&code),
        PackageField::Royalty.into() => IndexedScryptoValue::from_typed(&royalty),
        PackageField::FunctionAccessRules.into() =>IndexedScryptoValue::from_typed(&function_access_rules),
    );

    // Prepare node modules.
    let mut node_modules = BTreeMap::new();
    node_modules.insert(
        TYPE_INFO_FIELD_PARTITION,
        ModuleInit::TypeInfo(TypeInfoSubstate::Object(ObjectInfo {
            blueprint: BlueprintId::new(&PACKAGE_PACKAGE, PACKAGE_BLUEPRINT),
            global: true,
            outer_object: None,
            instance_schema: None,
        })),
    );
    let mut metadata_init = BTreeMap::new();
    for (key, value) in metadata {
        metadata_init.insert(
            SubstateKey::Map(scrypto_encode(&key).unwrap()),
            IndexedScryptoValue::from_typed(&Some(value)),
        );
    }
    node_modules.insert(
        METADATA_KV_STORE_PARTITION,
        ModuleInit::Metadata(metadata_init),
    );
    node_modules.insert(
        ROYALTY_FIELD_PARTITION,
        ModuleInit::Royalty(
            ComponentRoyaltyConfigSubstate {
                royalty_config: RoyaltyConfig::default(),
            },
            ComponentRoyaltyAccumulatorSubstate {
                royalty_vault: None,
            },
        ),
    );

    if let Some(access_rules) = access_rules {
        let mut node_substates = api.kernel_drop_node(access_rules.0.as_node_id())?;
        let access_rules = node_substates
            .remove(&OBJECT_BASE_PARTITION)
            .unwrap()
            .remove(&AccessRulesField::AccessRules.into())
            .unwrap();
        let access_rules: MethodAccessRulesSubstate = access_rules.as_typed().unwrap();
        node_modules.insert(
            ACCESS_RULES_FIELD_PARTITION,
            ModuleInit::AccessRules(access_rules),
        );
    } else {
        node_modules.insert(
            ACCESS_RULES_FIELD_PARTITION,
            ModuleInit::AccessRules(MethodAccessRulesSubstate {
                roles: BTreeMap::new(),
                role_mutability: BTreeMap::new(),
            }),
        );
    }

    let node_id = if let Some(address) = package_address {
        NodeId(address)
    } else {
        api.kernel_allocate_node_id(EntityType::GlobalPackage)?
    };

    let mut modules: NodeSubstates = node_modules
        .into_iter()
        .map(|(k, v)| (k, v.to_substates()))
        .collect();
    modules.insert(OBJECT_BASE_PARTITION, node_init);

    api.kernel_create_node(node_id, modules)?;

    let package_address = PackageAddress::new_or_panic(node_id.into());
    Ok(package_address)
}

pub struct PackageNativePackage;

impl PackageNativePackage {
    pub fn definition() -> PackageSetup {
        let mut aggregator = TypeAggregator::<ScryptoCustomTypeKind>::new();

        let mut fields = Vec::new();
        fields.push(aggregator.add_child_type_and_descendents::<PackageInfoSubstate>());
        fields.push(aggregator.add_child_type_and_descendents::<PackageCodeTypeSubstate>());
        fields.push(aggregator.add_child_type_and_descendents::<PackageCodeSubstate>());
        fields.push(aggregator.add_child_type_and_descendents::<PackageRoyaltySubstate>());
        fields.push(aggregator.add_child_type_and_descendents::<FunctionAccessRulesSubstate>());

        let mut functions = BTreeMap::new();
        functions.insert(
            PACKAGE_PUBLISH_WASM_IDENT.to_string(),
            FunctionSchema {
                receiver: None,
                input: aggregator.add_child_type_and_descendents::<PackagePublishWasmInput>(),
                output: aggregator.add_child_type_and_descendents::<PackagePublishWasmOutput>(),
                export: ExportNameMapping::normal(PACKAGE_PUBLISH_WASM_IDENT),
            },
        );
        functions.insert(
            PACKAGE_PUBLISH_WASM_ADVANCED_IDENT.to_string(),
            FunctionSchema {
                receiver: None,
                input: aggregator
                    .add_child_type_and_descendents::<PackagePublishWasmAdvancedInput>(),
                output: aggregator
                    .add_child_type_and_descendents::<PackagePublishWasmAdvancedOutput>(),
                export: ExportNameMapping::normal(PACKAGE_PUBLISH_WASM_ADVANCED_IDENT),
            },
        );
        functions.insert(
            PACKAGE_PUBLISH_NATIVE_IDENT.to_string(),
            FunctionSchema {
                receiver: None,
                input: aggregator.add_child_type_and_descendents::<PackagePublishNativeInput>(),
                output: aggregator.add_child_type_and_descendents::<PackagePublishNativeOutput>(),
                export: ExportNameMapping::normal(PACKAGE_PUBLISH_NATIVE_IDENT),
            },
        );
        functions.insert(
            PACKAGE_SET_ROYALTY_IDENT.to_string(),
            FunctionSchema {
                receiver: Some(schema::ReceiverInfo::normal_ref_mut()),
                input: aggregator.add_child_type_and_descendents::<PackageSetRoyaltyInput>(),
                output: aggregator.add_child_type_and_descendents::<PackageSetRoyaltyOutput>(),
                export: ExportNameMapping::normal(PACKAGE_SET_ROYALTY_IDENT),
            },
        );
        functions.insert(
            PACKAGE_CLAIM_ROYALTIES_IDENT.to_string(),
            FunctionSchema {
                receiver: Some(schema::ReceiverInfo::normal_ref_mut()),
                input: aggregator.add_child_type_and_descendents::<PackageClaimRoyaltiesInput>(),
                output: aggregator.add_child_type_and_descendents::<PackageClaimRoyaltiesOutput>(),
                export: ExportNameMapping::normal(PACKAGE_CLAIM_ROYALTIES_IDENT),
            },
        );

        let schema = generate_full_schema(aggregator);
        let blueprints = btreemap!(
            PACKAGE_BLUEPRINT.to_string() => BlueprintSetup {
                schema: BlueprintSchema {
                    outer_blueprint: None,
                    schema,
                    fields,
                    collections: vec![],
                    functions,
                    virtual_lazy_load_functions: btreemap!(),
                    event_schema: [].into(),
                    dependencies: btreeset!(
                        PACKAGE_OF_DIRECT_CALLER_VIRTUAL_BADGE.into(),
                        PACKAGE_OWNER_BADGE.into(),
                    ),
                },
                function_auth: btreemap!(
                    PACKAGE_PUBLISH_WASM_IDENT.to_string() => rule!(allow_all),
                    PACKAGE_PUBLISH_WASM_ADVANCED_IDENT.to_string() => rule!(allow_all),
                    PACKAGE_PUBLISH_NATIVE_IDENT.to_string() => rule!(require(SYSTEM_TRANSACTION_BADGE)),
                ),
                royalty_config: RoyaltyConfig::default(),
                template: BlueprintTemplate {
                    method_auth_template:  method_auth_template! {
                        SchemaMethodKey::metadata(METADATA_SET_IDENT) => [OWNER_ROLE];
                        SchemaMethodKey::metadata(METADATA_REMOVE_IDENT) => [OWNER_ROLE];
                        SchemaMethodKey::metadata(METADATA_GET_IDENT) => SchemaMethodPermission::Public;

                        SchemaMethodKey::main(PACKAGE_CLAIM_ROYALTIES_IDENT) => [OWNER_ROLE];
                        SchemaMethodKey::main(PACKAGE_SET_ROYALTY_IDENT) => [OWNER_ROLE];
                    },
                    outer_method_auth_template: btreemap!(),
                }
            }
        );

        PackageSetup { blueprints }
    }

    #[trace_resources(log=export_name)]
    pub fn invoke_export<Y>(
        export_name: &str,
        receiver: Option<&NodeId>,
        input: &IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi + ClientApi<RuntimeError>,
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
                    input.definition,
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
                    input.definition,
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

    pub(crate) fn publish_native<Y>(
        package_address: Option<[u8; NodeId::LENGTH]>, // TODO: Clean this up
        native_package_code_id: u8,
        definition: PackageSetup,
        metadata: BTreeMap<String, MetadataValue>,
        api: &mut Y,
    ) -> Result<PackageAddress, RuntimeError>
    where
        Y: KernelNodeApi + ClientApi<RuntimeError>,
    {
        // Validate schema
        validate_package_schema(definition.blueprints.values().map(|s| &s.schema))
            .map_err(|e| RuntimeError::ApplicationError(ApplicationError::PackageError(e)))?;
        validate_package_event_schema(definition.blueprints.values().map(|s| &s.schema))
            .map_err(|e| RuntimeError::ApplicationError(ApplicationError::PackageError(e)))?;

        // Build node init
        let (function_access_rules, info) = {
            let mut access_rules = BTreeMap::new();
            let mut blueprints = BTreeMap::new();

            for (blueprint, setup) in definition.blueprints {
                for (ident, rule) in setup.function_auth {
                    access_rules.insert(FnKey::new(blueprint.clone(), ident), rule);
                }

                let definition = BlueprintDefinition {
                    schema: setup.schema.into(),
                    template: setup.template,
                };
                blueprints.insert(blueprint.clone(), definition);
            }

            (
                FunctionAccessRulesSubstate { access_rules },
                PackageInfoSubstate {
                    schema: IndexedPackageSchema { blueprints },
                },
            )
        };

        let code_type = PackageCodeTypeSubstate::Native;
        let code = PackageCodeSubstate {
            code: vec![native_package_code_id],
        };
        let royalty = PackageRoyaltySubstate {
            royalty_vault: None,
            blueprint_royalty_configs: BTreeMap::new(),
        };

        globalize_package(
            package_address,
            info,
            code_type,
            code,
            royalty,
            function_access_rules,
            metadata,
            None,
            api,
        )
    }

    pub(crate) fn publish_wasm<Y>(
        code: Vec<u8>,
        definition: PackageSetup,
        metadata: BTreeMap<String, MetadataValue>,
        api: &mut Y,
    ) -> Result<(PackageAddress, Bucket), RuntimeError>
    where
        Y: KernelNodeApi + ClientApi<RuntimeError>,
    {
        let (access_rules, bucket) = SecurifiedPackage::create_securified(api)?;
        let address =
            Self::publish_wasm_internal(None, code, definition, metadata, access_rules, api)?;

        Ok((address, bucket))
    }

    pub(crate) fn publish_wasm_advanced<Y>(
        package_address: Option<[u8; NodeId::LENGTH]>, // TODO: Clean this up
        code: Vec<u8>,
        definition: PackageSetup,
        metadata: BTreeMap<String, MetadataValue>,
        owner_rule: OwnerRole,
        api: &mut Y,
    ) -> Result<PackageAddress, RuntimeError>
    where
        Y: KernelNodeApi + ClientApi<RuntimeError>,
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

    fn publish_wasm_internal<Y>(
        package_address: Option<[u8; NodeId::LENGTH]>, // TODO: Clean this up
        code: Vec<u8>,
        setup: PackageSetup,
        metadata: BTreeMap<String, MetadataValue>,
        access_rules: AccessRules,
        api: &mut Y,
    ) -> Result<PackageAddress, RuntimeError>
    where
        Y: KernelNodeApi + ClientApi<RuntimeError>,
    {
        // Validate schema
        validate_package_schema(setup.blueprints.values().map(|s| &s.schema))
            .map_err(|e| RuntimeError::ApplicationError(ApplicationError::PackageError(e)))?;
        validate_package_event_schema(setup.blueprints.values().map(|s| &s.schema))
            .map_err(|e| RuntimeError::ApplicationError(ApplicationError::PackageError(e)))?;
        for BlueprintSchema {
            collections,
            outer_blueprint: parent,
            virtual_lazy_load_functions,
            functions,
            ..
        } in setup.blueprints.values().map(|s| &s.schema)
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
        }

        // Validate WASM
        WasmValidator::default()
            .validate(&code, setup.blueprints.values().map(|s| &s.schema))
            .map_err(|e| {
                RuntimeError::ApplicationError(ApplicationError::PackageError(
                    PackageError::InvalidWasm(e),
                ))
            })?;

        // Build node init
        let (function_access_rules, info, royalty) = {
            let mut access_rules = BTreeMap::new();
            let mut blueprints = BTreeMap::new();
            let mut royalties = BTreeMap::new();

            for (blueprint, setup) in setup.blueprints {
                for (ident, rule) in setup.function_auth {
                    access_rules.insert(FnKey::new(blueprint.clone(), ident), rule);
                }

                let definition = BlueprintDefinition {
                    schema: setup.schema.into(),
                    template: setup.template,
                };
                blueprints.insert(blueprint.clone(), definition);
                royalties.insert(blueprint.clone(), setup.royalty_config);
            }

            (
                FunctionAccessRulesSubstate { access_rules },
                PackageInfoSubstate {
                    schema: IndexedPackageSchema { blueprints },
                },
                PackageRoyaltySubstate {
                    royalty_vault: None,
                    blueprint_royalty_configs: royalties,
                },
            )
        };

        let code_type = PackageCodeTypeSubstate::Wasm;
        let code = PackageCodeSubstate { code };

        globalize_package(
            package_address,
            info,
            code_type,
            code,
            royalty,
            function_access_rules,
            metadata,
            Some(access_rules),
            api,
        )
    }

    pub(crate) fn set_royalty<Y>(
        blueprint: String,
        fn_name: String,
        royalty: RoyaltyAmount,
        api: &mut Y,
    ) -> Result<(), RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        // FIXME: double check if auth is set up for any package

        let handle = api.actor_lock_field(
            OBJECT_HANDLE_SELF,
            PackageField::Royalty.into(),
            LockFlags::MUTABLE,
        )?;

        let mut substate: PackageRoyaltySubstate = api.field_lock_read_typed(handle)?;
        let royalty_config = substate
            .blueprint_royalty_configs
            .entry(blueprint)
            .or_insert(RoyaltyConfig::default());
        royalty_config.rules.insert(fn_name, royalty);
        api.field_lock_write_typed(handle, &substate)?;
        api.field_lock_release(handle)?;
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

        let substate: PackageRoyaltySubstate = api.field_lock_read_typed(handle)?;
        let bucket = match substate.royalty_vault.clone() {
            Some(vault) => Vault(vault).take_all(api)?,
            None => ResourceManager(RADIX_TOKEN).new_empty_bucket(api)?,
        };

        Ok(bucket)
    }
}
