use crate::errors::*;
use crate::kernel::kernel_api::{KernelNodeApi, KernelSubstateApi};
use crate::system::global::GlobalAddressSubstate;
use crate::system::node::RENodeInit;
use crate::system::node::RENodeModuleInit;
use crate::system::node_modules::auth::AccessRulesChainSubstate;
use crate::system::node_modules::metadata::MetadataSubstate;
use crate::types::*;
use core::fmt::Debug;
use radix_engine_interface::api::package::*;
use radix_engine_interface::api::types::*;
use radix_engine_interface::api::types::{PackageId, RENodeId};
use radix_engine_interface::api::{ClientApi};
use radix_engine_interface::data::ScryptoValue;

#[derive(Debug, Clone, PartialEq, Eq, Categorize, Encode, Decode)]
pub enum NativePackageError {
    InvalidAbi(DecodeError),
}

pub struct NativePackage;
impl NativePackage {
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
            NATIVE_PACKAGE_PUBLISH_IDENT => {
                if receiver.is_some() {
                    return Err(RuntimeError::InterpreterError(
                        InterpreterError::NativeUnexpectedReceiver(export_name.to_string()),
                    ));
                }

                Self::publish(input, api)
            }
            _ => Err(RuntimeError::InterpreterError(
                InterpreterError::NativeExportDoesNotExist(export_name.to_string()),
            )),
        }
    }

    pub(crate) fn publish<Y>(
        input: ScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
        where
            Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        let input: PackagePublishNativeInput =
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
                RuntimeError::ApplicationError(ApplicationError::NativePackageError(
                    NativePackageError::InvalidAbi(e),
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
}
