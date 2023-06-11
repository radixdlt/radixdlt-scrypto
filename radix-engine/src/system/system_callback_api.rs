use crate::errors::RuntimeError;
use crate::kernel::kernel_api::{KernelInternalApi, KernelNodeApi, KernelSubstateApi};
use crate::system::system_callback::{SystemConfig, SystemLockData};
use crate::types::*;
use radix_engine_interface::api::ClientApi;
use radix_engine_interface::blueprints::package::PackageExport;

/// Invocation callback invoked by the system layer
pub trait SystemCallbackObject: Sized {
    fn invoke<Y>(
        package_address: &PackageAddress,
        package_export: PackageExport,
        input: &IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: ClientApi<RuntimeError>
            + KernelInternalApi<SystemConfig<Self>>
            + KernelNodeApi
            + KernelSubstateApi<SystemLockData>;
}
