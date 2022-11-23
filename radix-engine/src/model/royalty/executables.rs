use crate::engine::{
    AuthModule, CallFrameUpdate, LockFlags, NativeExecutable, NativeInvocation,
    NativeInvocationInfo, RuntimeError, SystemApi,
};
use crate::model::{
    HardAuthRule, HardProofRule, HardResourceOrNonFungible, MethodAuthorization,
    ResourceOperationError, RoyaltyReserveSubstate,
};
use crate::types::*;
use radix_engine_interface::api::types::{
    NativeMethod, RENodeId, RoyaltyReserveMethod, RoyaltyReserveOffset, SubstateOffset,
};
use radix_engine_interface::model::*;

#[derive(Debug, Clone, Eq, PartialEq)]
#[scrypto(TypeId, Encode, Decode)]
pub enum RoyaltyReserveError {
    InvalidRequestData(DecodeError),
    ResourceOperationError(ResourceOperationError),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RoyaltyReserve {
    pub info: RoyaltyReserveSubstate,
}

impl NativeInvocation for RoyaltyReservePutInvocation {
    fn info(&self) -> NativeInvocationInfo {
        NativeInvocationInfo::Method(
            NativeMethod::RoyaltyReserve(RoyaltyReserveMethod::Put),
            RENodeId::RoyaltyReserve(self.receiver),
            CallFrameUpdate::empty(),
        )
    }
}

impl NativeExecutable for RoyaltyReservePutInvocation {
    type NativeOutput = ();

    fn execute<Y>(input: Self, system_api: &mut Y) -> Result<((), CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi,
    {
        let node_id = RENodeId::RoyaltyReserve(input.receiver);
        let offset = SubstateOffset::RoyaltyReserve(RoyaltyReserveOffset::RoyaltyReserve);
        let handle = system_api.lock_substate(node_id, offset, LockFlags::MUTABLE)?;

        let mut substate_mut = system_api.get_ref_mut(handle)?;
        let royalty_reserve = substate_mut.royalty_reserve();

        Ok(((), CallFrameUpdate::empty()))
    }
}

impl NativeInvocation for RoyaltyReserveTakeInvocation {
    fn info(&self) -> NativeInvocationInfo {
        NativeInvocationInfo::Method(
            NativeMethod::RoyaltyReserve(RoyaltyReserveMethod::Take),
            RENodeId::RoyaltyReserve(self.receiver),
            CallFrameUpdate::empty(),
        )
    }
}

impl NativeExecutable for RoyaltyReserveTakeInvocation {
    type NativeOutput = ();

    fn execute<Y>(input: Self, system_api: &mut Y) -> Result<((), CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi,
    {
        let node_id = RENodeId::RoyaltyReserve(input.receiver);
        let offset = SubstateOffset::RoyaltyReserve(RoyaltyReserveOffset::RoyaltyReserve);
        let handle = system_api.lock_substate(node_id, offset, LockFlags::MUTABLE)?;

        let mut substate_mut = system_api.get_ref_mut(handle)?;
        let royalty_reserve = substate_mut.royalty_reserve();

        Ok(((), CallFrameUpdate::empty()))
    }
}

impl NativeInvocation for RoyaltyReserveDrainInvocation {
    fn info(&self) -> NativeInvocationInfo {
        NativeInvocationInfo::Method(
            NativeMethod::RoyaltyReserve(RoyaltyReserveMethod::Drain),
            RENodeId::RoyaltyReserve(self.receiver),
            CallFrameUpdate::empty(),
        )
    }
}

impl NativeExecutable for RoyaltyReserveDrainInvocation {
    type NativeOutput = ();

    fn execute<Y>(input: Self, system_api: &mut Y) -> Result<((), CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi,
    {
        let node_id = RENodeId::RoyaltyReserve(input.receiver);
        let offset = SubstateOffset::RoyaltyReserve(RoyaltyReserveOffset::RoyaltyReserve);
        let handle = system_api.lock_substate(node_id, offset, LockFlags::MUTABLE)?;

        let mut substate_mut = system_api.get_ref_mut(handle)?;
        let royalty_reserve = substate_mut.royalty_reserve();

        Ok(((), CallFrameUpdate::empty()))
    }
}

impl RoyaltyReserve {
    pub fn method_auth(method: &RoyaltyReserveMethod) -> Vec<MethodAuthorization> {
        match method {
            // TODO: template, needs fix
            RoyaltyReserveMethod::Take => {
                vec![MethodAuthorization::Protected(HardAuthRule::ProofRule(
                    HardProofRule::Require(HardResourceOrNonFungible::NonFungible(
                        NonFungibleAddress::new(SYSTEM_TOKEN, AuthModule::supervisor_id()),
                    )),
                ))]
            }
            _ => vec![],
        }
    }
}
