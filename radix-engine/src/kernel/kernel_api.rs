use crate::errors::RuntimeError;
use crate::kernel::module::BaseModule;
use crate::kernel::Kernel;
use crate::system::kernel_modules::fee::FeeReserve;
use crate::wasm::WasmEngine;
use radix_engine_interface::api::static_link::InvokableModel;

impl<'g, 's, W, R, M> InvokableModel<RuntimeError> for Kernel<'g, 's, W, R, M>
where
    W: WasmEngine,
    R: FeeReserve,
    M: BaseModule<R>,
{
}

// Currently under `radix-engine/interface/src/api`.
//
// They will gradually be split into two sets of APIs:
// * Kernel API, for kernel modules and System
// * System API, for clients (wasm and native blueprints)
