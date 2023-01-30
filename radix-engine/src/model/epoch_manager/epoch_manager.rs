use crate::engine::{
    deref_and_update, ApplicationError, CallFrameUpdate, ExecutableInvocation, Executor, LockFlags,
    RENodeInit, RENodeModuleInit, ResolvedActor, ResolverApi, RuntimeError, SystemApi,
};
use crate::model::{
    AccessRulesChainSubstate, GlobalAddressSubstate, HardAuthRule, HardProofRule,
    HardResourceOrNonFungible, MethodAuthorization, ValidatorCreator,
};
use crate::types::*;
use crate::wasm::WasmEngine;
use native_sdk::resource::SysBucket;
use radix_engine_interface::api::types::{
    EpochManagerFn, EpochManagerOffset, GlobalAddress, NativeFn, RENodeId, SubstateOffset,
};
use radix_engine_interface::api::{EngineApi, InvokableModel};
use radix_engine_interface::model::*;
use radix_engine_interface::modules::auth::AuthAddresses;
use radix_engine_interface::rule;

#[derive(Debug, Clone, PartialEq, Eq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct EpochManagerSubstate {
    pub address: ComponentAddress, // TODO: Does it make sense for this to be stored here?
    pub epoch: u64,
    pub round: u64,

    // TODO: Move configuration to an immutable substate
    pub rounds_per_epoch: u64,
    pub num_unstake_epochs: u64,
}

#[derive(
    Debug, Clone, PartialEq, Eq, Ord, PartialOrd, ScryptoCategorize, ScryptoEncode, ScryptoDecode,
)]
pub struct Validator {
    pub key: EcdsaSecp256k1PublicKey,
    pub stake: Decimal,
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct ValidatorSetSubstate {
    pub validator_set: BTreeMap<ComponentAddress, Validator>,
    pub epoch: u64,
}

#[derive(Debug, Clone, Eq, PartialEq, Categorize, Encode, Decode)]
pub enum EpochManagerError {
    InvalidRoundUpdate { from: u64, to: u64 },
}

pub struct EpochManager;

impl ExecutableInvocation for EpochManagerCreateInvocation {
    type Exec = Self;

    fn resolve<D: ResolverApi>(
        self,
        _deref: &mut D,
    ) -> Result<(ResolvedActor, CallFrameUpdate, Self::Exec), RuntimeError>
    where
        Self: Sized,
    {
        let actor = ResolvedActor::function(NativeFn::EpochManager(EpochManagerFn::Create));

        let mut call_frame_update =
            CallFrameUpdate::copy_ref(RENodeId::Global(GlobalAddress::Resource(RADIX_TOKEN)));

        // TODO: Clean this up, this is currently required in order to be able to call the scrypto account component
        call_frame_update.add_ref(RENodeId::Global(GlobalAddress::Component(EPOCH_MANAGER)));
        call_frame_update.add_ref(RENodeId::Global(GlobalAddress::Component(CLOCK)));
        call_frame_update.add_ref(RENodeId::Global(GlobalAddress::Resource(
            ECDSA_SECP256K1_TOKEN,
        )));
        call_frame_update.add_ref(RENodeId::Global(GlobalAddress::Resource(
            EDDSA_ED25519_TOKEN,
        )));
        call_frame_update.add_ref(RENodeId::Global(GlobalAddress::Package(ACCOUNT_PACKAGE)));

        for (bucket, account_address) in self.validator_set.values() {
            call_frame_update
                .nodes_to_move
                .push(RENodeId::Bucket(bucket.0));
            call_frame_update.add_ref(RENodeId::Global(GlobalAddress::Component(*account_address)));
        }

        Ok((actor, call_frame_update, self))
    }
}

// TODO: Cleanup once native accounts implemented
#[derive(Debug, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct AccountDepositInput {
    bucket: Bucket,
}

impl Executor for EpochManagerCreateInvocation {
    type Output = ComponentAddress;

