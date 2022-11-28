use crate::engine::RENode;
use crate::engine::{
    CallFrameUpdate, LockFlags, NativeExecutable, NativeInvocation, NativeInvocationInfo,
    RuntimeError, SystemApi,
};
use crate::model::BucketSubstate;
use crate::types::*;
use radix_engine_interface::api::types::RENodeId;
use radix_engine_interface::api::types::{ComponentOffset, NativeMethod, SubstateOffset};
use radix_engine_interface::model::*;

#[derive(Debug, Clone, Eq, PartialEq, TypeId, Encode, Decode)]
pub enum ComponentError {
    InvalidRequestData(DecodeError),
}

impl NativeExecutable for ComponentSetRoyaltyConfigInvocation {
    type NativeOutput = ();

    fn execute<Y>(input: Self, system_api: &mut Y) -> Result<((), CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi,
    {
        // TODO: auth check
        let node_id = input.receiver;
        let offset = SubstateOffset::Component(ComponentOffset::RoyaltyConfig);
        let handle = system_api.lock_substate(node_id, offset, LockFlags::MUTABLE)?;

        let mut substate_mut = system_api.get_ref_mut(handle)?;
        substate_mut.component_royalty_config().royalty_config = input.royalty_config;

        system_api.drop_lock(handle)?;

        Ok(((), CallFrameUpdate::empty()))
    }
}

impl NativeInvocation for ComponentSetRoyaltyConfigInvocation {
    fn info(&self) -> NativeInvocationInfo {
        NativeInvocationInfo::Method(
            NativeMethod::Component(ComponentMethod::SetRoyaltyConfig),
            self.receiver,
            CallFrameUpdate::empty(),
        )
    }
}

impl NativeExecutable for ComponentClaimRoyaltyInvocation {
    type NativeOutput = Bucket;

    fn execute<Y>(
        input: Self,
        system_api: &mut Y,
    ) -> Result<(Bucket, CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi,
    {
        // TODO: auth check
        let node_id = input.receiver;
        let offset = SubstateOffset::Component(ComponentOffset::RoyaltyAccumulator);
        let handle = system_api.lock_substate(node_id, offset, LockFlags::MUTABLE)?;

        let mut substate_mut = system_api.get_ref_mut(handle)?;
        let resource = substate_mut
            .component_royalty_accumulator()
            .royalty
            .take_all();
        let bucket_node_id =
            system_api.create_node(RENode::Bucket(BucketSubstate::new(resource)))?;
        let bucket_id = bucket_node_id.into();

        system_api.drop_lock(handle)?;

        Ok((
            Bucket(bucket_id),
            CallFrameUpdate::move_node(RENodeId::Bucket(bucket_id)),
        ))
    }
}

impl NativeInvocation for ComponentClaimRoyaltyInvocation {
    fn info(&self) -> NativeInvocationInfo {
        NativeInvocationInfo::Method(
            NativeMethod::Component(ComponentMethod::SetRoyaltyConfig),
            self.receiver,
            CallFrameUpdate::empty(),
        )
    }
}
