use crate::engine::{
    AuthModule, CallFrameUpdate, Invokable, LockFlags, NativeExecutable, NativeInvocation,
    NativeInvocationInfo, REActor, RENode, ResolvedReceiver, RuntimeError, SystemApi,
};
use crate::model::{
    EpochManagerSubstate, GlobalAddressSubstate, HardAuthRule, HardProofRule,
    HardResourceOrNonFungible, MethodAuthorization,
};
use crate::types::*;
use radix_engine_interface::engine::types::{
    EpochManagerFunction, EpochManagerMethod, EpochManagerOffset, GlobalAddress, NativeFunction,
    NativeMethod, RENodeId, SubstateOffset,
};
use radix_engine_interface::model::*;

#[derive(Debug, Clone, Eq, PartialEq, TypeId, Encode, Decode)]
pub enum EpochManagerError {
    InvalidRequestData(DecodeError),
}

#[derive(Debug, Clone, TypeId, Encode, Decode, PartialEq, Eq)]
pub struct EpochManager {
    pub info: EpochManagerSubstate,
}

impl NativeExecutable for EpochManagerCreateInvocation {
    type NativeOutput = SystemAddress;

    fn execute<Y>(
        _invocation: Self,
        system_api: &mut Y,
    ) -> Result<(SystemAddress, CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi + Invokable<ScryptoInvocation>,
    {
        let node_id =
            system_api.create_node(RENode::EpochManager(EpochManagerSubstate { epoch: 0 }))?;

        let global_node_id = system_api.create_node(RENode::Global(
            GlobalAddressSubstate::System(node_id.into()),
        ))?;

        let system_address: SystemAddress = global_node_id.into();
        let mut node_refs_to_copy = HashSet::new();
        node_refs_to_copy.insert(global_node_id);

        let update = CallFrameUpdate {
            node_refs_to_copy,
            nodes_to_move: vec![],
        };

        Ok((system_address, update))
    }
}

impl NativeInvocation for EpochManagerCreateInvocation {
    fn info(&self) -> NativeInvocationInfo {
        NativeInvocationInfo::Function(
            NativeFunction::EpochManager(EpochManagerFunction::Create),
            CallFrameUpdate::empty(),
        )
    }
}

impl NativeExecutable for EpochManagerGetCurrentEpochInvocation {
    type NativeOutput = u64;

    fn execute<Y>(_input: Self, system_api: &mut Y) -> Result<(u64, CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi,
    {
        // TODO: Remove this hack and get resolved receiver in a better way
        let node_id = match system_api.get_actor() {
            REActor::Method(_, ResolvedReceiver { receiver, .. }) => *receiver,
            _ => panic!("Unexpected"),
        };
        let offset = SubstateOffset::EpochManager(EpochManagerOffset::EpochManager);
        let handle = system_api.lock_substate(node_id, offset, LockFlags::read_only())?;

        let substate_ref = system_api.get_ref(handle)?;
        let system = substate_ref.epoch_manager();

        Ok((system.epoch, CallFrameUpdate::empty()))
    }
}

impl NativeInvocation for EpochManagerGetCurrentEpochInvocation {
    fn info(&self) -> NativeInvocationInfo {
        NativeInvocationInfo::Method(
            NativeMethod::EpochManager(EpochManagerMethod::GetCurrentEpoch),
            RENodeId::Global(GlobalAddress::System(self.receiver)),
            CallFrameUpdate::empty(),
        )
    }
}

impl NativeExecutable for EpochManagerSetEpochInvocation {
    type NativeOutput = ();

    fn execute<Y>(input: Self, system_api: &mut Y) -> Result<((), CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi,
    {
        // TODO: Remove this hack and get resolved receiver in a better way
        let node_id = match system_api.get_actor() {
            REActor::Method(_, ResolvedReceiver { receiver, .. }) => *receiver,
            _ => panic!("Unexpected"),
        };
        let offset = SubstateOffset::EpochManager(EpochManagerOffset::EpochManager);
        let handle = system_api.lock_substate(node_id, offset, LockFlags::MUTABLE)?;

        let mut substate_mut = system_api.get_ref_mut(handle)?;
        substate_mut.epoch_manager().epoch = input.epoch;

        Ok(((), CallFrameUpdate::empty()))
    }
}

impl NativeInvocation for EpochManagerSetEpochInvocation {
    fn info(&self) -> NativeInvocationInfo {
        NativeInvocationInfo::Method(
            NativeMethod::EpochManager(EpochManagerMethod::SetEpoch),
            RENodeId::Global(GlobalAddress::System(self.receiver)),
            CallFrameUpdate::empty(),
        )
    }
}

impl EpochManager {
    pub fn function_auth(func: &EpochManagerFunction) -> Vec<MethodAuthorization> {
        match func {
            EpochManagerFunction::Create => {
                vec![MethodAuthorization::Protected(HardAuthRule::ProofRule(
                    HardProofRule::Require(HardResourceOrNonFungible::NonFungible(
                        NonFungibleAddress::new(SYSTEM_TOKEN, AuthModule::system_id()),
                    )),
                ))]
            }
        }
    }

    pub fn method_auth(method: &EpochManagerMethod) -> Vec<MethodAuthorization> {
        match method {
            EpochManagerMethod::SetEpoch => {
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
