use crate::engine::{Kernel, RuntimeError};
use crate::fee::FeeReserve;
use crate::wasm::WasmEngine;
use radix_engine_interface::api::api::SysInvokableNative;

impl<'g, 's, W, R> SysInvokableNative<RuntimeError> for Kernel<'g, 's, W, R>
where
    W: WasmEngine,
    R: FeeReserve,
{
}
