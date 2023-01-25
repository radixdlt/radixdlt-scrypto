use crate::errors::RuntimeError;
use crate::kernel::module::BaseModule;
use crate::kernel::*;
use crate::system::invocation::native_wrapper::{invoke_call_table, resolve_method};
use crate::system::kernel_modules::fee::FeeReserve;
use crate::types::*;
use crate::wasm::WasmEngine;
use radix_engine_interface::api::types::ScryptoReceiver;
use radix_engine_interface::api::types::{LockHandle, RENodeId};
use radix_engine_interface::api::{
    EngineActorApi, EngineApi, EngineComponentApi, EngineDerefApi, EngineInvokeApi,
    EnginePackageApi,
};
use radix_engine_interface::data::*;

impl<'g, 's, W, R, M> EngineDerefApi<RuntimeError> for Kernel<'g, 's, W, R, M>
where
    W: WasmEngine,
    R: FeeReserve,
    M: BaseModule<R>,
{
    fn deref(&mut self, node_id: RENodeId) -> Result<Option<(RENodeId, LockHandle)>, RuntimeError> {
        self.node_method_deref(node_id)
    }
}

impl<'g, 's, W, R, M> EngineActorApi<RuntimeError> for Kernel<'g, 's, W, R, M>
where
    W: WasmEngine,
    R: FeeReserve,
    M: BaseModule<R>,
{
    fn fn_identifier(&mut self) -> Result<FnIdentifier, RuntimeError> {
        Ok(self.current_frame.actor.identifier.clone())
    }
}

impl<'g, 's, W, R, M> EngineInvokeApi<RuntimeError> for Kernel<'g, 's, W, R, M>
where
    W: WasmEngine,
    R: FeeReserve,
    M: BaseModule<R>,
{
}

impl<'g, 's, W, R, M> EnginePackageApi for Kernel<'g, 's, W, R, M>
where
    W: WasmEngine,
    R: FeeReserve,
    M: BaseModule<R>,
{
}

impl<'g, 's, W, R, M> EngineComponentApi<RuntimeError> for Kernel<'g, 's, W, R, M>
where
    W: WasmEngine,
    R: FeeReserve,
    M: BaseModule<R>,
{
    fn invoke_method(
        &mut self,
        receiver: ScryptoReceiver,
        method_name: &str,
        args: Vec<u8>,
    ) -> Result<IndexedScryptoValue, RuntimeError> {
        // TODO: Use execution mode?
        let invocation = resolve_method(receiver, method_name, &args, self)?;
        invoke_call_table(invocation, self)
    }
}

impl<'g, 's, W, R, M> EngineApi<RuntimeError> for Kernel<'g, 's, W, R, M>
where
    W: WasmEngine,
    R: FeeReserve,
    M: BaseModule<R>,
{
}
