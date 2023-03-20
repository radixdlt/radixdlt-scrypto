use super::actor::Actor;
use super::call_frame::CallFrameUpdate;
use super::kernel_api::KernelNodeApi;
use super::kernel_api::KernelSubstateApi;
use super::kernel_api::KernelWasmApi;
use crate::errors::*;
use crate::types::*;
use crate::wasm::WasmEngine;
use radix_engine_interface::api::*;

pub trait ExecutableInvocation: Invocation {
    type Exec: Executor<Output = Self::Output>;

    fn resolve<Y: KernelSubstateApi>(
        self,
        api: &mut Y,
    ) -> Result<ResolvedInvocation<Self::Exec>, RuntimeError>;

    fn payload_size(&self) -> usize;
}

pub trait Executor {
    type Output: Debug;

    fn execute<Y, W>(
        self,
        args: IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<(Self::Output, CallFrameUpdate), RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + KernelWasmApi<W> + ClientApi<RuntimeError>,
        W: WasmEngine;
}

pub struct ResolvedInvocation<E: Executor> {
    pub executor: E,
    pub update: CallFrameUpdate,

    // TODO: Make these two RENodes / Substates
    pub resolved_actor: Option<Actor>,
    pub args: IndexedScryptoValue,
}
