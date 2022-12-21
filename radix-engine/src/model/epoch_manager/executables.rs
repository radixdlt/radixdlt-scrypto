use crate::engine::{deref_and_update, CallFrameUpdate, ExecutableInvocation, Executor, LockFlags, RENode, ResolvedActor, ResolverApi, RuntimeError, SystemApi, ApplicationError};
use crate::model::{
    AccessRulesChainSubstate, EpochManagerSubstate, GlobalAddressSubstate, HardAuthRule,
    HardProofRule, HardResourceOrNonFungible, MethodAuthorization, ValidatorSetSubstate,
};
use crate::types::*;
use crate::wasm::WasmEngine;
use radix_engine_interface::api::api::EngineApi;
use radix_engine_interface::api::types::{
    EpochManagerFunction, EpochManagerMethod, EpochManagerOffset, GlobalAddress, NativeFunction,
    NativeMethod, RENodeId, SubstateOffset,
};
use radix_engine_interface::model::*;
use radix_engine_interface::modules::auth::AuthAddresses;
use radix_engine_interface::rule;

#[derive(Debug, Clone, Eq, PartialEq, TypeId, Encode, Decode)]
pub enum EpochManagerError {
    InvalidRoundUpdate {
        from: u64,
        to: u64,
    }
}

pub struct EpochManager;

impl<W: WasmEngine> ExecutableInvocation<W> for EpochManagerCreateInvocation {
    type Exec = Self;

    fn resolve<D: ResolverApi<W>>(
        self,
        _deref: &mut D,
    ) -> Result<(ResolvedActor, CallFrameUpdate, Self::Exec), RuntimeError>
    where
        Self: Sized,
    {
        let actor =
            ResolvedActor::function(NativeFunction::EpochManager(EpochManagerFunction::Create));

        let call_frame_update = CallFrameUpdate::empty();

        Ok((actor, call_frame_update, self))
    }
}

impl Executor for EpochManagerCreateInvocation {
    type Output = SystemAddress;

    fn execute<Y>(self, api: &mut Y) -> Result<(Self::Output, CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi + EngineApi<RuntimeError>,
    {
        let underlying_node_id = api.allocate_node_id(RENodeType::EpochManager)?;

        let epoch_manager = EpochManagerSubstate {
            epoch: 0,
            round: 0,
            rounds_per_epoch: self.rounds_per_epoch,
        };

        let validator_set = ValidatorSetSubstate {
            validator_set: self.validator_set,
        };

        let mut access_rules = AccessRules::new();
        access_rules.set_method_access_rule(
            AccessRuleKey::Native(NativeFn::Method(NativeMethod::EpochManager(
                EpochManagerMethod::NextRound,
            ))),
            rule!(require(AuthAddresses::validator_role())),
        );
        access_rules.set_method_access_rule(
            AccessRuleKey::Native(NativeFn::Method(NativeMethod::EpochManager(
                EpochManagerMethod::GetCurrentEpoch,
            ))),
            rule!(allow_all),
        );

        api.create_node(
            underlying_node_id,
            RENode::EpochManager(
                epoch_manager,
                validator_set,
                AccessRulesChainSubstate {
                    access_rules_chain: vec![access_rules],
                },
            ),
        )?;

        let global_node_id = api.allocate_node_id(RENodeType::GlobalEpochManager)?;
        api.create_node(
            global_node_id,
            RENode::Global(GlobalAddressSubstate::EpochManager(
                underlying_node_id.into(),
            )),
        )?;

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

pub struct EpochManagerGetCurrentEpochExecutable(RENodeId);

impl<W: WasmEngine> ExecutableInvocation<W> for EpochManagerGetCurrentEpochInvocation {
    type Exec = EpochManagerGetCurrentEpochExecutable;

    fn resolve<D: ResolverApi<W>>(
        self,
        deref: &mut D,
    ) -> Result<(ResolvedActor, CallFrameUpdate, Self::Exec), RuntimeError>
    where
        Self: Sized,
    {
        let mut call_frame_update = CallFrameUpdate::empty();
        let receiver = RENodeId::Global(GlobalAddress::System(self.receiver));
        let resolved_receiver = deref_and_update(receiver, &mut call_frame_update, deref)?;

        let actor = ResolvedActor::method(
            NativeMethod::EpochManager(EpochManagerMethod::GetCurrentEpoch),
            resolved_receiver,
        );
        let executor = EpochManagerGetCurrentEpochExecutable(resolved_receiver.receiver);

        Ok((actor, call_frame_update, executor))
    }
}

impl Executor for EpochManagerGetCurrentEpochExecutable {
    type Output = u64;

    fn execute<Y>(self, system_api: &mut Y) -> Result<(u64, CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi,
    {
        let offset = SubstateOffset::EpochManager(EpochManagerOffset::EpochManager);
        let handle = system_api.lock_substate(self.0, offset, LockFlags::read_only())?;
        let substate_ref = system_api.get_ref(handle)?;
        let epoch_manager = substate_ref.epoch_manager();
        Ok((epoch_manager.epoch, CallFrameUpdate::empty()))
    }
}

pub struct EpochManagerNextRoundExecutable {
    node_id: RENodeId,
    round: u64,
}

impl<W: WasmEngine> ExecutableInvocation<W> for EpochManagerNextRoundInvocation {
    type Exec = EpochManagerNextRoundExecutable;

    fn resolve<D: ResolverApi<W>>(
        self,
        deref: &mut D,
    ) -> Result<(ResolvedActor, CallFrameUpdate, Self::Exec), RuntimeError>
    where
        Self: Sized,
    {
        let mut call_frame_update = CallFrameUpdate::empty();
        let receiver = RENodeId::Global(GlobalAddress::System(self.receiver));
        let resolved_receiver = deref_and_update(receiver, &mut call_frame_update, deref)?;

        let actor = ResolvedActor::method(
            NativeMethod::EpochManager(EpochManagerMethod::NextRound),
            resolved_receiver,
        );
        let executor = EpochManagerNextRoundExecutable {
            node_id: resolved_receiver.receiver,
            round: self.round,
        };

        Ok((actor, call_frame_update, executor))
    }
}

impl Executor for EpochManagerNextRoundExecutable {
    type Output = ();

    fn execute<Y>(self, system_api: &mut Y) -> Result<((), CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi,
    {
        let offset = SubstateOffset::EpochManager(EpochManagerOffset::EpochManager);
        let handle = system_api.lock_substate(self.node_id, offset, LockFlags::MUTABLE)?;
        let mut substate_mut = system_api.get_ref_mut(handle)?;
        let epoch_manager = substate_mut.epoch_manager();

        if self.round <= epoch_manager.round {
            return Err(RuntimeError::ApplicationError(ApplicationError::EpochManagerError(EpochManagerError::InvalidRoundUpdate {
                from: epoch_manager.round,
                to: self.round,
            })))
        }

        if self.round >= epoch_manager.rounds_per_epoch {
            epoch_manager.epoch = epoch_manager.epoch + 1;
            epoch_manager.round = 0;
        } else {
            epoch_manager.round = self.round;
        }

        Ok(((), CallFrameUpdate::empty()))
    }
}

impl EpochManager {
    pub fn function_auth(func: &EpochManagerFunction) -> Vec<MethodAuthorization> {
        match func {
            EpochManagerFunction::Create => {
                vec![MethodAuthorization::Protected(HardAuthRule::ProofRule(
                    HardProofRule::Require(HardResourceOrNonFungible::NonFungible(
                        AuthAddresses::system_role(),
                    )),
                ))]
            }
        }
    }
}
