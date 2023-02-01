use crate::engine::{
    deref_and_update, CallFrameUpdate, ExecutableInvocation, Executor, InterpreterError, LockFlags,
    ResolvedActor, ResolverApi, RuntimeError, SystemApi,
};
use crate::types::*;
use crate::wasm::WasmEngine;
use radix_engine_interface::api::types::{NativeFn, RENodeId, SubstateOffset};
use radix_engine_interface::api::EngineApi;
use radix_engine_interface::model::*;

impl ExecutableInvocation for MetadataSetInvocation {
    type Exec = Self;

    fn resolve<D: ResolverApi>(
        mut self,
        deref: &mut D,
    ) -> Result<(ResolvedActor, CallFrameUpdate, Self::Exec), RuntimeError> {
        let mut call_frame_update = CallFrameUpdate::empty();

        let resolved_receiver = deref_and_update(self.receiver, &mut call_frame_update, deref)?;

        // TODO: Move this into a more static check once node types implemented
        match &resolved_receiver.receiver {
            RENodeId::Package(..)
            | RENodeId::ResourceManager(..)
            | RENodeId::Component(..)
            | RENodeId::Validator(..)
            | RENodeId::Identity(..) => {}
            _ => {
                return Err(RuntimeError::InterpreterError(
                    InterpreterError::InvalidInvocation,
                ))
            }
        }

        self.receiver = resolved_receiver.receiver;
        let actor = ResolvedActor::method(NativeFn::Metadata(MetadataFn::Set), resolved_receiver);

        Ok((actor, call_frame_update, self))
    }
}

impl Executor for MetadataSetInvocation {
    type Output = ();

    fn execute<Y, W: WasmEngine>(
        self,
        system_api: &mut Y,
    ) -> Result<(Self::Output, CallFrameUpdate), RuntimeError>
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
    type Exec = Self;

    fn resolve<D: ResolverApi>(
        mut self,
        deref: &mut D,
    ) -> Result<(ResolvedActor, CallFrameUpdate, Self::Exec), RuntimeError> {
        let mut call_frame_update = CallFrameUpdate::empty();

        let resolved_receiver = deref_and_update(self.receiver, &mut call_frame_update, deref)?;

        // TODO: Move this into a more static check once node types implemented
        match &resolved_receiver.receiver {
            RENodeId::Package(..)
            | RENodeId::ResourceManager(..)
            | RENodeId::Component(..)
            | RENodeId::Validator(..)
            | RENodeId::Identity(..) => {}
            _ => {
                return Err(RuntimeError::InterpreterError(
                    InterpreterError::InvalidInvocation,
                ))
            }
        }

        self.receiver = resolved_receiver.receiver;
        let actor = ResolvedActor::method(NativeFn::Metadata(MetadataFn::Get), resolved_receiver);

        Ok((actor, call_frame_update, self))
    }
}

impl Executor for MetadataGetInvocation {
    type Output = Option<String>;

    fn execute<Y, W: WasmEngine>(
        self,
        api: &mut Y,
    ) -> Result<(Self::Output, CallFrameUpdate), RuntimeError>
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
