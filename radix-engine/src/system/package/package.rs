use crate::errors::*;
use crate::kernel::kernel_api::{KernelNodeApi, KernelSubstateApi};
use crate::system::kernel_modules::costing::FIXED_HIGH_FEE;
use crate::system::node::RENodeInit;
use crate::system::node::RENodeModuleInit;
use crate::system::node_modules::access_rules::{
    FunctionAccessRulesSubstate, MethodAccessRulesSubstate,
};
use crate::system::node_modules::metadata::MetadataSubstate;
use crate::system::node_modules::type_info::TypeInfoSubstate;
use crate::system::type_info::PackageCodeTypeSubstate;
use crate::types::*;
use crate::wasm::{PrepareError, WasmValidator};
use core::fmt::Debug;
use native_sdk::resource::ResourceManager;
use radix_engine_interface::api::package::*;
use radix_engine_interface::api::unsafe_api::ClientCostingReason;
use radix_engine_interface::api::ClientApi;
use radix_engine_interface::blueprints::resource::{AccessRule, AccessRules};
use radix_engine_interface::schema::PackageSchema;

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum PackageError {
    InvalidWasm(PrepareError),

    InvalidBlueprintWasm(SchemaValidationError),
    MissingSubstateSchema,
    TooManySubstateSchemas,
}

fn validate_package_schema(schema: &PackageSchema) -> Result<(), PackageError> {
    for blueprint in schema.blueprints.values() {
        validate_schema(&blueprint.schema).map_err(|e| PackageError::InvalidBlueprintWasm(e))?;

        if blueprint.substates.is_empty() {
            return Err(PackageError::MissingSubstateSchema);
        } else if blueprint.substates.len() > 0xff {
            return Err(PackageError::TooManySubstateSchemas);
        }
    }
    Ok(())
}

fn build_package_node_modules(
    royalty_vault: Own,
    royalty_config: BTreeMap<String, RoyaltyConfig>,
    metadata: BTreeMap<String, String>,
    access_rules: AccessRules,
) -> BTreeMap<NodeModuleId, RENodeModuleInit> {
    let mut node_modules = BTreeMap::new();
    node_modules.insert(
        NodeModuleId::TypeInfo,
        RENodeModuleInit::TypeInfo(TypeInfoSubstate {
            package_address: PACKAGE_LOADER,
            blueprint_name: PACKAGE_LOADER_BLUEPRINT.to_string(),
            global: true,
        }),
    );
    node_modules.insert(
        NodeModuleId::PackageRoyalty,
        RENodeModuleInit::PackageRoyalty(
            PackageRoyaltyConfigSubstate { royalty_config },
            PackageRoyaltyAccumulatorSubstate { royalty_vault },
        ),
    );
    node_modules.insert(
        NodeModuleId::Metadata,
        RENodeModuleInit::Metadata(MetadataSubstate { metadata: metadata }),
    );
    node_modules.insert(
        NodeModuleId::AccessRules,
        RENodeModuleInit::MethodAccessRules(MethodAccessRulesSubstate {
            access_rules: access_rules,
        }),
    );
    node_modules.insert(
        NodeModuleId::FunctionAccessRules,
        RENodeModuleInit::FunctionAccessRules(FunctionAccessRulesSubstate {
            access_rules: BTreeMap::new(),
            default_auth: AccessRule::AllowAll,
        }),
    );

    node_modules
}

pub struct Package;

