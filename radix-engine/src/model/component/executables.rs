use crate::engine::deref_and_update;
use crate::engine::{
    CallFrameUpdate, ExecutableInvocation, LockFlags, MethodDeref, NativeExecutor, NativeProcedure,
    REActor, ResolvedMethod, RuntimeError, SystemApi,
};
use crate::types::*;
use radix_engine_interface::api::types::{ComponentOffset, SubstateOffset};
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
        let mut call_frame_update = CallFrameUpdate::empty();
        let receiver = self.receiver;
        let resolved_receiver = deref_and_update(receiver, &mut call_frame_update, deref)?;

        let actor = REActor::Method(
            ResolvedMethod::Native(NativeMethod::EpochManager(
                EpochManagerMethod::GetCurrentEpoch,
            )),
            resolved_receiver,
        );
        let executor = NativeExecutor(Self {
            receiver: resolved_receiver.receiver,
            royalty_config: self.royalty_config,
        });

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
