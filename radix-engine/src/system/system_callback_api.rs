use crate::errors::RuntimeError;
use crate::internal_prelude::*;
use crate::kernel::kernel_api::{KernelInternalApi, KernelNodeApi, KernelSubstateApi};
use crate::system::system_callback::{System, SystemLockData};
use crate::track::BootStore;
use radix_engine_interface::api::ClientApi;
use radix_engine_interface::blueprints::package::PackageExport;

/// Invocation callback invoked by the system layer
pub trait SystemCallbackObject: Sized {
    type InitInput: Clone;

    /// Initialize the layer above the system with data from the substate store
    fn init<S: BootStore>(store: &S, init_input: Self::InitInput) -> Result<Self, BootloadingError>;

    fn invoke<Y>(
        package_address: &PackageAddress,
        package_export: PackageExport,
        input: &IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: ClientApi<RuntimeError>
            + KernelInternalApi<System<Self>>
            + KernelNodeApi
            + KernelSubstateApi<SystemLockData>;
}
