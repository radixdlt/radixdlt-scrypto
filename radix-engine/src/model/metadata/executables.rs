use crate::engine::{
    deref_and_update, CallFrameUpdate, ExecutableInvocation, InterpreterError, LockFlags,
    MethodDeref, NativeExecutor, NativeProgram, REActor, ResolvedMethod, RuntimeError, SystemApi,
};
use crate::types::*;
use radix_engine_interface::api::api::EngineApi;
use radix_engine_interface::api::types::{NativeMethod, RENodeId, SubstateOffset};
use radix_engine_interface::data::IndexedScryptoValue;
use radix_engine_interface::model::*;

impl ExecutableInvocation for MetadataSetInvocation {
    type Exec = NativeExecutor<Self>;

    fn resolve<D: MethodDeref>(
        mut self,
        deref: &mut D,
    ) -> Result<(REActor, CallFrameUpdate, Self::Exec), RuntimeError> {
        let input = IndexedScryptoValue::from_typed(&self);
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

        let executor = NativeExecutor(self, input);
        Ok((actor, call_frame_update, executor))
    }
}

impl NativeProgram for MetadataSetInvocation {
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
