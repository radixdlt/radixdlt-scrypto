use crate::blueprints::access_controller::AccessControllerNativePackage;
use crate::blueprints::account::AccountNativePackage;
use crate::blueprints::clock::ClockNativePackage;
use crate::blueprints::epoch_manager::EpochManagerNativePackage;
use crate::blueprints::identity::IdentityNativePackage;
use crate::blueprints::package::PackageNativePackage;
use crate::blueprints::resource::ResourceManagerNativePackage;
use crate::blueprints::transaction_processor::TransactionProcessorNativePackage;
use crate::errors::{RuntimeError, SystemInvokeError};
use crate::kernel::kernel_api::{KernelApi, KernelNodeApi, KernelSubstateApi};
use crate::system::node_modules::access_rules::AccessRulesNativePackage;
use crate::system::node_modules::metadata::MetadataNativePackage;
use crate::system::node_modules::royalty::RoyaltyNativePackage;
use crate::types::*;
use radix_engine_interface::api::{ClientApi, ClientTransactionLimitsApi};
use radix_engine_interface::blueprints::package::*;

pub struct NativeVm;

impl NativeVm {
    pub fn create_instance(
        package_address: PackageAddress,
        native_package_code_id: u8,
    ) -> NativeVmInstance {
        NativeVmInstance {
            native_package_code_id
        }
    }
}

pub struct NativeVmInstance {
    native_package_code_id: u8,
}

impl NativeVmInstance {
    pub fn invoke<'r, Y>(
        &mut self,
        receiver: Option<&NodeId>,
        export_name: &str,
        input: &IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
        where Y: ClientApi<RuntimeError> + KernelNodeApi + KernelSubstateApi  {

        match self.native_package_code_id {
            PACKAGE_CODE_ID => {
                PackageNativePackage::invoke_export(export_name, receiver, input, api)
            }
            RESOURCE_MANAGER_CODE_ID => {
                ResourceManagerNativePackage::invoke_export(export_name, receiver, input, api)
            }
            EPOCH_MANAGER_CODE_ID => {
                EpochManagerNativePackage::invoke_export(export_name, receiver, input, api)
            }
            IDENTITY_CODE_ID => {
                IdentityNativePackage::invoke_export(export_name, receiver, input, api)
            }
            CLOCK_CODE_ID => ClockNativePackage::invoke_export(export_name, receiver, input, api),
            ACCOUNT_CODE_ID => {
                AccountNativePackage::invoke_export(export_name, receiver, input, api)
            }
            ACCESS_CONTROLLER_CODE_ID => {
                AccessControllerNativePackage::invoke_export(export_name, receiver, input, api)
            }
            TRANSACTION_PROCESSOR_CODE_ID => {
                TransactionProcessorNativePackage::invoke_export(export_name, receiver, input, api)
            }
            METADATA_CODE_ID => {
                MetadataNativePackage::invoke_export(export_name, receiver, input, api)
            }
            ROYALTY_CODE_ID => {
                RoyaltyNativePackage::invoke_export(export_name, receiver, input, api)
            }
            ACCESS_RULES_CODE_ID => {
                AccessRulesNativePackage::invoke_export(export_name, receiver, input, api)
            }
            _ => Err(RuntimeError::SystemInvokeError(
                SystemInvokeError::NativeInvalidCodeId(self.native_package_code_id),
            )),
        }
    }
}