impl Package {
    pub fn invoke_export<Y>(
        export_name: &str,
        receiver: Option<RENodeId>,
        input: IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        match export_name {
            PACKAGE_LOADER_PUBLISH_NATIVE_IDENT => {
                api.consume_cost_units(FIXED_HIGH_FEE, ClientCostingReason::RunNative)?;

                if receiver.is_some() {
                    return Err(RuntimeError::InterpreterError(
                        InterpreterError::NativeUnexpectedReceiver(export_name.to_string()),
                    ));
                }

                Self::publish_native(input, api)
            }
            PACKAGE_LOADER_PUBLISH_WASM_IDENT => {
                api.consume_cost_units(FIXED_HIGH_FEE, ClientCostingReason::RunNative)?;

                if receiver.is_some() {
                    return Err(RuntimeError::InterpreterError(
                        InterpreterError::NativeUnexpectedReceiver(export_name.to_string()),
                    ));
                }

                Self::publish_wasm(input, api)
            }
            _ => Err(RuntimeError::InterpreterError(
                InterpreterError::NativeExportDoesNotExist(export_name.to_string()),
            )),
        }
    }

    pub(crate) fn publish_native<Y>(
        input: IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        let input: PackageLoaderPublishNativeInput = input.as_typed().map_err(|e| {
            RuntimeError::InterpreterError(InterpreterError::ScryptoInputDecodeError(e))
        })?;

        // Validate schema
        validate_package_schema(&input.schema)
            .map_err(|e| RuntimeError::ApplicationError(ApplicationError::PackageError(e)))?;

        // Build node init
        let info = PackageInfoSubstate {
            schema: input.schema,
            dependent_resources: input.dependent_resources.into_iter().collect(),
            dependent_components: input.dependent_components.into_iter().collect(),
        };
        let code_type = PackageCodeTypeSubstate::Native;
        let code = PackageCodeSubstate {
            code: vec![input.native_package_code_id],
        };
        let node_init = RENodeInit::GlobalPackage(info, code_type, code);

        // Build node module init
        let node_modules = build_package_node_modules(
            ResourceManager(RADIX_TOKEN).new_vault(api)?,
            BTreeMap::new(),
            input.metadata,
            input.access_rules,
        );

        // Create package node
        let node_id = if let Some(address) = input.package_address {
            RENodeId::GlobalPackage(PackageAddress::Normal(address))
        } else {
            api.kernel_allocate_node_id(RENodeType::GlobalPackage)?
        };
        api.kernel_create_node(node_id, node_init, node_modules)?;

        // Return
        let package_address: PackageAddress = node_id.into();
        Ok(IndexedScryptoValue::from_typed(&package_address))
    }

    pub(crate) fn publish_wasm<Y>(
        input: IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        let input: PackageLoaderPublishWasmInput = input.as_typed().map_err(|e| {
            RuntimeError::InterpreterError(InterpreterError::ScryptoInputDecodeError(e))
        })?;

        // Validate schema
        validate_package_schema(&input.schema)
            .map_err(|e| RuntimeError::ApplicationError(ApplicationError::PackageError(e)))?;

        // Validate WASM
        WasmValidator::default()
            .validate(&input.code, &input.schema)
            .map_err(|e| {
                RuntimeError::ApplicationError(ApplicationError::PackageError(
                    PackageError::InvalidWasm(e),
                ))
            })?;

        // Build node init
        let info = PackageInfoSubstate {
            schema: input.schema,
            dependent_resources: BTreeSet::new(),
            dependent_components: BTreeSet::new(),
        };
        let code_type = PackageCodeTypeSubstate::Wasm;
        let code = PackageCodeSubstate { code: input.code };
        let node_init = RENodeInit::GlobalPackage(info, code_type, code);

        // Build node module init
        let node_modules = build_package_node_modules(
            ResourceManager(RADIX_TOKEN).new_vault(api)?,
            input.royalty_config,
            input.metadata,
            input.access_rules,
        );

        // Create package node
        let node_id = if let Some(address) = input.package_address {
            RENodeId::GlobalPackage(PackageAddress::Normal(address))
        } else {
            api.kernel_allocate_node_id(RENodeType::GlobalPackage)?
        };
        api.kernel_create_node(node_id, node_init, node_modules)?;

        // Return
        let package_address: PackageAddress = node_id.into();
        Ok(IndexedScryptoValue::from_typed(&package_address))
    }
}
