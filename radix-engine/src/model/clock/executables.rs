use crate::engine::{
    AuthModule, CallFrameUpdate, Invokable, LockFlags, NativeExecutable, NativeInvocation,
    NativeInvocationInfo, REActor, RENode, ResolvedReceiver, RuntimeError, SystemApi,
};
use crate::model::{
    CurrentTimeInMillisSubstate, CurrentTimeInMinutesSubstate, CurrentTimeInSecondsSubstate,
    GlobalAddressSubstate, HardAuthRule, HardProofRule, HardResourceOrNonFungible,
    MethodAuthorization,
};
use crate::types::*;
use radix_engine_interface::api::types::{
    ClockFunction, ClockMethod, ClockOffset, GlobalAddress, NativeFunction, NativeMethod, RENodeId,
    SubstateOffset,
};
use radix_engine_interface::model::*;

const SECONDS_TO_MILLIS_FACTOR: u64 = 1000;
const MINUTES_TO_MILLIS_FACTOR: u64 = SECONDS_TO_MILLIS_FACTOR * 60;

#[derive(Debug, Clone, Eq, PartialEq, TypeId, Encode, Decode)]
pub enum ClockError {
    InvalidRequestData(DecodeError),
}

#[derive(Debug, Clone, TypeId, Encode, Decode, PartialEq, Eq)]
pub struct Clock {
    pub current_time_in_minutes_substate: CurrentTimeInMinutesSubstate,
}

impl NativeExecutable for ClockCreateInvocation {
    type NativeOutput = SystemAddress;

    fn execute<Y>(
        _invocation: Self,
        system_api: &mut Y,
    ) -> Result<(SystemAddress, CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi + Invokable<ScryptoInvocation>,
    {
        let node_id = system_api.create_node(RENode::Clock(
            CurrentTimeInMillisSubstate {
                current_time_in_millis: 0,
            },
            CurrentTimeInSecondsSubstate {
                current_time_in_seconds: 0,
            },
            CurrentTimeInMinutesSubstate {
                current_time_in_minutes: 0,
            },
        ))?;

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

impl NativeInvocation for ClockCreateInvocation {
    fn info(&self) -> NativeInvocationInfo {
        NativeInvocationInfo::Function(
            NativeFunction::Clock(ClockFunction::Create),
            CallFrameUpdate::empty(),
        )
    }
}

impl NativeExecutable for ClockGetCurrentTimeInMinutesInvocation {
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
        let offset = SubstateOffset::Clock(ClockOffset::CurrentTimeInMinutes);
        let handle = system_api.lock_substate(node_id, offset, LockFlags::read_only())?;
        let substate_ref = system_api.get_ref(handle)?;
        let substate = substate_ref.current_time_in_minutes();

        Ok((substate.current_time_in_minutes, CallFrameUpdate::empty()))
    }
}

impl NativeInvocation for ClockGetCurrentTimeInMinutesInvocation {
    fn info(&self) -> NativeInvocationInfo {
        NativeInvocationInfo::Method(
            NativeMethod::Clock(ClockMethod::GetCurrentTimeToMinutePrecision),
            RENodeId::Global(GlobalAddress::System(self.receiver)),
            CallFrameUpdate::empty(),
        )
    }
}

impl NativeExecutable for ClockSetCurrentTimeInvocation {
    type NativeOutput = ();

    fn execute<Y>(input: Self, system_api: &mut Y) -> Result<((), CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi,
    {
        let current_time_in_seconds = input.current_time_millis / SECONDS_TO_MILLIS_FACTOR;
        let current_time_in_minutes = input.current_time_millis / MINUTES_TO_MILLIS_FACTOR;

        // TODO: Remove this hack and get resolved receiver in a better way
        let node_id = match system_api.get_actor() {
            REActor::Method(_, ResolvedReceiver { receiver, .. }) => *receiver,
            _ => panic!("Unexpected"),
        };

        let millis_offset = SubstateOffset::Clock(ClockOffset::CurrentTimeInMillis);
        let millis_handle = system_api.lock_substate(node_id, millis_offset, LockFlags::MUTABLE)?;
        let mut millis_substate_mut = system_api.get_ref_mut(millis_handle)?;
        millis_substate_mut
            .current_time_in_millis()
            .current_time_in_millis = input.current_time_millis;

        let seconds_offset = SubstateOffset::Clock(ClockOffset::CurrentTimeInSeconds);
        let seconds_handle =
            system_api.lock_substate(node_id, seconds_offset, LockFlags::MUTABLE)?;
        let mut seconds_substate_mut = system_api.get_ref_mut(seconds_handle)?;
        seconds_substate_mut
            .current_time_in_seconds()
            .current_time_in_seconds = current_time_in_seconds;

        let minutes_offset = SubstateOffset::Clock(ClockOffset::CurrentTimeInMinutes);
        let minutes_handle =
            system_api.lock_substate(node_id, minutes_offset, LockFlags::MUTABLE)?;
        let mut minutes_substate_mut = system_api.get_ref_mut(minutes_handle)?;
        minutes_substate_mut
            .current_time_in_minutes()
            .current_time_in_minutes = current_time_in_minutes;

        Ok(((), CallFrameUpdate::empty()))
    }
}

impl NativeInvocation for ClockSetCurrentTimeInvocation {
    fn info(&self) -> NativeInvocationInfo {
        NativeInvocationInfo::Method(
            NativeMethod::Clock(ClockMethod::SetCurrentTime),
            RENodeId::Global(GlobalAddress::System(self.receiver)),
            CallFrameUpdate::empty(),
        )
    }
}

impl Clock {
    pub fn function_auth(func: &ClockFunction) -> Vec<MethodAuthorization> {
        match func {
            ClockFunction::Create => {
                vec![MethodAuthorization::Protected(HardAuthRule::ProofRule(
                    HardProofRule::Require(HardResourceOrNonFungible::NonFungible(
                        NonFungibleAddress::new(SYSTEM_TOKEN, AuthModule::system_id()),
                    )),
                ))]
            }
        }
    }

    pub fn method_auth(method: &ClockMethod) -> Vec<MethodAuthorization> {
        match method {
            ClockMethod::SetCurrentTime => {
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
