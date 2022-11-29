use crate::engine::{deref_and_update, RENode};
use crate::engine::{
    CallFrameUpdate, ExecutableInvocation, LockFlags, MethodDeref, NativeExecutor, NativeProcedure,
    REActor, ResolvedMethod, RuntimeError, SystemApi,
};
use crate::model::BucketSubstate;
use crate::types::*;
use radix_engine_interface::api::types::{ComponentOffset, SubstateOffset};
use radix_engine_interface::data::IndexedScryptoValue;
use radix_engine_interface::model::*;

#[derive(Debug, Clone, Eq, PartialEq, TypeId, Encode, Decode)]
pub enum ComponentError {
    InvalidRequestData(DecodeError),
}

impl ExecutableInvocation for ComponentSetRoyaltyConfigInvocation {
    type Exec = NativeExecutor<Self>;

    fn resolve<D: MethodDeref>(
        self,
        deref: &mut D,
    ) -> Result<(REActor, CallFrameUpdate, Self::Exec), RuntimeError>
    where
        Self: Sized,
    {
        let input = IndexedScryptoValue::from_typed(&self);
        let mut call_frame_update = CallFrameUpdate::empty();
        let receiver = self.receiver;
        let resolved_receiver = deref_and_update(receiver, &mut call_frame_update, deref)?;

        let actor = REActor::Method(
            ResolvedMethod::Native(NativeMethod::Component(ComponentMethod::SetRoyaltyConfig)),
            resolved_receiver,
        );
        let executor = NativeExecutor(
            Self {
                receiver: resolved_receiver.receiver,
                royalty_config: self.royalty_config,
            },
            input,
        );

        Ok((actor, call_frame_update, executor))
    }
}

impl NativeProcedure for ComponentSetRoyaltyConfigInvocation {
    type Output = ();

    fn main<Y>(self, system_api: &mut Y) -> Result<((), CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi,
    {
        // TODO: auth check
        let node_id = self.receiver;
        let offset = SubstateOffset::Component(ComponentOffset::RoyaltyConfig);
        let handle = system_api.lock_substate(node_id, offset, LockFlags::MUTABLE)?;

        let mut substate_mut = system_api.get_ref_mut(handle)?;
        substate_mut.component_royalty_config().royalty_config = self.royalty_config;

        system_api.drop_lock(handle)?;

        Ok(((), CallFrameUpdate::empty()))
    }
}

impl ExecutableInvocation for ComponentClaimRoyaltyInvocation {
    type Exec = NativeExecutor<Self>;

    fn resolve<D: MethodDeref>(
        self,
        deref: &mut D,
    ) -> Result<(REActor, CallFrameUpdate, Self::Exec), RuntimeError> {
        let input = IndexedScryptoValue::from_typed(&self);
        let mut call_frame_update = CallFrameUpdate::empty();
        let receiver = self.receiver;
        let resolved_receiver = deref_and_update(receiver, &mut call_frame_update, deref)?;

        let actor = REActor::Method(
            ResolvedMethod::Native(NativeMethod::Component(ComponentMethod::ClaimRoyalty)),
            resolved_receiver,
        );
        let executor = NativeExecutor(
            Self {
                receiver: resolved_receiver.receiver,
            },
            input,
        );

        Ok((actor, call_frame_update, executor))
    }
}

impl NativeProcedure for ComponentClaimRoyaltyInvocation {
    type Output = Bucket;

    fn main<Y>(self, system_api: &mut Y) -> Result<(Bucket, CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi,
    {
        // TODO: auth check
        let node_id = self.receiver;
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
