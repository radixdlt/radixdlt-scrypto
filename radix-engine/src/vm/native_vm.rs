use crate::blueprints::access_controller::AccessControllerNativePackage;
use crate::blueprints::account::AccountNativePackage;
use crate::blueprints::consensus_manager::ConsensusManagerNativePackage;
use crate::blueprints::identity::IdentityNativePackage;
use crate::blueprints::package::PackageNativePackage;
use crate::blueprints::pool::PoolNativePackage;
use crate::blueprints::resource::ResourceNativePackage;
use crate::blueprints::transaction_processor::TransactionProcessorNativePackage;
use crate::blueprints::transaction_tracker::TransactionTrackerNativePackage;
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
use resources_tracker_macro::trace_resources;

pub struct NativeVm;

impl NativeVm {
    pub fn create_instance(
        package_address: &PackageAddress,
        code: &[u8],
    ) -> Result<NativeVmInstance, RuntimeError> {
        let code: [u8; 8] = match code.clone().try_into() {
            Ok(code) => code,
            Err(..) => {
                return Err(RuntimeError::VmError(VmError::Native(
                    NativeRuntimeError::InvalidCodeId,
                )));
            }
        };

        let native_package_code_id = u64::from_be_bytes(code);

        let instance = NativeVmInstance {
            package_address: *package_address,
            native_package_code_id,
        };

        Ok(instance)
    }
}

pub struct NativeVmInstance {
    // Used by profiling
    #[allow(dead_code)]
    package_address: PackageAddress,
    native_package_code_id: u64,
}

impl VmInvoke for NativeVmInstance {
    #[trace_resources(log=self.package_address.is_native_address(), log=self.package_address.to_hex(), log=export_name)]
    fn invoke<Y>(
        &mut self,
        export_name: &str,
        input: &IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: ClientApi<RuntimeError> + KernelNodeApi + KernelSubstateApi<SystemLockData>,
    {
        api.consume_cost_units(ClientCostingEntry::RunNativeCode {
            package_address: &self.package_address,
            export_name: export_name,
            input_size: input.len(),
        })?;

        match self.native_package_code_id {
            PACKAGE_CODE_ID => PackageNativePackage::invoke_export(export_name, input, api),
            RESOURCE_CODE_ID => ResourceNativePackage::invoke_export(export_name, input, api),
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
            POOL_CODE_ID => PoolNativePackage::invoke_export(export_name, input, api),
            TRANSACTION_TRACKER_CODE_ID => {
                TransactionTrackerNativePackage::invoke_export(export_name, input, api)
            }
            _ => {
                return Err(RuntimeError::VmError(VmError::Native(
                    NativeRuntimeError::InvalidCodeId,
                )));
            }
        }
    }
}
