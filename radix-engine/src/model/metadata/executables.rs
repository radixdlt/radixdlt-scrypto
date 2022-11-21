use crate::engine::{
    ApplicationError, CallFrameUpdate, InterpreterError, LockFlags, NativeExecutable,
    NativeInvocation, NativeInvocationInfo, RuntimeError, SystemApi,
};
use crate::types::*;
use radix_engine_interface::api::types::{
    AccessRulesMethod, GlobalAddress, NativeMethod, PackageOffset, RENodeId, SubstateOffset,
};
use radix_engine_interface::model::*;

impl NativeExecutable for MetadataSetInvocation {
    type NativeOutput = ();

    fn execute<Y>(input: Self, system_api: &mut Y) -> Result<((), CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi,
    {
        let node_id = input.receiver;

        // TODO: Move this into a more static check once node types implemented
        if !matches!(node_id, RENodeId::Package(..)) {
            return Err(RuntimeError::InterpreterError(
                InterpreterError::InvalidInvocation,
            ));
        }

        let offset = SubstateOffset::Metadata(MetadataOffset::Metadata);
        let handle = system_api.lock_substate(node_id, offset, LockFlags::MUTABLE)?;

        let mut substate_ref_mut = system_api.get_ref_mut(handle)?;
        let metadata = substate_ref_mut.metadata();
        metadata.metadata.insert(input.key, input.value);

        Ok(((), CallFrameUpdate::empty()))
    }
}

impl NativeInvocation for MetadataSetInvocation {
    fn info(&self) -> NativeInvocationInfo {
        NativeInvocationInfo::Method(
            NativeMethod::Metadata(MetadataMethod::Set),
            self.receiver,
            CallFrameUpdate::empty(),
        )
    }
}
