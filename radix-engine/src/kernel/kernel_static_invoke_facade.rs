use super::module::SysCallOutput;
use crate::errors::RuntimeError;
use crate::kernel::kernel_api::{KernelSubstateApi, KernelWasmApi};
use crate::kernel::module::BaseModule;
use crate::kernel::*;
use crate::system::kernel_modules::fee::FeeReserve;
use crate::wasm::WasmEngine;
use radix_engine_interface::api::static_invoke_api::Invocation;
use radix_engine_interface::api::{EngineApi, EngineDerefApi, Invokable};
use sbor::rust::fmt::Debug;

pub trait Executor {
    type Output: Debug;

    fn execute<Y, W>(self, api: &mut Y) -> Result<(Self::Output, CallFrameUpdate), RuntimeError>
    where
        Y: KernelSubstateApi + EngineApi<RuntimeError> + KernelWasmApi<W>,
        W: WasmEngine;
}

pub trait ExecutableInvocation: Invocation {
    type Exec: Executor<Output = Self::Output>;

    fn resolve<Y: EngineDerefApi<RuntimeError> + KernelSubstateApi>(
        self,
        api: &mut Y,
    ) -> Result<(ResolvedActor, CallFrameUpdate, Self::Exec), RuntimeError>;
}

impl<'g, 's, W, R, N, M> Invokable<N, RuntimeError> for Kernel<'g, 's, W, R, M>
where
    W: WasmEngine,
    R: FeeReserve,
    M: BaseModule<R>,
    N: ExecutableInvocation,
{
    fn invoke(&mut self, invocation: N) -> Result<<N as Invocation>::Output, RuntimeError> {
        self.module
            .pre_sys_call(
                &self.current_frame,
                &mut self.heap,
                &mut self.track,
                SysCallInput::Invoke {
                    fn_identifier: invocation.fn_identifier(),
                    input_size: 0, // TODO: Fix this
                    depth: self.current_frame.depth,
                },
            )
            .map_err(RuntimeError::ModuleError)?;

        // Change to kernel mode
        let saved_mode = self.execution_mode;

        self.execution_mode = ExecutionMode::Resolver;
        let (actor, call_frame_update, executor) = invocation.resolve(self)?;

        self.execution_mode = ExecutionMode::Kernel;
        let rtn = self.invoke_internal(executor, actor, call_frame_update)?;

        // Restore previous mode
        self.execution_mode = saved_mode;

        self.module
            .post_sys_call(
                &self.current_frame,
                &mut self.heap,
                &mut self.track,
                SysCallOutput::Invoke { rtn: &rtn },
            )
            .map_err(RuntimeError::ModuleError)?;

        Ok(rtn)
    }
}
