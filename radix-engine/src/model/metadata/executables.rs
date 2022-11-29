use crate::engine::{
    deref_and_update, CallFrameUpdate, ExecutableInvocation, InterpreterError, LockFlags,
    MethodDeref, NativeExecutor, NativeProcedure, REActor, ResolvedMethod, RuntimeError, SystemApi,
};
use crate::types::*;
use radix_engine_interface::api::api::EngineApi;
use radix_engine_interface::api::types::{NativeMethod, RENodeId, SubstateOffset};
use radix_engine_interface::model::*;

impl ExecutableInvocation for MetadataSetInvocation {
    type Exec = NativeExecutor<Self>;

    fn resolve<D: MethodDeref>(
        mut self,
        deref: &mut D,
    ) -> Result<(REActor, CallFrameUpdate, Self::Exec), RuntimeError> {
        let mut call_frame_update = CallFrameUpdate::empty();

        let resolved_receiver = deref_and_update(self.receiver, &mut call_frame_update, deref)?;

        // TODO: Move this into a more static check once node types implemented
        if !matches!(resolved_receiver.receiver, RENodeId::Package(..)) {
            return Err(RuntimeError::InterpreterError(
                InterpreterError::InvalidInvocation,
            ));
        }

        self.receiver = resolved_receiver.receiver;
        let actor = REActor::Method(
            ResolvedMethod::Native(NativeMethod::Metadata(MetadataMethod::Set)),
            resolved_receiver,
        );

        let executor = NativeExecutor(self);
        Ok((actor, call_frame_update, executor))
    }
}

impl NativeProcedure for MetadataSetInvocation {
    type Output = ();

    fn main<Y>(self, system_api: &mut Y) -> Result<(Self::Output, CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi + EngineApi<RuntimeError>,
    {
        let offset = SubstateOffset::Metadata(MetadataOffset::Metadata);
        let handle = system_api.lock_substate(self.receiver, offset, LockFlags::MUTABLE)?;

        let mut substate_ref_mut = system_api.get_ref_mut(handle)?;
        let metadata = substate_ref_mut.metadata();
        metadata.metadata.insert(self.key, self.value);

        Ok(((), CallFrameUpdate::empty()))
    }
}

impl ExecutableInvocation for MetadataGetInvocation {
    type Exec = NativeExecutor<Self>;

    fn resolve<D: MethodDeref>(
        mut self,
        deref: &mut D,
    ) -> Result<(REActor, CallFrameUpdate, Self::Exec), RuntimeError> {
        let mut call_frame_update = CallFrameUpdate::empty();

        let resolved_receiver = deref_and_update(self.receiver, &mut call_frame_update, deref)?;

        // TODO: Move this into a more static check once node types implemented
        if !matches!(resolved_receiver.receiver, RENodeId::Package(..)) {
            return Err(RuntimeError::InterpreterError(
                InterpreterError::InvalidInvocation,
            ));
        }

        self.receiver = resolved_receiver.receiver;
        let actor = REActor::Method(
            ResolvedMethod::Native(NativeMethod::Metadata(MetadataMethod::Get)),
            resolved_receiver,
        );

        let executor = NativeExecutor(self);
        Ok((actor, call_frame_update, executor))
    }
}

impl NativeProcedure for MetadataGetInvocation {
    type Output = Option<String>;

    fn main<Y>(self, api: &mut Y) -> Result<(Self::Output, CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi + EngineApi<RuntimeError>,
    {
        let offset = SubstateOffset::Metadata(MetadataOffset::Metadata);
        let handle = api.lock_substate(self.receiver, offset, LockFlags::MUTABLE)?;

        let substate_ref = api.get_ref(handle)?;
        let metadata = substate_ref.metadata();

        let rtn = metadata.metadata.get(&self.key).cloned();

        Ok((rtn, CallFrameUpdate::empty()))
    }
}
