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
use crate::system::node_module::metadata::MetadataNativePackage;
use crate::system::node_module::role_assignment::RoleAssignmentNativePackage;
use crate::system::node_module::royalty::RoyaltyNativePackage;
use crate::system::system_callback::SystemLockData;
use crate::types::*;
use crate::vm::VmInvoke;
use radix_engine_interface::api::ClientApi;
use radix_engine_interface::blueprints::package::*;
use resources_tracker_macro::trace_resources;

#[derive(Clone)]
pub struct NativeVm<E: NativeVmExtension> {
    extension: E,
}

impl<E: NativeVmExtension> NativeVm<E> {
    pub fn new_with_extension(extension: E) -> Self {
        Self { extension }
    }

    pub fn create_instance(
        &self,
        package_address: &PackageAddress,
        code: &[u8],
    ) -> Result<NativeVmInstance<E::Instance>, RuntimeError> {
        if let Some(custom_invoke) = self.extension.try_create_instance(code) {
            return Ok(NativeVmInstance::Extension(custom_invoke));
        }

        let code: [u8; 8] = match code.clone().try_into() {
            Ok(code) => code,
            Err(..) => {
                return Err(RuntimeError::VmError(VmError::Native(
                    NativeRuntimeError::InvalidCodeId,
                )));
            }
        };
        let native_package_code_id = u64::from_be_bytes(code);
        let instance = NativeVmInstance::Native {
            package_address: *package_address,
            native_package_code_id,
        };

        Ok(instance)
    }
}

pub enum NativeVmInstance<I: VmInvoke> {
    Native {
        // Used by profiling
        #[allow(dead_code)]
        package_address: PackageAddress,
        native_package_code_id: u64,
    },
    Extension(I),
}

impl<I: VmInvoke> NativeVmInstance<I> {
    // Used by profiling
    #[allow(dead_code)]
    pub fn package_address(&self) -> PackageAddress {
        match self {
            NativeVmInstance::Native {
                package_address, ..
            } => package_address.clone(),
            _ => panic!("Profiling with NativeVmExtension is not supported."),
        }
    }
}

impl<I: VmInvoke> VmInvoke for NativeVmInstance<I> {
    #[trace_resources(log=self.package_address().is_native_package(), log=self.package_address().to_hex(), log=export_name)]
    fn invoke<Y>(
        &mut self,
        export_name: &str,
        input: &IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: ClientApi<RuntimeError> + KernelNodeApi + KernelSubstateApi<SystemLockData>,
    {
        match self {
            NativeVmInstance::Extension(e) => e.invoke(export_name, input, api),
            NativeVmInstance::Native {
                native_package_code_id,
                package_address,
            } => {
                api.consume_cost_units(ClientCostingEntry::RunNativeCode {
                    package_address: package_address,
                    export_name: export_name,
                    input_size: input.len(),
                })?;

                match *native_package_code_id {
                    PACKAGE_CODE_ID => PackageNativePackage::invoke_export(export_name, input, api),
                    RESOURCE_CODE_ID => {
                        ResourceNativePackage::invoke_export(export_name, input, api)
                    }
                    CONSENSUS_MANAGER_CODE_ID => {
                        ConsensusManagerNativePackage::invoke_export(export_name, input, api)
                    }
                    IDENTITY_CODE_ID => {
                        IdentityNativePackage::invoke_export(export_name, input, api)
                    }
                    ACCOUNT_CODE_ID => AccountNativePackage::invoke_export(export_name, input, api),
                    ACCESS_CONTROLLER_CODE_ID => {
                        AccessControllerNativePackage::invoke_export(export_name, input, api)
                    }
                    TRANSACTION_PROCESSOR_CODE_ID => {
                        TransactionProcessorNativePackage::invoke_export(export_name, input, api)
                    }
                    METADATA_CODE_ID => {
                        MetadataNativePackage::invoke_export(export_name, input, api)
                    }
                    ROYALTY_CODE_ID => RoyaltyNativePackage::invoke_export(export_name, input, api),
                    ROLE_ASSIGNMENT_CODE_ID => {
                        RoleAssignmentNativePackage::invoke_export(export_name, input, api)
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
    }
}

pub trait NativeVmExtension: Clone {
    type Instance: VmInvoke + Clone;

    fn try_create_instance(&self, code: &[u8]) -> Option<Self::Instance>;
}

#[derive(Clone)]
pub struct NoExtension;
impl NativeVmExtension for NoExtension {
    type Instance = NullVmInvoke;
    fn try_create_instance(&self, _code: &[u8]) -> Option<Self::Instance> {
        None
    }
}

pub type DefaultNativeVm = NativeVm<NoExtension>;

impl DefaultNativeVm {
    pub fn new() -> Self {
        NativeVm::new_with_extension(NoExtension)
    }
}

#[derive(Clone)]
pub struct NullVmInvoke;

impl VmInvoke for NullVmInvoke {
    fn invoke<Y>(
        &mut self,
        _export_name: &str,
        _input: &IndexedScryptoValue,
        _api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: ClientApi<RuntimeError> + KernelNodeApi + KernelSubstateApi<SystemLockData>,
    {
        panic!("Invocation was called on null VmInvoke");
    }
}

#[derive(Clone)]
pub struct OverridePackageCode<C: VmInvoke + Clone> {
    custom_package_code_id: u64,
    custom_invoke: C,
}

impl<C: VmInvoke + Clone> OverridePackageCode<C> {
    pub fn new(custom_package_code_id: u64, custom_invoke: C) -> Self {
        Self {
            custom_package_code_id,
            custom_invoke,
        }
    }
}

impl<C: VmInvoke + Clone> NativeVmExtension for OverridePackageCode<C> {
    type Instance = C;

    fn try_create_instance(&self, code: &[u8]) -> Option<C> {
        let code_id = {
            let code: [u8; 8] = match code.clone().try_into() {
                Ok(code) => code,
                Err(..) => return None,
            };
            u64::from_be_bytes(code)
        };

        if self.custom_package_code_id == code_id {
            Some(self.custom_invoke.clone())
        } else {
            None
        }
    }
}
