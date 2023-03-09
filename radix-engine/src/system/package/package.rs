use crate::errors::*;
use crate::kernel::kernel_api::{KernelNodeApi, KernelSubstateApi};
use crate::system::kernel_modules::costing::FIXED_HIGH_FEE;
use crate::system::kernel_modules::events::EventError;
use crate::system::node::RENodeInit;
use crate::system::node::RENodeModuleInit;
use crate::system::node_modules::access_rules::{
    FunctionAccessRulesSubstate, MethodAccessRulesSubstate,
};
use crate::system::node_modules::event_schema::PackageEventSchemaSubstate;
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
use radix_engine_interface::blueprints::resource::AccessRule;

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum PackageError {
    InvalidRequestData(DecodeError),
    InvalidWasm(PrepareError),
    InvalidSchema(DecodeError),
    BlueprintNotFound,
    MethodNotFound(String),
    CouldNotEncodePackageAddress,
}

pub struct Package;
impl Package {
    pub fn invoke_export<Y>(
        export_name: &str,
        receiver: Option<RENodeId>,
        input: ScryptoValue,
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
        input: ScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        let input: PackageLoaderPublishNativeInput =
            scrypto_decode(&scrypto_encode(&input).unwrap()).map_err(|e| {
                RuntimeError::InterpreterError(InterpreterError::ScryptoInputDecodeError(e))
            })?;

        let metadata_substate = MetadataSubstate {
            metadata: input.metadata,
        };
        let access_rules = MethodAccessRulesSubstate {
            access_rules: input.access_rules,
        };

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
            NodeModuleId::Metadata,
            RENodeModuleInit::Metadata(metadata_substate),
        );
        node_modules.insert(
            NodeModuleId::AccessRules,
            RENodeModuleInit::MethodAccessRules(access_rules),
        );
        node_modules.insert(
            NodeModuleId::FunctionAccessRules,
            RENodeModuleInit::FunctionAccessRules(FunctionAccessRulesSubstate {
                access_rules: input.package_access_rules,
                default_auth: input.default_package_access_rule,
            }),
        );
        {
            let mut package_event_schema = BTreeMap::<
                String,
                BTreeMap<String, (LocalTypeIndex, Schema<ScryptoCustomTypeExtension>)>,
            >::new();
            for (blueprint_name, event_schemas) in input.event_schema {
                let blueprint_schema = package_event_schema.entry(blueprint_name).or_default();
                for (local_type_index, schema) in event_schemas {
                    let event_name = {
                        (*schema
                            .resolve_type_metadata(local_type_index)
                            .map_or(
                                Err(RuntimeError::ApplicationError(
                                    ApplicationError::EventError(EventError::InvalidEventSchema),
                                )),
                                Ok,
                            )?
                            .type_name)
                            .to_owned()
                    };
                    blueprint_schema.insert(event_name, (local_type_index, schema));
                }
            }

            node_modules.insert(
                NodeModuleId::PackageEventSchema,
                RENodeModuleInit::PackageEventSchema(PackageEventSchemaSubstate(
                    package_event_schema,
                )),
            );
        }

        let info = PackageInfoSubstate {
            schema: input.schema,
            dependent_resources: input.dependent_resources.into_iter().collect(),
            dependent_components: input.dependent_components.into_iter().collect(),
        };
        let code_type = PackageCodeTypeSubstate::Native;
        let code = PackageCodeSubstate {
            code: vec![input.native_package_code_id],
        };

        // Create package node
        // Globalize
        let node_id = if let Some(address) = input.package_address {
            RENodeId::GlobalObject(PackageAddress::Normal(address).into())
        } else {
            api.kernel_allocate_node_id(RENodeType::GlobalPackage)?
        };

        api.kernel_create_node(
            node_id,
            RENodeInit::GlobalPackage(info, code_type, code),
            node_modules,
        )?;

        let package_address: PackageAddress = node_id.into();
        Ok(IndexedScryptoValue::from_typed(&package_address))
    }

    pub(crate) fn publish_wasm<Y>(
        input: ScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        let input: PackageLoaderPublishWasmInput = scrypto_decode(&scrypto_encode(&input).unwrap())
            .map_err(|e| {
                RuntimeError::InterpreterError(InterpreterError::ScryptoInputDecodeError(e))
            })?;

        let royalty_vault_id = ResourceManager(RADIX_TOKEN).new_vault(api)?.vault_id();

        WasmValidator::default()
            .validate(&input.code, &input.schema)
            .map_err(|e| {
                RuntimeError::ApplicationError(ApplicationError::PackageError(
                    PackageError::InvalidWasm(e),
                ))
            })?;

        // FIXME: schema - validate schema consistency!

        let code_type_substate = PackageCodeTypeSubstate::Wasm;
        let wasm_code_substate = PackageCodeSubstate { code: input.code };
        let package_info_substate = PackageInfoSubstate {
            schema: input.schema,
            dependent_resources: BTreeSet::new(),
            dependent_components: BTreeSet::new(),
        };
        let package_royalty_config = PackageRoyaltyConfigSubstate {
            royalty_config: input.royalty_config,
        };
        let package_royalty_accumulator = PackageRoyaltyAccumulatorSubstate {
            royalty: Own::Vault(royalty_vault_id),
        };
        let metadata_substate = MetadataSubstate {
            metadata: input.metadata,
        };
        let access_rules = MethodAccessRulesSubstate {
            access_rules: input.access_rules,
        };

        // TODO: Can we trust developers enough to add protection for
        // - `metadata::set`
        // - `access_rules_chain::add_access_rules`
        // - `royalty::set_royalty_config`
        // - `royalty::claim_royalty`

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
            RENodeModuleInit::PackageRoyalty(package_royalty_config, package_royalty_accumulator),
        );
        node_modules.insert(
            NodeModuleId::Metadata,
            RENodeModuleInit::Metadata(metadata_substate),
        );
        node_modules.insert(
            NodeModuleId::AccessRules,
            RENodeModuleInit::MethodAccessRules(access_rules),
        );
        node_modules.insert(
            NodeModuleId::FunctionAccessRules,
            RENodeModuleInit::FunctionAccessRules(FunctionAccessRulesSubstate {
                access_rules: BTreeMap::new(),
                default_auth: AccessRule::AllowAll,
            }),
        );
        node_modules.insert(
            NodeModuleId::PackageEventSchema,
            RENodeModuleInit::PackageEventSchema(PackageEventSchemaSubstate(BTreeMap::new())), // TODO: To rework in Pt3
        );

        // Create package node
        let node_id = if let Some(address) = input.package_address {
            RENodeId::GlobalObject(PackageAddress::Normal(address).into())
        } else {
            api.kernel_allocate_node_id(RENodeType::GlobalPackage)?
        };

        api.kernel_create_node(
            node_id,
            RENodeInit::GlobalPackage(
                package_info_substate,
                code_type_substate,
                wasm_code_substate,
            ),
            node_modules,
        )?;

        let package_address: PackageAddress = node_id.into();

        Ok(IndexedScryptoValue::from_typed(&package_address))
    }
}
