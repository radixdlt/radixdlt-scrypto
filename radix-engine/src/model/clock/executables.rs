use crate::engine::{
    deref_and_update, AuthModule, CallFrameUpdate, ExecutableInvocation, Invokable, LockFlags,
    MethodDeref, NativeExecutor, NativeProcedure, REActor, RENode, ResolvedFunction,
    ResolvedMethod, RuntimeError, SystemApi,
};
use crate::model::{
    CurrentTimeRoundedToMinutesSubstate, CurrentTimeRoundedToSecondsSubstate, CurrentTimeSubstate,
    GlobalAddressSubstate, HardAuthRule, HardProofRule, HardResourceOrNonFungible,
    MethodAuthorization, SubstateRefMut,
};
use crate::types::*;
use radix_engine_interface::api::api::EngineApi;
use radix_engine_interface::api::types::{
    ClockFunction, ClockMethod, ClockOffset, GlobalAddress, NativeFunction, NativeMethod, RENodeId,
    SubstateOffset,
};
use radix_engine_interface::data::IndexedScryptoValue;
use radix_engine_interface::model::*;

const SECONDS_TO_MS_FACTOR: u64 = 1000;
const MINUTES_TO_MS_FACTOR: u64 = SECONDS_TO_MS_FACTOR * 60;

#[derive(Debug, Clone, TypeId, Encode, Decode, PartialEq, Eq)]
pub struct Clock {}

impl ExecutableInvocation for ClockCreateInvocation {
    type Exec = NativeExecutor<Self>;

    fn resolve<D: MethodDeref>(
        self,
        _deref: &mut D,
    ) -> Result<(REActor, CallFrameUpdate, Self::Exec), RuntimeError>
    where
        Self: Sized,
    {
        let input = IndexedScryptoValue::from_typed(&self);
        let actor = REActor::Function(ResolvedFunction::Native(NativeFunction::Clock(
            ClockFunction::Create,
        )));
        let call_frame_update = CallFrameUpdate::empty();
        let executor = NativeExecutor(self, input);

        Ok((actor, call_frame_update, executor))
    }
}

impl NativeProcedure for ClockCreateInvocation {
    type Output = SystemAddress;

    fn main<Y>(self, system_api: &mut Y) -> Result<(Self::Output, CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi + Invokable<ScryptoInvocation> + EngineApi<RuntimeError>,
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

pub struct ClockGetCurrentTimeRoundedToMinutesExecutable(RENodeId);

impl ExecutableInvocation for ClockGetCurrentTimeRoundedToMinutesInvocation {
    type Exec = NativeExecutor<ClockGetCurrentTimeRoundedToMinutesExecutable>;

    fn resolve<D: MethodDeref>(
        self,
        deref: &mut D,
    ) -> Result<(REActor, CallFrameUpdate, Self::Exec), RuntimeError>
    where
        Self: Sized,
    {
        let input = IndexedScryptoValue::from_typed(&self);
        let mut call_frame_update = CallFrameUpdate::empty();
        let receiver = RENodeId::Global(GlobalAddress::System(self.receiver));
        let resolved_receiver = deref_and_update(receiver, &mut call_frame_update, deref)?;

        let actor = REActor::Method(
            ResolvedMethod::Native(NativeMethod::Clock(
                ClockMethod::GetCurrentTimeRoundedToMinutes,
            )),
            resolved_receiver,
        );
        let executor = NativeExecutor(
            ClockGetCurrentTimeRoundedToMinutesExecutable(resolved_receiver.receiver),
            input,
        );

        Ok((actor, call_frame_update, executor))
    }
}

impl NativeProcedure for ClockGetCurrentTimeRoundedToMinutesExecutable {
    type Output = u64;

    fn main<Y>(self, system_api: &mut Y) -> Result<(u64, CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi,
    {
        let offset = SubstateOffset::Clock(ClockOffset::CurrentTimeRoundedToMinutes);
        let handle = system_api.lock_substate(self.0, offset, LockFlags::read_only())?;
        let substate_ref = system_api.get_ref(handle)?;
        let substate = substate_ref.current_time_rounded_to_minutes();
        Ok((
            substate.current_time_rounded_to_minutes_ms,
            CallFrameUpdate::empty(),
        ))
    }
}

pub struct ClockSetCurrentTimeExecutable(RENodeId, u64);

impl ExecutableInvocation for ClockSetCurrentTimeInvocation {
    type Exec = NativeExecutor<ClockSetCurrentTimeExecutable>;

    fn resolve<D: MethodDeref>(
        self,
        deref: &mut D,
    ) -> Result<(REActor, CallFrameUpdate, Self::Exec), RuntimeError>
    where
        Self: Sized,
    {
        let input = IndexedScryptoValue::from_typed(&self);
        let mut call_frame_update = CallFrameUpdate::empty();
        let receiver = RENodeId::Global(GlobalAddress::System(self.receiver));
        let resolved_receiver = deref_and_update(receiver, &mut call_frame_update, deref)?;

        let actor = REActor::Method(
            ResolvedMethod::Native(NativeMethod::Clock(ClockMethod::SetCurrentTime)),
            resolved_receiver,
        );
        let executor = NativeExecutor(
            ClockSetCurrentTimeExecutable(resolved_receiver.receiver, self.current_time_ms),
            input,
        );

        Ok((actor, call_frame_update, executor))
    }
}

impl NativeProcedure for ClockSetCurrentTimeExecutable {
    type Output = ();

    fn main<Y>(self, system_api: &mut Y) -> Result<((), CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi,
    {
        let node_id = self.0;

        let current_time_ms = self.1;
        let current_time_rounded_to_seconds =
            (current_time_ms / SECONDS_TO_MS_FACTOR) * SECONDS_TO_MS_FACTOR;
        let current_time_rounded_to_minutes =
            (current_time_ms / MINUTES_TO_MS_FACTOR) * MINUTES_TO_MS_FACTOR;

        update_clock_substate(
            system_api,
            node_id,
            ClockOffset::CurrentTime,
            |substate_ref| substate_ref.current_time().current_time_ms = current_time_ms,
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