    fn execute<Y, W: WasmEngine>(
        self,
        api: &mut Y,
    ) -> Result<(Self::Output, CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi + EngineApi<RuntimeError> + InvokableModel<RuntimeError>,
    {
        let underlying_node_id = api.allocate_node_id(RENodeType::EpochManager)?;
        let global_node_id = RENodeId::Global(GlobalAddress::Component(
            ComponentAddress::EpochManager(self.component_address),
        ));

        let epoch_manager = EpochManagerSubstate {
            address: global_node_id.into(),
            epoch: self.initial_epoch,
            round: 0,
            rounds_per_epoch: self.rounds_per_epoch,
            num_unstake_epochs: self.num_unstake_epochs,
        };

        let mut validator_set = BTreeMap::new();

        for (key, (initial_stake, account_address)) in self.validator_set {
            let stake = initial_stake.sys_amount(api)?;
            let (address, lp_bucket) = ValidatorCreator::create_with_initial_stake(
                global_node_id.into(),
                key,
                initial_stake,
                true,
                api,
            )?;
            let validator = Validator { key, stake };
            validator_set.insert(address, validator);
            api.invoke(ScryptoInvocation {
                package_address: ACCOUNT_PACKAGE,
                blueprint_name: "Account".to_string(),
                fn_name: "deposit".to_string(),
                receiver: Some(ScryptoReceiver::Global(account_address)),
                args: args!(lp_bucket),
            })?;
        }

        let current_validator_set = ValidatorSetSubstate {
            epoch: self.initial_epoch,
            validator_set: validator_set.clone(),
        };

        let preparing_validator_set = ValidatorSetSubstate {
            epoch: self.initial_epoch + 1,
            validator_set,
        };

        let mut access_rules = AccessRules::new();
        access_rules.set_method_access_rule(
            AccessRuleKey::Native(NativeFn::EpochManager(EpochManagerFn::NextRound)),
            rule!(require(AuthAddresses::validator_role())),
        );
        access_rules.set_method_access_rule(
            AccessRuleKey::Native(NativeFn::EpochManager(EpochManagerFn::GetCurrentEpoch)),
            rule!(allow_all),
        );
        access_rules.set_method_access_rule(
            AccessRuleKey::Native(NativeFn::EpochManager(EpochManagerFn::CreateValidator)),
            rule!(allow_all),
        );
        let non_fungible_local_id = NonFungibleLocalId::Bytes(
            scrypto_encode(&PackageIdentifier::Native(NativePackage::EpochManager)).unwrap(),
        );
        let non_fungible_global_id = NonFungibleGlobalId::new(PACKAGE_TOKEN, non_fungible_local_id);
        access_rules.set_method_access_rule(
            AccessRuleKey::Native(NativeFn::EpochManager(EpochManagerFn::UpdateValidator)),
            rule!(require(non_fungible_global_id)),
        );
        access_rules.set_method_access_rule(
            AccessRuleKey::Native(NativeFn::EpochManager(EpochManagerFn::SetEpoch)),
            rule!(require(AuthAddresses::system_role())), // Set epoch only used for debugging
        );

        let mut node_modules = BTreeMap::new();
        node_modules.insert(
            NodeModuleId::AccessRules,
            RENodeModuleInit::AccessRulesChain(AccessRulesChainSubstate {
                access_rules_chain: vec![access_rules],
            }),
        );

        api.create_node(
            underlying_node_id,
            RENodeInit::EpochManager(
                epoch_manager,
                current_validator_set,
                preparing_validator_set,
            ),
            node_modules,
        )?;

        api.create_node(
            global_node_id,
            RENodeInit::Global(GlobalAddressSubstate::EpochManager(
                underlying_node_id.into(),
            )),
            BTreeMap::new(),
        )?;

        let component_address: ComponentAddress = global_node_id.into();
        let mut node_refs_to_copy = HashSet::new();
        node_refs_to_copy.insert(global_node_id);

        let update = CallFrameUpdate {
            node_refs_to_copy,
            nodes_to_move: vec![],
        };

        Ok((component_address, update))
    }
}

pub struct EpochManagerGetCurrentEpochExecutable(RENodeId);

impl ExecutableInvocation for EpochManagerGetCurrentEpochInvocation {
    type Exec = EpochManagerGetCurrentEpochExecutable;

