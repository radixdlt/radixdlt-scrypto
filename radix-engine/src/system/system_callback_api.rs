use crate::errors::RuntimeError;
use crate::kernel::kernel_api::{KernelInternalApi, KernelNodeApi, KernelSubstateApi};
use crate::types::*;
use radix_engine_interface::api::ClientApi;
use crate::system::system_callback::SystemCallback;
use crate::vm::wasm::WasmEngine;

pub trait SystemCallbackApi<'g, W: WasmEngine + 'g> {
    // TODO: Remove KernelNodeAPI + KernelSubstateAPI from api
    fn invoke<Y>(
        package_address: &PackageAddress,
        receiver: Option<&NodeId>,
        export_name: &str,
        input: &IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: ClientApi<RuntimeError> + KernelInternalApi<SystemCallback<'g, W>> + KernelNodeApi + KernelSubstateApi;
}

pub trait VmInvoke {
    // TODO: Remove KernelNodeAPI + KernelSubstateAPI from api
    fn invoke<Y>(
        &mut self,
        receiver: Option<&NodeId>,
        export_name: &str,
        input: &IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: ClientApi<RuntimeError> + KernelNodeApi + KernelSubstateApi;
}
