use crate::engine::{BaseModule, Kernel, RuntimeError};
use crate::fee::FeeReserve;
use crate::wasm::WasmEngine;
use radix_engine_interface::api::InvokableModel;

impl<'g, 's, W, R, M> InvokableModel<RuntimeError> for Kernel<'g, 's, W, R, M>
where
    W: WasmEngine,
    R: FeeReserve,
    M: BaseModule<R>,
{
}
