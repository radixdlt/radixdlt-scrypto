use crate::errors::RuntimeError;
use crate::internal_prelude::*;
use crate::kernel::kernel_api::{KernelInternalApi, KernelNodeApi, KernelSubstateApi};
use crate::system::system_callback::{System, SystemLockData};
use crate::track::BootStore;
use radix_engine_interface::api::SystemApi;
use radix_engine_interface::blueprints::package::PackageExport;

/// Callback object invoked by the system layer
pub trait SystemCallbackObject: Sized {
    /// Initialization Object
    type Init: Clone;

    /// Initialize and create the callback object above the system
    fn init<S: BootStore>(store: &S, init_input: Self::Init) -> Result<Self, BootloadingError>;

    /// Invoke a function
    fn invoke<
        Y: SystemApi<RuntimeError>
            + KernelInternalApi<System<Self>>
            + KernelNodeApi
            + KernelSubstateApi<SystemLockData>,
    >(
        package_address: &PackageAddress,
        package_export: PackageExport,
        input: &IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>;
}
