use crate::engine::{
    AuthModule, CallFrameUpdate, Invokable, LockFlags, NativeExecutable, NativeInvocation,
    NativeInvocationInfo, REActor, RENode, ResolvedReceiver, RuntimeError, SystemApi,
};
use crate::model::{
    CurrentTimeRoundedToMinutesSubstate, CurrentTimeRoundedToSecondsSubstate, CurrentTimeSubstate,
    GlobalAddressSubstate, HardAuthRule, HardProofRule, HardResourceOrNonFungible,
    MethodAuthorization, SubstateRefMut,
};
use crate::types::*;
use radix_engine_interface::api::types::{
    ClockFunction, ClockMethod, ClockOffset, GlobalAddress, NativeFunction, NativeMethod, RENodeId,
    SubstateOffset,
};
use radix_engine_interface::model::*;

const SECONDS_TO_MS_FACTOR: u64 = 1000;
const MINUTES_TO_MS_FACTOR: u64 = SECONDS_TO_MS_FACTOR * 60;

#[derive(Debug, Clone, TypeId, Encode, Decode, PartialEq, Eq)]
pub struct Clock {}

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
            CurrentTimeSubstate { current_time_ms: 0 },
            CurrentTimeRoundedToSecondsSubstate {
                current_time_rounded_to_seconds_ms: 0,
            },
            CurrentTimeRoundedToMinutesSubstate {
                current_time_rounded_to_minutes_ms: 0,
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

impl NativeExecutable for ClockGetCurrentTimeRoundedToMinutesInvocation {
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
        let offset = SubstateOffset::Clock(ClockOffset::CurrentTimeRoundedToMinutes);
        let handle = system_api.lock_substate(node_id, offset, LockFlags::read_only())?;
        let substate_ref = system_api.get_ref(handle)?;
        let substate = substate_ref.current_time_rounded_to_minutes();

        Ok((
            substate.current_time_rounded_to_minutes_ms,
            CallFrameUpdate::empty(),
        ))
    }
}

impl NativeInvocation for ClockGetCurrentTimeRoundedToMinutesInvocation {
    fn info(&self) -> NativeInvocationInfo {
        NativeInvocationInfo::Method(
            NativeMethod::Clock(ClockMethod::GetCurrentTimeRoundedToMinutes),
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
        let current_time_rounded_to_seconds =
            (input.current_time_ms / SECONDS_TO_MS_FACTOR) * SECONDS_TO_MS_FACTOR;
        let current_time_rounded_to_minutes =
            (input.current_time_ms / MINUTES_TO_MS_FACTOR) * MINUTES_TO_MS_FACTOR;

        // TODO: Remove this hack and get resolved receiver in a better way
        let node_id = match system_api.get_actor() {
            REActor::Method(_, ResolvedReceiver { receiver, .. }) => *receiver,
            _ => panic!("Unexpected"),
        };

        update_clock_substate(
            system_api,
            node_id,
            ClockOffset::CurrentTime,
            |substate_ref| substate_ref.current_time().current_time_ms = input.current_time_ms,
        )?;

        update_clock_substate(
            system_api,
            node_id,
            ClockOffset::CurrentTimeRoundedToSeconds,
            |substate_ref| {
                substate_ref
                    .current_time_rounded_to_seconds()
                    .current_time_rounded_to_seconds_ms = current_time_rounded_to_seconds
            },
        )?;

        update_clock_substate(
            system_api,
            node_id,
            ClockOffset::CurrentTimeRoundedToMinutes,
            |substate_ref| {
                substate_ref
                    .current_time_rounded_to_minutes()
                    .current_time_rounded_to_minutes_ms = current_time_rounded_to_minutes
            },
        )?;

        Ok(((), CallFrameUpdate::empty()))
    }
}

fn update_clock_substate<'a, Y, F>(
    system_api: &'a mut Y,
    node_id: RENodeId,
    clock_offset: ClockOffset,
    fun: F,
) -> Result<(), RuntimeError>
where
    Y: SystemApi,
    F: FnOnce(&mut SubstateRefMut<'a>) -> (),
{
    let offset = SubstateOffset::Clock(clock_offset);
    let handle = system_api.lock_substate(node_id, offset, LockFlags::MUTABLE)?;
    let mut substate_ref = system_api.get_ref_mut(handle)?;
    Ok(fun(&mut substate_ref))
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
