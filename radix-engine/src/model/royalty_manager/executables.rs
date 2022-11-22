use crate::engine::{
    ApplicationError, AuthModule, CallFrameUpdate, Invokable, LockFlags, NativeExecutable,
    NativeInvocation, NativeInvocationInfo, REActor, RENode, ResolvedReceiver, RuntimeError,
    SystemApi,
};
use crate::model::{
    GlobalAddressSubstate, HardAuthRule, HardProofRule, HardResourceOrNonFungible,
    MethodAuthorization, Resource, ResourceOperationError, RoyaltyManagerSubstate,
};
use crate::types::*;
use radix_engine_interface::api::types::{
    GlobalAddress, NativeFunction, NativeMethod, RENodeId, RoyaltyManagerMethod,
    RoyaltyManagerOffset, SubstateOffset,
};
use radix_engine_interface::model::*;

#[derive(Debug, Clone, Eq, PartialEq)]
#[scrypto(TypeId, Encode, Decode)]
pub enum RoyaltyManagerError {
    InvalidRequestData(DecodeError),
    ResourceOperationError(ResourceOperationError),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RoyaltyManager {
    pub info: RoyaltyManagerSubstate,
}

impl NativeInvocation for RoyaltyManagerPutInvocation {
    fn info(&self) -> NativeInvocationInfo {
        NativeInvocationInfo::Method(
            NativeMethod::RoyaltyManager(RoyaltyManagerMethod::Put),
            RENodeId::RoyaltyManager,
            CallFrameUpdate::empty(),
        )
    }
}

impl NativeExecutable for RoyaltyManagerPutInvocation {
    type NativeOutput = ();

    fn execute<Y>(input: Self, system_api: &mut Y) -> Result<((), CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi,
    {
        let node_id = RENodeId::RoyaltyManager;
        let offset = SubstateOffset::RoyaltyManager(RoyaltyManagerOffset::RoyaltyManager);
        let handle = system_api.lock_substate(node_id, offset, LockFlags::MUTABLE)?;

        let bucket: Bucket = system_api
            .drop_node(RENodeId::Bucket(input.bucket.0))?
            .into();
        let mut substate_mut = system_api.get_ref_mut(handle)?;
        let royalty_manager = substate_mut.royalty_manager();
        royalty_manager.put(bucket).map_err(|e| {
            RuntimeError::ApplicationError(ApplicationError::RoyaltyManagerError(
                RoyaltyManagerError::ResourceOperationError(e),
            ))
        })?;

        Ok(((), CallFrameUpdate::empty()))
    }
}

impl NativeInvocation for RoyaltyManagerTakeInvocation {
    fn info(&self) -> NativeInvocationInfo {
        NativeInvocationInfo::Method(
            NativeMethod::RoyaltyManager(RoyaltyManagerMethod::Take),
            RENodeId::RoyaltyManager,
            CallFrameUpdate::empty(),
        )
    }
}

impl NativeExecutable for RoyaltyManagerTakeInvocation {
    type NativeOutput = ();

    fn execute<Y>(input: Self, system_api: &mut Y) -> Result<((), CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi,
    {
        let node_id = RENodeId::RoyaltyManager;
        let offset = SubstateOffset::RoyaltyManager(RoyaltyManagerOffset::RoyaltyManager);
        let handle = system_api.lock_substate(node_id, offset, LockFlags::MUTABLE)?;

        let mut substate_mut = system_api.get_ref_mut(handle)?;
        let royalty_manager = substate_mut.royalty_manager();
        royalty_manager.take(input.amount).map_err(|e| {
            RuntimeError::ApplicationError(ApplicationError::RoyaltyManagerError(
                RoyaltyManagerError::ResourceOperationError(e),
            ))
        })?;

        Ok(((), CallFrameUpdate::empty()))
    }
}

impl RoyaltyManager {
    pub fn method_auth(method: &RoyaltyManagerMethod) -> Vec<MethodAuthorization> {
        match method {
            RoyaltyManagerMethod::Take => {
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
