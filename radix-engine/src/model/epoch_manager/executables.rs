use crate::engine::{
    deref_and_update, AuthModule, CallFrameUpdate, ExecutableInvocation, Invokable, LockFlags,
    MethodDeref, NativeExecutor, NativeProcedure, REActor, RENode, ResolvedFunction,
    ResolvedMethod, RuntimeError, SystemApi,
};
use crate::model::{
    AccessRulesSubstate, EpochManagerSubstate, GlobalAddressSubstate, HardAuthRule, HardProofRule,
    HardResourceOrNonFungible, MethodAuthorization,
};
use crate::types::*;
use radix_engine_interface::api::api::EngineApi;
use radix_engine_interface::api::types::{
    EpochManagerFunction, EpochManagerMethod, EpochManagerOffset, GlobalAddress, NativeFunction,
    NativeMethod, RENodeId, SubstateOffset,
};
use radix_engine_interface::model::*;
use scrypto::access_rule_node;
use scrypto::rule;

#[derive(Debug, Clone, Eq, PartialEq, TypeId, Encode, Decode)]
pub enum EpochManagerError {
    InvalidRequestData(DecodeError),
}

#[derive(Debug, Clone, TypeId, Encode, Decode, PartialEq, Eq)]
pub struct EpochManager {
    pub info: EpochManagerSubstate,
}

impl ExecutableInvocation for EpochManagerCreateInvocation {
    type Exec = NativeExecutor<Self>;

    fn resolve<D: MethodDeref>(
        self,
        _deref: &mut D,
    ) -> Result<(REActor, CallFrameUpdate, Self::Exec), RuntimeError>
    where
        Self: Sized,
    {
        let actor = REActor::Function(ResolvedFunction::Native(NativeFunction::EpochManager(
            EpochManagerFunction::Create,
        )));
        let call_frame_update = CallFrameUpdate::empty();
        let executor = NativeExecutor(self);

        Ok((actor, call_frame_update, executor))
    }
}

impl NativeProcedure for EpochManagerCreateInvocation {
    type Output = SystemAddress;

    fn main<Y>(self, api: &mut Y) -> Result<(Self::Output, CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi + Invokable<ScryptoInvocation> + EngineApi<RuntimeError>,
    {
        let underlying_node_id = api.allocate_node_id(RENodeType::EpochManager)?;

        let epoch_manager = EpochManagerSubstate { epoch: 0 };

        let auth_non_fungible = NonFungibleAddress::new(SYSTEM_TOKEN, AuthModule::supervisor_id());
        let mut access_rules = AccessRules::new();
        access_rules.set_method_access_rule(
            AccessRuleKey::Native(NativeFn::Method(NativeMethod::EpochManager(
                EpochManagerMethod::SetEpoch,
            ))),
            rule!(require(auth_non_fungible)),
        );
        access_rules.set_method_access_rule(
            AccessRuleKey::Native(NativeFn::Method(NativeMethod::EpochManager(
                EpochManagerMethod::GetCurrentEpoch,
            ))),
            rule!(allow_all),
        );

        let access_rules_substate = AccessRulesSubstate {
            access_rules: vec![access_rules],
        };

        api.create_node(
            underlying_node_id,
            RENode::EpochManager(epoch_manager, access_rules_substate),
        )?;

        let global_node_id = api.allocate_node_id(RENodeType::GlobalEpochManager)?;
        api.create_node(
            global_node_id,
            RENode::Global(GlobalAddressSubstate::System(underlying_node_id.into())),
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

impl ExecutableInvocation for EpochManagerGetCurrentEpochInvocation {
    type Exec = NativeExecutor<EpochManagerGetCurrentEpochExecutable>;

    fn resolve<D: MethodDeref>(
        self,
        deref: &mut D,
    ) -> Result<(REActor, CallFrameUpdate, Self::Exec), RuntimeError>
    where
        Self: Sized,
    {
        let mut call_frame_update = CallFrameUpdate::empty();
        let receiver = RENodeId::Global(GlobalAddress::System(self.receiver));
        let resolved_receiver = deref_and_update(receiver, &mut call_frame_update, deref)?;

        let actor = REActor::Method(
            ResolvedMethod::Native(NativeMethod::EpochManager(
                EpochManagerMethod::GetCurrentEpoch,
            )),
            resolved_receiver,
        );
        let executor = NativeExecutor(EpochManagerGetCurrentEpochExecutable(
            resolved_receiver.receiver,
        ));

        Ok((actor, call_frame_update, executor))
    }
}

impl NativeProcedure for EpochManagerGetCurrentEpochExecutable {
    type Output = u64;

    fn main<Y>(self, system_api: &mut Y) -> Result<(u64, CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi,
    {
        let offset = SubstateOffset::EpochManager(EpochManagerOffset::EpochManager);
        let handle = system_api.lock_substate(self.0, offset, LockFlags::read_only())?;
        let substate_ref = system_api.get_ref(handle)?;
        let system = substate_ref.epoch_manager();
        Ok((system.epoch, CallFrameUpdate::empty()))
    }
}

pub struct EpochManagerSetEpochExecutable(RENodeId, u64);

impl ExecutableInvocation for EpochManagerSetEpochInvocation {
    type Exec = NativeExecutor<EpochManagerSetEpochExecutable>;

    fn resolve<D: MethodDeref>(
        self,
        deref: &mut D,
    ) -> Result<(REActor, CallFrameUpdate, Self::Exec), RuntimeError>
    where
        Self: Sized,
    {
        let mut call_frame_update = CallFrameUpdate::empty();
        let receiver = RENodeId::Global(GlobalAddress::System(self.receiver));
        let resolved_receiver = deref_and_update(receiver, &mut call_frame_update, deref)?;

        let actor = REActor::Method(
            ResolvedMethod::Native(NativeMethod::EpochManager(EpochManagerMethod::SetEpoch)),
            resolved_receiver,
        );
        let executor = NativeExecutor(EpochManagerSetEpochExecutable(
            resolved_receiver.receiver,
            self.epoch,
        ));

        Ok((actor, call_frame_update, executor))
    }
}

impl NativeProcedure for EpochManagerSetEpochExecutable {
    type Output = ();

    fn main<Y>(self, system_api: &mut Y) -> Result<((), CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi,
    {
        let offset = SubstateOffset::EpochManager(EpochManagerOffset::EpochManager);
        let handle = system_api.lock_substate(self.0, offset, LockFlags::MUTABLE)?;
        let mut substate_mut = system_api.get_ref_mut(handle)?;
        substate_mut.epoch_manager().epoch = self.1;
        Ok(((), CallFrameUpdate::empty()))
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
}