    fn resolve<D: ResolverApi>(
        self,
        deref: &mut D,
    ) -> Result<(ResolvedActor, CallFrameUpdate, Self::Exec), RuntimeError>
    where
        Self: Sized,
    {
        let mut call_frame_update = CallFrameUpdate::empty();
        let receiver = RENodeId::Global(GlobalAddress::Component(self.receiver));
        let resolved_receiver = deref_and_update(receiver, &mut call_frame_update, deref)?;

        let actor = ResolvedActor::method(
            NativeFn::EpochManager(EpochManagerFn::GetCurrentEpoch),
            resolved_receiver,
        );
        let executor = EpochManagerGetCurrentEpochExecutable(resolved_receiver.receiver);

        Ok((actor, call_frame_update, executor))
    }
}

impl Executor for EpochManagerGetCurrentEpochExecutable {
    type Output = u64;

    fn execute<Y, W: WasmEngine>(
        self,
        system_api: &mut Y,
    ) -> Result<(u64, CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi,
    {
        let offset = SubstateOffset::EpochManager(EpochManagerOffset::EpochManager);
        let handle =
            system_api.lock_substate(self.0, NodeModuleId::SELF, offset, LockFlags::read_only())?;
        let substate_ref = system_api.get_ref(handle)?;
        let epoch_manager = substate_ref.epoch_manager();
        Ok((epoch_manager.epoch, CallFrameUpdate::empty()))
    }
}

pub struct EpochManagerNextRoundExecutable {
    node_id: RENodeId,
    round: u64,
}

impl ExecutableInvocation for EpochManagerNextRoundInvocation {
    type Exec = EpochManagerNextRoundExecutable;

