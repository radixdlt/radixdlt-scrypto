use crate::errors::*;
use crate::kernel::kernel_api::{KernelNodeApi, KernelSubstateApi};
use crate::system::kernel_modules::costing::{FIXED_HIGH_FEE, FIXED_MEDIUM_FEE};
use crate::system::node::RENodeInit;
use crate::system::node::RENodeModuleInit;
use crate::system::node_modules::access_rules::{
    FunctionAccessRulesSubstate, MethodAccessRulesSubstate,
};
use crate::system::node_modules::metadata::MetadataSubstate;
use crate::system::type_info::PackageCodeTypeSubstate;
use crate::types::*;
use crate::wasm::{PrepareError, WasmValidator};
use core::fmt::Debug;
use native_sdk::resource::ResourceManager;
use radix_engine_interface::api::package::*;
use radix_engine_interface::api::types::RENodeId;
use radix_engine_interface::api::types::*;
use radix_engine_interface::api::unsafe_api::ClientCostingReason;
use radix_engine_interface::api::{ClientApi, LockFlags};
use radix_engine_interface::blueprints::resource::AccessRule;
use radix_engine_interface::data::ScryptoValue;

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum PackageError {
    InvalidRequestData(DecodeError),
    InvalidAbi(DecodeError),
    InvalidWasm(PrepareError),
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
            PACKAGE_GET_FN_ABI_IDENT => {
                api.consume_cost_units(FIXED_MEDIUM_FEE, ClientCostingReason::RunNative)?;

                let receiver = receiver.ok_or(RuntimeError::InterpreterError(
                    InterpreterError::NativeExpectedReceiver(export_name.to_string()),
                ))?;

                let input: PackageGetFnAbiInput = scrypto_decode(&scrypto_encode(&input).unwrap())
                    .map_err(|_| {
                        RuntimeError::InterpreterError(InterpreterError::InvalidInvocation)
                    })?;

                Self::get_fn_abi(receiver, input, api).map(|r| IndexedScryptoValue::from_typed(&r))
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
            scrypto_decode(&scrypto_encode(&input).unwrap())
                .map_err(|_| RuntimeError::InterpreterError(InterpreterError::InvalidInvocation))?;

        let metadata_substate = MetadataSubstate {
            metadata: input.metadata,
        };
        let access_rules = MethodAccessRulesSubstate {
            access_rules: input.access_rules,
        };

        let mut node_modules = BTreeMap::new();
        node_modules.insert(
            NodeModuleId::Metadata,
            RENodeModuleInit::Metadata(metadata_substate),
        );
        node_modules.insert(
            NodeModuleId::AccessRules,
            RENodeModuleInit::ObjectAccessRulesChain(access_rules),
        );
        node_modules.insert(
            NodeModuleId::FunctionAccessRules,
            RENodeModuleInit::PackageAccessRules(FunctionAccessRulesSubstate {
                access_rules: input.package_access_rules,
                default_auth: input.default_package_access_rule,
            }),
        );

        let info = PackageInfoSubstate {
            dependent_resources: input.dependent_resources.into_iter().collect(),
            dependent_components: input.dependent_components.into_iter().collect(),
            blueprint_abis: input.abi,
        };
        let code_type = PackageCodeTypeSubstate::Native;
        let code = PackageCodeSubstate {
            code: vec![input.native_package_code_id],
        };

        // Create package node
        // Globalize
        let node_id = if let Some(address) = input.package_address {
            RENodeId::GlobalPackage(PackageAddress::Normal(address))
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
            .map_err(|_| RuntimeError::InterpreterError(InterpreterError::InvalidInvocation))?;

        let royalty_vault_id = ResourceManager(RADIX_TOKEN).new_vault(api)?.vault_id();

        WasmValidator::default()
            .validate(&input.code, &input.abi)
            .map_err(|e| {
                RuntimeError::ApplicationError(ApplicationError::PackageError(
                    PackageError::InvalidWasm(e),
                ))
            })?;

        let code_type_substate = PackageCodeTypeSubstate::Wasm;
        let wasm_code_substate = PackageCodeSubstate { code: input.code };
        let package_info_substate = PackageInfoSubstate {
            blueprint_abis: input.abi,
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
            NodeModuleId::PackageRoyalty,
            RENodeModuleInit::PackageRoyalty(package_royalty_config, package_royalty_accumulator),
        );
        node_modules.insert(
            NodeModuleId::Metadata,
            RENodeModuleInit::Metadata(metadata_substate),
        );
        node_modules.insert(
            NodeModuleId::AccessRules,
            RENodeModuleInit::ObjectAccessRulesChain(access_rules),
        );
        node_modules.insert(
            NodeModuleId::FunctionAccessRules,
            RENodeModuleInit::PackageAccessRules(FunctionAccessRulesSubstate {
                access_rules: BTreeMap::new(),
                default_auth: AccessRule::AllowAll,
            }),
        );

        // Create package node
        let node_id = if let Some(address) = input.package_address {
            RENodeId::GlobalPackage(PackageAddress::Normal(address))
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

    pub(crate) fn get_code_type<Y>(
        receiver: RENodeId,
        api: &mut Y,
    ) -> Result<PackageCodeTypeSubstate, RuntimeError>
    where
        Y: KernelSubstateApi,
    {
        let handle = api.kernel_lock_substate(
            receiver,
            NodeModuleId::SELF,
            SubstateOffset::Package(PackageOffset::CodeType),
            LockFlags::read_only(),
        )?;
        let code_type: &PackageCodeTypeSubstate = api.kernel_get_substate_ref(handle)?;
        let code_type = code_type.clone();
        api.kernel_drop_lock(handle)?;
        Ok(code_type)
    }

    pub(crate) fn get_fn_abi<Y>(
        receiver: RENodeId,
        input: PackageGetFnAbiInput,
        api: &mut Y,
    ) -> Result<Fn, RuntimeError>
    where
        Y: KernelSubstateApi,
    {
        let handle = api.kernel_lock_substate(
            receiver,
            NodeModuleId::SELF,
            SubstateOffset::Package(PackageOffset::Info),
            LockFlags::read_only(),
        )?;
        let info: &PackageInfoSubstate = api.kernel_get_substate_ref(handle)?;

        // Find the abi
        let abi =
            info.blueprint_abi(&input.fn_key.blueprint)
                .ok_or(RuntimeError::InterpreterError(
                    InterpreterError::InvalidScryptoInvocation(
                        FnIdentifier::new(
                            receiver.into(),
                            input.fn_key.blueprint.clone(),
                            input.fn_key.ident.clone(),
                        ),
                        ScryptoFnResolvingError::BlueprintNotFound,
                    ),
                ))?;
        let fn_abi = abi
            .get_fn_abi(&input.fn_key.ident)
            .ok_or(RuntimeError::InterpreterError(
                InterpreterError::InvalidScryptoInvocation(
                    FnIdentifier::new(
                        receiver.into(),
                        input.fn_key.blueprint.clone(),
                        input.fn_key.ident.clone(),
                    ),
                    ScryptoFnResolvingError::MethodNotFound,
                ),
            ))?;

        Ok(fn_abi.clone())
    }
}
