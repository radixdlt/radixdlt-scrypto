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

    fn execute<Y>(input: Self, _system_api: &mut Y) -> Result<((), CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi,
    {
        let node_id = input.receiver;

        // TODO: Move this into a more static check once node types implemented
        if !matches!(node_id, RENodeId::Global(..)) {
            return Err(RuntimeError::InterpreterError(
                InterpreterError::InvalidInvocation,
            ));
        }

        todo!();

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
