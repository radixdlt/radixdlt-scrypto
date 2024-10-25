use crate::blueprints::access_controller::v1::*;
use crate::blueprints::access_controller::v2::*;
use crate::blueprints::account::AccountBlueprintCuttlefishExtension;
use crate::blueprints::account::{AccountBlueprintBottlenoseExtension, AccountNativePackage};
use crate::blueprints::consensus_manager::{
    ConsensusManagerNativePackage, ConsensusManagerSecondsPrecisionNativeCode,
};
use crate::blueprints::identity::IdentityNativePackage;
use crate::blueprints::identity::IdentityV1MinorVersion;
use crate::blueprints::locker::LockerNativePackage;
use crate::blueprints::package::PackageNativePackage;
use crate::blueprints::pool::v1::package::*;
use crate::blueprints::resource::{ResourceNativePackage, WorktopBlueprintCuttlefishExtension};
use crate::blueprints::test_utils::TestUtilsNativePackage;
use crate::blueprints::transaction_processor::{
    TransactionProcessorNativePackage, TransactionProcessorV1MinorVersion,
};
use crate::blueprints::transaction_tracker::TransactionTrackerNativePackage;
use crate::errors::{NativeRuntimeError, RuntimeError, VmError};
use crate::internal_prelude::*;
use crate::kernel::kernel_api::{KernelNodeApi, KernelSubstateApi};
use crate::object_modules::metadata::MetadataNativePackage;
use crate::object_modules::role_assignment::*;
use crate::object_modules::royalty::RoyaltyNativePackage;
use crate::system::system_callback::SystemLockData;
use crate::vm::{VmApi, VmInvoke};
use radix_engine_interface::api::SystemApi;
use radix_engine_interface::blueprints::package::*;
use radix_engine_profiling_derive::trace_resources;

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

        let code: [u8; 8] = match code.try_into() {
            Ok(code) => code,
            // It should be impossible for us to get to this point here. The code argument is
            // provided by the Vm after it reads the `PackageCodeOriginalCodeEntrySubstate`. Thus,
            // if the code-id at this point is invalid for the native-vm, then this means that the
            // database has been corrupted. We could safely panic here, however, we're choosing to
            // keep the `Err` here for safety.
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
    fn invoke<
        Y: SystemApi<RuntimeError> + KernelNodeApi + KernelSubstateApi<SystemLockData>,
        V: VmApi,
    >(
        &mut self,
        export_name: &str,
        input: &IndexedScryptoValue,
        api: &mut Y,
        vm_api: &V,
    ) -> Result<IndexedScryptoValue, RuntimeError> {
        #[allow(unused_mut)]
        let mut func = || match self {
            NativeVmInstance::Extension(e) => e.invoke(export_name, input, api, vm_api),
            NativeVmInstance::Native {
                native_package_code_id,
                package_address,
            } => {
                api.consume_cost_units(ClientCostingEntry::RunNativeCode {
                    package_address: package_address,
                    export_name: export_name,
                    input_size: input.len(),
                })?;

                let code_id = NativeCodeId::from_repr(*native_package_code_id).ok_or(
                    RuntimeError::VmError(VmError::Native(NativeRuntimeError::InvalidCodeId)),
                )?;

                match code_id {
                    NativeCodeId::PackageCode1 => PackageNativePackage::invoke_export(
                        export_name,
                        input,
                        PackageV1MinorVersion::Zero,
                        api,
                        vm_api,
                    ),
                    NativeCodeId::PackageCode2 => PackageNativePackage::invoke_export(
                        export_name,
                        input,
                        PackageV1MinorVersion::One,
                        api,
                        vm_api,
                    ),
                    NativeCodeId::ResourceCode1 => {
                        ResourceNativePackage::invoke_export(export_name, input, api)
                    }
                    NativeCodeId::ResourceCode2 => {
                        WorktopBlueprintCuttlefishExtension::invoke_export(export_name, input, api)
                    }
                    NativeCodeId::ConsensusManagerCode1 => {
                        ConsensusManagerNativePackage::invoke_export(export_name, input, api)
                    }
                    NativeCodeId::ConsensusManagerCode2 => {
                        ConsensusManagerSecondsPrecisionNativeCode::invoke_export(
                            export_name,
                            input,
                            api,
                        )
                    }
                    NativeCodeId::IdentityCode1 => IdentityNativePackage::invoke_export(
                        export_name,
                        input,
                        IdentityV1MinorVersion::Zero,
                        api,
                    ),
                    NativeCodeId::IdentityCode2 => IdentityNativePackage::invoke_export(
                        export_name,
                        input,
                        IdentityV1MinorVersion::One,
                        api,
                    ),
                    NativeCodeId::AccountCode1 => {
                        AccountNativePackage::invoke_export(export_name, input, api)
                    }
                    NativeCodeId::AccountCode2 => {
                        AccountBlueprintBottlenoseExtension::invoke_export(export_name, input, api)
                    }
                    NativeCodeId::AccountCode3 => {
                        AccountBlueprintCuttlefishExtension::invoke_export(export_name, input, api)
                    }
                    NativeCodeId::AccessControllerCode1 => {
                        AccessControllerV1NativePackage::invoke_export(export_name, input, api)
                    }
                    NativeCodeId::AccessControllerCode2 => {
                        AccessControllerV2NativePackage::invoke_export(export_name, input, api)
                    }
                    NativeCodeId::TransactionProcessorCode1 => {
                        TransactionProcessorNativePackage::invoke_export(
                            export_name,
                            input,
                            TransactionProcessorV1MinorVersion::Zero,
                            api,
                        )
                    }
                    NativeCodeId::TransactionProcessorCode2 => {
                        TransactionProcessorNativePackage::invoke_export(
                            export_name,
                            input,
                            TransactionProcessorV1MinorVersion::One,
                            api,
                        )
                    }
                    NativeCodeId::MetadataCode1 => {
                        MetadataNativePackage::invoke_export(export_name, input, api)
                    }
                    NativeCodeId::RoyaltyCode1 => {
                        RoyaltyNativePackage::invoke_export(export_name, input, api)
                    }
                    NativeCodeId::RoleAssignmentCode1 => {
                        RoleAssignmentNativePackage::invoke_export(export_name, input, api)
                    }
                    NativeCodeId::RoleAssignmentCode2 => {
                        RoleAssignmentBottlenoseExtension::invoke_export(export_name, input, api)
                    }
                    NativeCodeId::PoolCode1 => PoolNativePackage::invoke_export(
                        export_name,
                        input,
                        PoolV1MinorVersion::Zero,
                        api,
                    ),
                    NativeCodeId::PoolCode2 => PoolNativePackage::invoke_export(
                        export_name,
                        input,
                        PoolV1MinorVersion::One,
                        api,
                    ),
                    NativeCodeId::TransactionTrackerCode1 => {
                        TransactionTrackerNativePackage::invoke_export(export_name, input, api)
                    }
                    NativeCodeId::TestUtilsCode1 => {
                        TestUtilsNativePackage::invoke_export(export_name, input, api)
                    }
                    NativeCodeId::LockerCode1 => {
                        LockerNativePackage::invoke_export(export_name, input, api)
                    }
                }
            }
        };

        // Note: we can't unwind if we're compiling for no-std. See:
        // https://github.com/rust-lang/rfcs/issues/2810
        {
            #[cfg(feature = "std")]
            {
                match std::panic::catch_unwind(std::panic::AssertUnwindSafe(func)) {
                    Ok(rtn) => rtn,
                    Err(cause) => {
                        let message = if let Some(s) = cause.downcast_ref::<&'static str>() {
                            (*s).to_string()
                        } else if let Some(s) = cause.downcast_ref::<String>() {
                            s.clone()
                        } else {
                            "Unknown panic!".to_string()
                        };
                        Err(RuntimeError::VmError(VmError::Native(
                            NativeRuntimeError::Trap {
                                export_name: export_name.to_owned(),
                                input: input.as_scrypto_value().clone(),
                                error: message,
                            },
                        )))
                    }
                }
            }

            #[cfg(not(feature = "std"))]
            func()
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
    fn invoke<
        Y: SystemApi<RuntimeError> + KernelNodeApi + KernelSubstateApi<SystemLockData>,
        V: VmApi,
    >(
        &mut self,
        _export_name: &str,
        _input: &IndexedScryptoValue,
        _api: &mut Y,
        _vm_api: &V,
    ) -> Result<IndexedScryptoValue, RuntimeError> {
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
            let code: [u8; 8] = match code.try_into() {
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
