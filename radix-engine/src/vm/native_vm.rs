use crate::blueprints::access_controller::AccessControllerNativePackage;
use crate::blueprints::account::AccountNativePackage;
use crate::blueprints::consensus_manager::ConsensusManagerNativePackage;
use crate::blueprints::identity::IdentityNativePackage;
use crate::blueprints::package::PackageNativePackage;
use crate::blueprints::pool::PoolNativePackage;
use crate::blueprints::resource::ResourceManagerNativePackage;
use crate::blueprints::transaction_processor::TransactionProcessorNativePackage;
use crate::errors::{NativeRuntimeError, RuntimeError, VmError};
use crate::kernel::kernel_api::{KernelNodeApi, KernelSubstateApi};
use crate::system::node_modules::access_rules::AccessRulesNativePackage;
use crate::system::node_modules::metadata::MetadataNativePackage;
use crate::system::node_modules::royalty::RoyaltyNativePackage;
use crate::system::system_callback::SystemLockData;
use crate::types::*;
use crate::vm::VmInvoke;
use radix_engine_interface::api::ClientApi;
use radix_engine_interface::blueprints::package::*;

pub struct NativeVm;

impl NativeVm {
    pub fn create_instance(
        _package_address: &PackageAddress,
        code: &[u8],
    ) -> Result<NativeVmInstance, RuntimeError> {
        if code.len() != 1 {
            return Err(RuntimeError::VmError(VmError::Native(
                NativeRuntimeError::InvalidCodeId,
            )));
        }

        let instance = NativeVmInstance {
            native_package_code_id: code[0],
        };

        Ok(instance)
    }
}

pub struct NativeVmInstance {
    native_package_code_id: u8,
}

impl VmInvoke for NativeVmInstance {
    fn invoke<Y>(
        &mut self,

        export_name: &str,
        input: &IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: ClientApi<RuntimeError> + KernelNodeApi + KernelSubstateApi<SystemLockData>,
    {
        match self.native_package_code_id {
            PACKAGE_CODE_ID => PackageNativePackage::invoke_export(export_name, input, api),
            RESOURCE_MANAGER_CODE_ID => {
                ResourceManagerNativePackage::invoke_export(export_name, input, api)
            }
            CONSENSUS_MANAGER_CODE_ID => {
                ConsensusManagerNativePackage::invoke_export(export_name, input, api)
            }
            IDENTITY_CODE_ID => IdentityNativePackage::invoke_export(export_name, input, api),
            ACCOUNT_CODE_ID => AccountNativePackage::invoke_export(export_name, input, api),
            ACCESS_CONTROLLER_CODE_ID => {
                AccessControllerNativePackage::invoke_export(export_name, input, api)
            }
            TRANSACTION_PROCESSOR_CODE_ID => {
                TransactionProcessorNativePackage::invoke_export(export_name, input, api)
            }
            METADATA_CODE_ID => MetadataNativePackage::invoke_export(export_name, input, api),
            ROYALTY_CODE_ID => RoyaltyNativePackage::invoke_export(export_name, input, api),
            ACCESS_RULES_CODE_ID => {
                AccessRulesNativePackage::invoke_export(export_name, input, api)
            }
            POOL_ID => PoolNativePackage::invoke_export(export_name, input, api),
            _ => {
                return Err(RuntimeError::VmError(VmError::Native(
                    NativeRuntimeError::InvalidCodeId,
                )));
            }
        }
    }
}
