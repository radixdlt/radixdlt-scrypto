use crate::engine::*;
use crate::wasm::WasmEngine;
use radix_engine_interface::api::types::*;
use radix_engine_interface::api::EngineApi;

impl ExecutableInvocation for LoggerLogInvocation {
    type Exec = Self;

    fn resolve<D: ResolverApi>(
        self,
        _deref: &mut D,
    ) -> Result<(ResolvedActor, CallFrameUpdate, Self::Exec), RuntimeError>
    where
        Self: Sized,
    {
        let actor = ResolvedActor::method(
            NativeFn::Logger(LoggerFn::Log),
            ResolvedReceiver::new(RENodeId::Logger),
        );
        let call_frame_update = CallFrameUpdate::empty();

        Ok((actor, call_frame_update, self))
    }
}

impl Executor for LoggerLogInvocation {
    type Output = ();

    fn execute<Y, W: WasmEngine>(
        self,
        api: &mut Y,
    ) -> Result<(Self::Output, CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi + EngineApi<RuntimeError>,
    {
        let offset = SubstateOffset::Logger(LoggerOffset::Logger);
        let node_id = RENodeId::Logger;
        let handle = api.lock_substate(node_id, offset, LockFlags::MUTABLE)?;
        let mut substate = api.get_ref_mut(handle)?;
        let logger = substate.logger();
        logger.logs.push((self.level, self.message));

        Ok(((), CallFrameUpdate::empty()))
    }
}