    fn resolve<D: ResolverApi>(
        self,
        deref: &mut D,
    ) -> Result<(ResolvedActor, CallFrameUpdate, Self::Exec), RuntimeError>
    where
        Self: Sized,
    {
        let mut call_frame_update = CallFrameUpdate::empty();
        let receiver = RENodeId::Global(GlobalAddress::Component(self.receiver));
        let resolved_receiver = deref_and_update(receiver, &mut call_frame_update, deref)?;

        let actor = ResolvedActor::method(
            NativeFn::EpochManager(EpochManagerFn::NextRound),
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

    fn execute<Y, W: WasmEngine>(
        self,
        system_api: &mut Y,
    ) -> Result<((), CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi,
    {
        let offset = SubstateOffset::EpochManager(EpochManagerOffset::EpochManager);
        let mgr_handle = system_api.lock_substate(
            self.node_id,
            NodeModuleId::SELF,
            offset,
            LockFlags::MUTABLE,
        )?;
        let mut substate_mut = system_api.get_ref_mut(mgr_handle)?;
        let epoch_manager = substate_mut.epoch_manager();

        if self.round <= epoch_manager.round {
            return Err(RuntimeError::ApplicationError(
                ApplicationError::EpochManagerError(EpochManagerError::InvalidRoundUpdate {
                    from: epoch_manager.round,
                    to: self.round,
                }),
            ));
        }

        if self.round >= epoch_manager.rounds_per_epoch {
            let offset = SubstateOffset::EpochManager(EpochManagerOffset::PreparingValidatorSet);
            let handle = system_api.lock_substate(
                self.node_id,
                NodeModuleId::SELF,
                offset,
                LockFlags::MUTABLE,
            )?;
            let mut substate_mut = system_api.get_ref_mut(handle)?;
            let preparing_validator_set = substate_mut.validator_set();
            let prepared_epoch = preparing_validator_set.epoch;
            let next_validator_set = preparing_validator_set.validator_set.clone();
            preparing_validator_set.epoch = prepared_epoch + 1;

            let mut substate_mut = system_api.get_ref_mut(mgr_handle)?;
            let epoch_manager = substate_mut.epoch_manager();
            epoch_manager.epoch = prepared_epoch;
            epoch_manager.round = 0;

            let offset = SubstateOffset::EpochManager(EpochManagerOffset::CurrentValidatorSet);
            let handle = system_api.lock_substate(
                self.node_id,
                NodeModuleId::SELF,
                offset,
                LockFlags::MUTABLE,
            )?;
            let mut substate_mut = system_api.get_ref_mut(handle)?;
            let validator_set = substate_mut.validator_set();
            validator_set.epoch = prepared_epoch;
            validator_set.validator_set = next_validator_set;
        } else {
            epoch_manager.round = self.round;
        }

        Ok(((), CallFrameUpdate::empty()))
    }
}

pub struct EpochManagerSetEpochExecutable(RENodeId, u64);

impl ExecutableInvocation for EpochManagerSetEpochInvocation {
    type Exec = EpochManagerSetEpochExecutable;

    fn resolve<D: ResolverApi>(
        self,
        deref: &mut D,
    ) -> Result<(ResolvedActor, CallFrameUpdate, Self::Exec), RuntimeError>
    where
        Self: Sized,
    {
        let mut call_frame_update = CallFrameUpdate::empty();
        let receiver = RENodeId::Global(GlobalAddress::Component(self.receiver));
        let resolved_receiver = deref_and_update(receiver, &mut call_frame_update, deref)?;

        let actor = ResolvedActor::method(
            NativeFn::EpochManager(EpochManagerFn::SetEpoch),
            resolved_receiver,
        );
        let executor = EpochManagerSetEpochExecutable(resolved_receiver.receiver, self.epoch);

        Ok((actor, call_frame_update, executor))
    }
}

impl Executor for EpochManagerSetEpochExecutable {
    type Output = ();

    fn execute<Y, W: WasmEngine>(
        self,
        system_api: &mut Y,
    ) -> Result<((), CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi,
    {
        let offset = SubstateOffset::EpochManager(EpochManagerOffset::EpochManager);
        let handle =
            system_api.lock_substate(self.0, NodeModuleId::SELF, offset, LockFlags::MUTABLE)?;
        let mut substate_mut = system_api.get_ref_mut(handle)?;
        substate_mut.epoch_manager().epoch = self.1;
        Ok(((), CallFrameUpdate::empty()))
    }
}

pub struct EpochManagerCreateValidatorExecutable(RENodeId, EcdsaSecp256k1PublicKey);

impl ExecutableInvocation for EpochManagerCreateValidatorInvocation {
    type Exec = EpochManagerCreateValidatorExecutable;

    fn resolve<D: ResolverApi>(
        self,
        deref: &mut D,
    ) -> Result<(ResolvedActor, CallFrameUpdate, Self::Exec), RuntimeError>
    where
        Self: Sized,
    {
        let mut call_frame_update = CallFrameUpdate::empty();
        let receiver = RENodeId::Global(GlobalAddress::Component(self.receiver));
        let resolved_receiver = deref_and_update(receiver, &mut call_frame_update, deref)?;
        call_frame_update.add_ref(RENodeId::Global(GlobalAddress::Resource(RADIX_TOKEN)));

        let actor = ResolvedActor::method(
            NativeFn::EpochManager(EpochManagerFn::CreateValidator),
            resolved_receiver,
        );
        let executor = EpochManagerCreateValidatorExecutable(resolved_receiver.receiver, self.key);

        Ok((actor, call_frame_update, executor))
    }
}

impl Executor for EpochManagerCreateValidatorExecutable {
    type Output = ComponentAddress;

    fn execute<Y, W: WasmEngine>(
        self,
        api: &mut Y,
    ) -> Result<(ComponentAddress, CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi + EngineApi<RuntimeError> + InvokableModel<RuntimeError>,
    {
        let handle = api.lock_substate(
            self.0,
            NodeModuleId::SELF,
            SubstateOffset::EpochManager(EpochManagerOffset::EpochManager),
            LockFlags::read_only(),
        )?;
        let substate_ref = api.get_ref(handle)?;
        let epoch_manager = substate_ref.epoch_manager();
        let manager = epoch_manager.address;
        let validator_address = ValidatorCreator::create(manager, self.1, false, api)?;
        Ok((
            validator_address,
            CallFrameUpdate::copy_ref(RENodeId::Global(GlobalAddress::Component(
                validator_address,
            ))),
        ))
    }
}

pub struct EpochManagerUpdateValidatorExecutable(RENodeId, ComponentAddress, UpdateValidator);

impl ExecutableInvocation for EpochManagerUpdateValidatorInvocation {
    type Exec = EpochManagerUpdateValidatorExecutable;

    fn resolve<D: ResolverApi>(
        self,
        deref: &mut D,
    ) -> Result<(ResolvedActor, CallFrameUpdate, Self::Exec), RuntimeError>
    where
        Self: Sized,
    {
        let mut call_frame_update = CallFrameUpdate::empty();
        let receiver = RENodeId::Global(GlobalAddress::Component(self.receiver));
        let resolved_receiver = deref_and_update(receiver, &mut call_frame_update, deref)?;

        let actor = ResolvedActor::method(
            NativeFn::EpochManager(EpochManagerFn::UpdateValidator),
            resolved_receiver,
        );
        let executor = EpochManagerUpdateValidatorExecutable(
            resolved_receiver.receiver,
            self.validator_address,
            self.update,
        );

        Ok((actor, call_frame_update, executor))
    }
}

impl Executor for EpochManagerUpdateValidatorExecutable {
    type Output = ();

    fn execute<Y, W: WasmEngine>(self, api: &mut Y) -> Result<((), CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi + InvokableModel<RuntimeError>,
    {
        let offset = SubstateOffset::EpochManager(EpochManagerOffset::PreparingValidatorSet);
        let handle = api.lock_substate(self.0, NodeModuleId::SELF, offset, LockFlags::MUTABLE)?;
        let mut substate_ref = api.get_ref_mut(handle)?;
        let validator_set = substate_ref.validator_set();
        match self.2 {
            UpdateValidator::Register(key, stake) => {
                validator_set
                    .validator_set
                    .insert(self.1, Validator { key, stake });
            }
            UpdateValidator::Unregister => {
                validator_set.validator_set.remove(&self.1);
            }
        }

        Ok(((), CallFrameUpdate::empty()))
    }
}

impl EpochManager {
    pub fn create_auth() -> Vec<MethodAuthorization> {
        vec![MethodAuthorization::Protected(HardAuthRule::ProofRule(
            HardProofRule::Require(HardResourceOrNonFungible::NonFungible(
                AuthAddresses::system_role(),
            )),
        ))]
    }
}
/*
<<<<<<< HEAD:radix-engine/src/model/epoch_manager/executables.rs

pub struct ValidatorRegisterExecutable(RENodeId);

impl ExecutableInvocation for ValidatorRegisterInvocation {
    type Exec = ValidatorRegisterExecutable;

    fn resolve<D: ResolverApi>(
        self,
        deref: &mut D,
    ) -> Result<(ResolvedActor, CallFrameUpdate, Self::Exec), RuntimeError>
    where
        Self: Sized,
    {
        let mut call_frame_update = CallFrameUpdate::empty();
        let receiver = RENodeId::Global(GlobalAddress::Component(self.receiver));
        let resolved_receiver = deref_and_update(receiver, &mut call_frame_update, deref)?;

        let actor = ResolvedActor::method(
            NativeFn::Validator(ValidatorFn::Register),
            resolved_receiver,
        );
        let executor = ValidatorRegisterExecutable(resolved_receiver.receiver);
        Ok((actor, call_frame_update, executor))
    }
}

impl Executor for ValidatorRegisterExecutable {
    type Output = ();

    fn execute<Y, W: WasmEngine>(self, api: &mut Y) -> Result<((), CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi + EngineApi<RuntimeError> + InvokableModel<RuntimeError>,
    {
        let offset = SubstateOffset::Validator(ValidatorOffset::Validator);
        let handle = api.lock_substate(
            self.0,
            NodeModuleId::SELF,
            offset.clone(),
            LockFlags::MUTABLE,
        )?;

        // Update state
        {
            let mut substate = api.get_ref_mut(handle)?;
            let validator = substate.validator();

            if validator.is_registered {
                return Ok(((), CallFrameUpdate::empty()));
            }

            validator.is_registered = true;
        }

        // Update EpochManager
        {
            let substate = api.get_ref(handle)?;
            let validator = substate.validator();
            let stake_vault = Vault(validator.stake_vault_id);
            let stake_amount = stake_vault.sys_amount(api)?;
            if stake_amount.is_positive() {
                let substate = api.get_ref(handle)?;
                let validator = substate.validator();
                let invocation = EpochManagerUpdateValidatorInvocation {
                    receiver: validator.manager,
                    validator_address: validator.address,
                    update: UpdateValidator::Register(validator.key, stake_amount),
                };
                api.invoke(invocation)?;
            }
        }

        Ok(((), CallFrameUpdate::empty()))
    }
}

pub struct ValidatorUnregisterExecutable(RENodeId);

impl ExecutableInvocation for ValidatorUnregisterInvocation {
    type Exec = ValidatorUnregisterExecutable;

    fn resolve<D: ResolverApi>(
        self,
        deref: &mut D,
    ) -> Result<(ResolvedActor, CallFrameUpdate, Self::Exec), RuntimeError>
    where
        Self: Sized,
    {
        let mut call_frame_update = CallFrameUpdate::empty();
        let receiver = RENodeId::Global(GlobalAddress::Component(self.receiver));
        let resolved_receiver = deref_and_update(receiver, &mut call_frame_update, deref)?;
        let actor = ResolvedActor::method(
            NativeFn::Validator(ValidatorFn::Unregister),
            resolved_receiver,
        );
        let executor = ValidatorUnregisterExecutable(resolved_receiver.receiver);
        Ok((actor, call_frame_update, executor))
    }
}

impl Executor for ValidatorUnregisterExecutable {
    type Output = ();

    fn execute<Y, W: WasmEngine>(self, api: &mut Y) -> Result<((), CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi + InvokableModel<RuntimeError>,
    {
        let offset = SubstateOffset::Validator(ValidatorOffset::Validator);
        let handle = api.lock_substate(
            self.0,
            NodeModuleId::SELF,
            offset.clone(),
            LockFlags::MUTABLE,
        )?;

        // Update state
        {
            let mut substate = api.get_ref_mut(handle)?;
            let validator = substate.validator();
            if !validator.is_registered {
                return Ok(((), CallFrameUpdate::empty()));
            }
            validator.is_registered = false;
        }

        // Update EpochManager
        {
            let mut substate = api.get_ref_mut(handle)?;
            let validator = substate.validator();
            let invocation = EpochManagerUpdateValidatorInvocation {
                receiver: validator.manager,
                validator_address: validator.address,
                update: UpdateValidator::Unregister,
            };
            api.invoke(invocation)?;
        }

        Ok(((), CallFrameUpdate::empty()))
    }
}

pub struct ValidatorStakeExecutable(RENodeId, Bucket);

impl ExecutableInvocation for ValidatorStakeInvocation {
    type Exec = ValidatorStakeExecutable;

    fn resolve<D: ResolverApi>(
        self,
        deref: &mut D,
    ) -> Result<(ResolvedActor, CallFrameUpdate, Self::Exec), RuntimeError>
    where
        Self: Sized,
    {
        let mut call_frame_update = CallFrameUpdate::empty();
        let receiver = RENodeId::Global(GlobalAddress::Component(self.receiver));
        let resolved_receiver = deref_and_update(receiver, &mut call_frame_update, deref)?;
        call_frame_update
            .nodes_to_move
            .push(RENodeId::Bucket(self.stake.0));

        let actor =
            ResolvedActor::method(NativeFn::Validator(ValidatorFn::Stake), resolved_receiver);
        let executor = ValidatorStakeExecutable(resolved_receiver.receiver, self.stake);
        Ok((actor, call_frame_update, executor))
    }
}

impl Executor for ValidatorStakeExecutable {
    type Output = ();

    fn execute<Y, W: WasmEngine>(self, api: &mut Y) -> Result<((), CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi + EngineApi<RuntimeError> + InvokableModel<RuntimeError>,
    {
        let offset = SubstateOffset::Validator(ValidatorOffset::Validator);
        let handle =
            api.lock_substate(self.0, NodeModuleId::SELF, offset, LockFlags::read_only())?;

        // Stake
        {
            let substate = api.get_ref(handle)?;
            let validator = substate.validator();
            let mut xrd_vault = Vault(validator.stake_vault_id);
            xrd_vault.sys_put(self.1, api)?;
        }

        // Update EpochManager
        {
            let substate = api.get_ref(handle)?;
            let validator = substate.validator();
            if validator.is_registered {
                let receiver = validator.manager;
                let key = validator.key;
                let validator_address = validator.address;
                let xrd_vault = Vault(validator.stake_vault_id);
                let xrd_amount = xrd_vault.sys_amount(api)?;
                let invocation = EpochManagerUpdateValidatorInvocation {
                    receiver,
                    validator_address,
                    update: UpdateValidator::Register(key, xrd_amount),
                };
                api.invoke(invocation)?;
            }
        }

        Ok(((), CallFrameUpdate::empty()))
    }
}

pub struct ValidatorUnstakeExecutable(RENodeId, Decimal);

impl ExecutableInvocation for ValidatorUnstakeInvocation {
    type Exec = ValidatorUnstakeExecutable;

    fn resolve<D: ResolverApi>(
        self,
        deref: &mut D,
    ) -> Result<(ResolvedActor, CallFrameUpdate, Self::Exec), RuntimeError>
    where
        Self: Sized,
    {
        let mut call_frame_update = CallFrameUpdate::empty();
        let receiver = RENodeId::Global(GlobalAddress::Component(self.receiver));
        let resolved_receiver = deref_and_update(receiver, &mut call_frame_update, deref)?;

        let actor =
            ResolvedActor::method(NativeFn::Validator(ValidatorFn::Unstake), resolved_receiver);
        let executor = ValidatorUnstakeExecutable(resolved_receiver.receiver, self.amount);
        Ok((actor, call_frame_update, executor))
    }
}

impl Executor for ValidatorUnstakeExecutable {
    type Output = Bucket;

    fn execute<Y, W: WasmEngine>(
        self,
        api: &mut Y,
    ) -> Result<(Bucket, CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi + EngineApi<RuntimeError> + InvokableModel<RuntimeError>,
    {
        let offset = SubstateOffset::Validator(ValidatorOffset::Validator);
        let handle =
            api.lock_substate(self.0, NodeModuleId::SELF, offset, LockFlags::read_only())?;

        // Unstake
        let unstake_bucket = {
            let substate = api.get_ref(handle)?;
            let validator = substate.validator();
            let mut stake_vault = Vault(validator.stake_vault_id);
            stake_vault.sys_take(self.1, api)?
        };

        // Update EpochManager
        {
            let substate = api.get_ref(handle)?;
            let validator = substate.validator();
            if validator.is_registered {
                let stake_vault = Vault(validator.stake_vault_id);
                let stake_amount = stake_vault.sys_amount(api)?;

                let substate = api.get_ref(handle)?;
                let validator = substate.validator();
                let update = if stake_amount.is_zero() {
                    UpdateValidator::Unregister
                } else {
                    UpdateValidator::Register(validator.key, stake_amount)
                };

                let invocation = EpochManagerUpdateValidatorInvocation {
                    receiver: validator.manager,
                    validator_address: validator.address,
                    update,
                };
                api.invoke(invocation)?;
            }
        };

        let update = CallFrameUpdate::move_node(RENodeId::Bucket(unstake_bucket.0));
        Ok((unstake_bucket, update))
    }
}
=======
>>>>>>> develop:radix-engine/src/model/epoch_manager/epoch_manager.rs
 */