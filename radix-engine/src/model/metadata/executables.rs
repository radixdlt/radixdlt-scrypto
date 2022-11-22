use crate::engine::{ApplicationError, CallFrameUpdate, InterpreterError, Invokable, LockFlags, NativeInvocation, NativeInvocationInfo, NativeInvocationMethod, RuntimeError, SystemApi};
use crate::types::*;
use radix_engine_interface::api::api::{EngineApi, Invocation, SysInvokableNative};
use radix_engine_interface::api::types::{GlobalAddress, NativeMethod, RENodeId, SubstateOffset};
use radix_engine_interface::model::*;
use crate::model::ResourceManagerSetResourceAddressInvocation;

/*
impl NativeInvocation for MetadataSetInvocation {
    fn info(&self) -> NativeInvocationInfo {
        NativeInvocationInfo::Method(
            NativeMethod::Metadata(MetadataMethod::Set),
            self.receiver,
            CallFrameUpdate::empty(),
        )
    }

    fn execute<Y>(input: Self, system_api: &mut Y) -> Result<((), CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi,
    {
        let node_id = input.receiver;

        // TODO: Move this into a more static check once node types implemented
        if !matches!(node_id, RENodeId::Global(GlobalAddress::Package(_))) {
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
 */

impl NativeInvocationMethod for MetadataSetInvocation {
    type Receiver = RENodeId;
    type Args = MetadataSetArgs;

    fn info(self) -> (RENodeId, Self::Args, NativeMethod, CallFrameUpdate) {
        (
            self.receiver,
            MetadataSetArgs {
                key: self.key,
                value: self.value,
            },
            NativeMethod::Metadata(MetadataMethod::Set),
            CallFrameUpdate::empty(),
        )
    }

    fn execute<Y>(
        receiver: RENodeId,
        args: Self::Args,
        system_api: &mut Y,
    ) -> Result<(<Self as Invocation>::Output, CallFrameUpdate), RuntimeError>
        where
            Y: SystemApi
            + Invokable<ScryptoInvocation>
            + EngineApi<RuntimeError>
            + SysInvokableNative<RuntimeError>
            + Invokable<ResourceManagerSetResourceAddressInvocation> {
        // TODO: Move this into a more static check once node types implemented
        if !matches!(receiver, RENodeId::Global(GlobalAddress::Package(_))) {
            return Err(RuntimeError::InterpreterError(
                InterpreterError::InvalidInvocation,
            ));
        }

        let offset = SubstateOffset::Metadata(MetadataOffset::Metadata);
        let handle = system_api.lock_substate(receiver, offset, LockFlags::MUTABLE)?;

        let mut substate_ref_mut = system_api.get_ref_mut(handle)?;
        let metadata = substate_ref_mut.metadata();
        metadata.metadata.insert(args.key, args.value);

        Ok(((), CallFrameUpdate::empty()))
    }
}

pub struct MetadataSetArgs {
    pub key: String,
    pub value: String,
}
