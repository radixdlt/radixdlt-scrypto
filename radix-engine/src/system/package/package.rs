use crate::errors::*;
use crate::kernel::kernel_api::{KernelNodeApi, KernelSubstateApi};
use crate::system::global::GlobalAddressSubstate;
use crate::system::node::RENodeInit;
use crate::system::node::RENodeModuleInit;
use crate::system::node_modules::auth::AccessRulesChainSubstate;
use crate::system::node_modules::metadata::MetadataSubstate;
use crate::types::*;
use core::fmt::Debug;
use native_sdk::resource::ResourceManager;
use radix_engine_interface::api::package::*;
use radix_engine_interface::api::types::*;
use radix_engine_interface::api::types::{PackageId, RENodeId};
use radix_engine_interface::api::{ClientApi};
use radix_engine_interface::data::ScryptoValue;
use crate::wasm::{PrepareError, WasmValidator};

#[derive(Debug, Clone, PartialEq, Eq, Categorize, Encode, Decode)]
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
            PACKAGE_PUBLISH_PRECOMPILED_IDENT => {
                if receiver.is_some() {
                    return Err(RuntimeError::InterpreterError(
                        InterpreterError::NativeUnexpectedReceiver(export_name.to_string()),
                    ));
                }

                Self::publish_precompiled(input, api)
            }
            PACKAGE_PUBLISH_WASM_IDENT => {
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

    pub(crate) fn publish_precompiled<Y>(
        input: ScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
        where
            Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        let input: PackagePublishPrecompiledInput =
            scrypto_decode(&scrypto_encode(&input).unwrap())
                .map_err(|_| RuntimeError::InterpreterError(InterpreterError::InvalidInvocation))?;

        let metadata_substate = MetadataSubstate {
            metadata: input.metadata,
        };
        let access_rules = AccessRulesChainSubstate {
            access_rules_chain: vec![input.access_rules],
        };
        let blueprint_abis =
            scrypto_decode::<BTreeMap<String, BlueprintAbi>>(&input.abi).map_err(|e| {
                RuntimeError::ApplicationError(ApplicationError::PackageError(
                    PackageError::InvalidAbi(e),
                ))
            })?;

        let mut node_modules = BTreeMap::new();
        node_modules.insert(
            NodeModuleId::Metadata,
            RENodeModuleInit::Metadata(metadata_substate),
        );
        node_modules.insert(
            NodeModuleId::AccessRules,
            RENodeModuleInit::AccessRulesChain(access_rules),
        );

        let info = PackageInfoSubstate {
            dependent_resources: input.dependent_resources.into_iter().collect(),
            dependent_components: input.dependent_components.into_iter().collect(),
            blueprint_abis,
        };
        let code = NativeCodeSubstate {
            native_package_code_id: input.native_package_code_id,
        };

        // Create package node
        let node_id = api.kernel_allocate_node_id(RENodeType::Package)?;
        api.kernel_create_node(node_id, RENodeInit::NativePackage(info, code), node_modules)?;
        let package_id: PackageId = node_id.into();

        // Globalize
        let global_node_id = if let Some(address) = input.package_address {
            RENodeId::Global(GlobalAddress::Package(PackageAddress::Normal(address)))
        } else {
            api.kernel_allocate_node_id(RENodeType::GlobalPackage)?
        };

        api.kernel_create_node(
            global_node_id,
            RENodeInit::Global(GlobalAddressSubstate::Package(package_id)),
            BTreeMap::new(),
        )?;

        let package_address: PackageAddress = global_node_id.into();
        Ok(IndexedScryptoValue::from_typed(&package_address))
    }

    pub(crate) fn publish_wasm<Y>(
        input: ScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
        where
            Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {

        let input: PackagePublishWasmInput =
            scrypto_decode(&scrypto_encode(&input).unwrap())
                .map_err(|_| RuntimeError::InterpreterError(InterpreterError::InvalidInvocation))?;

        let royalty_vault_id = ResourceManager(RADIX_TOKEN).new_vault(api)?.vault_id();

        let blueprint_abis =
            scrypto_decode::<BTreeMap<String, BlueprintAbi>>(&input.abi).map_err(|e| {
                RuntimeError::ApplicationError(ApplicationError::PackageError(
                    PackageError::InvalidAbi(e),
                ))
            })?;

        WasmValidator::default().validate(&input.code, &blueprint_abis).map_err(|e| {
            RuntimeError::ApplicationError(ApplicationError::PackageError(
                PackageError::InvalidWasm(e),
            ))
        })?;

        let wasm_code_substate = WasmCodeSubstate { code: input.code };
        let package_info_substate = PackageInfoSubstate {
            blueprint_abis,
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
        let access_rules = AccessRulesChainSubstate {
            access_rules_chain: vec![input.access_rules],
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
            RENodeModuleInit::AccessRulesChain(access_rules),
        );

        // Create package node
        let node_id = api.kernel_allocate_node_id(RENodeType::Package)?;
        api.kernel_create_node(
            node_id,
            RENodeInit::WasmPackage(package_info_substate, wasm_code_substate),
            node_modules,
        )?;
        let package_id: PackageId = node_id.into();

        // Globalize
        let global_node_id = if let Some(address) = input.package_address {
            RENodeId::Global(GlobalAddress::Package(PackageAddress::Normal(address)))
        } else {
            api.kernel_allocate_node_id(RENodeType::GlobalPackage)?
        };

        api.kernel_create_node(
            global_node_id,
            RENodeInit::Global(GlobalAddressSubstate::Package(package_id)),
            BTreeMap::new(),
        )?;

        let package_address: PackageAddress = global_node_id.into();

        Ok(IndexedScryptoValue::from_typed(&package_address))
    }
}
