use crate::engine::{
    deref_and_update, ApplicationError, CallFrameUpdate, ExecutableInvocation, Executor, LockFlags,
    RENodeInit, ResolvedActor, ResolverApi, RuntimeError, SystemApi,
};
use crate::model::{AccessRulesChainSubstate, GlobalAddressSubstate, MetadataSubstate};
use crate::types::*;
use crate::wasm::WasmEngine;
use native_sdk::resource::{ResourceManager, SysBucket, Vault};
use radix_engine_interface::api::types::{GlobalAddress, NativeFn, RENodeId, SubstateOffset};
use radix_engine_interface::api::{EngineApi, InvokableModel};
use radix_engine_interface::model::*;
use radix_engine_interface::rule;

#[derive(Debug, Clone, PartialEq, Eq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct ValidatorSubstate {
    pub manager: ComponentAddress,
    pub address: ComponentAddress,
    pub key: EcdsaSecp256k1PublicKey,
    pub is_registered: bool,

    pub unstake_nft: ResourceAddress,
    pub liquidity_token: ResourceAddress,
    pub stake_xrd_vault_id: VaultId,
    pub pending_xrd_withdraw_vault_id: VaultId,
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub enum ValidatorError {
    InvalidClaimResource,
    EpochUnlockHasNotOccurredYet,
}

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
        let handle = api.lock_substate(self.0, offset.clone(), LockFlags::MUTABLE)?;

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
            let stake_vault = Vault(validator.stake_xrd_vault_id);
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
        let handle = api.lock_substate(self.0, offset.clone(), LockFlags::MUTABLE)?;

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
    type Output = Bucket;

    fn execute<Y, W: WasmEngine>(
        self,
        api: &mut Y,
    ) -> Result<(Bucket, CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi + EngineApi<RuntimeError> + InvokableModel<RuntimeError>,
    {
        let offset = SubstateOffset::Validator(ValidatorOffset::Validator);
        let handle = api.lock_substate(self.0, offset, LockFlags::read_only())?;

        // Stake
        let lp_token_bucket = {
            let substate = api.get_ref(handle)?;
            let validator = substate.validator();
            let mut lp_token_resman = ResourceManager(validator.liquidity_token);
            let mut xrd_vault = Vault(validator.stake_xrd_vault_id);

            let total_lp_supply = lp_token_resman.total_supply(api)?;
            let active_stake_amount = xrd_vault.sys_amount(api)?;
            let xrd_bucket = self.1;
            let stake_amount = xrd_bucket.sys_amount(api)?;
            let lp_mint_amount = if active_stake_amount.is_zero() {
                stake_amount
            } else {
                stake_amount * total_lp_supply / active_stake_amount
            };

            let lp_token_bucket = lp_token_resman.mint_fungible(lp_mint_amount, api)?;
            xrd_vault.sys_put(xrd_bucket, api)?;
            lp_token_bucket
        };

        // Update EpochManager
        {
            let substate = api.get_ref(handle)?;
            let validator = substate.validator();
            if validator.is_registered {
                let receiver = validator.manager;
                let key = validator.key;
                let validator_address = validator.address;
                let xrd_vault = Vault(validator.stake_xrd_vault_id);
                let xrd_amount = xrd_vault.sys_amount(api)?;
                let invocation = EpochManagerUpdateValidatorInvocation {
                    receiver,
                    validator_address,
                    update: UpdateValidator::Register(key, xrd_amount),
                };
                api.invoke(invocation)?;
            }
        }

        let update = CallFrameUpdate::move_node(RENodeId::Bucket(lp_token_bucket.0));
        Ok((lp_token_bucket, update))
    }
}

pub struct ValidatorUnstakeExecutable(RENodeId, Bucket);

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
        call_frame_update
            .nodes_to_move
            .push(RENodeId::Bucket(self.lp_tokens.0));

        let actor =
            ResolvedActor::method(NativeFn::Validator(ValidatorFn::Unstake), resolved_receiver);
        let executor = ValidatorUnstakeExecutable(resolved_receiver.receiver, self.lp_tokens);
        Ok((actor, call_frame_update, executor))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct UnstakeData {
    epoch_unlocked: u64,
    amount: Decimal,
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
        let handle = api.lock_substate(self.0, offset, LockFlags::read_only())?;

        // Unstake
        let unstake_bucket = {
            let substate = api.get_ref(handle)?;
            let validator = substate.validator();

            let manager = validator.manager;
            let mut stake_vault = Vault(validator.stake_xrd_vault_id);
            let mut unstake_vault = Vault(validator.pending_xrd_withdraw_vault_id);
            let mut nft_resman = ResourceManager(validator.unstake_nft);
            let mut lp_token_resman = ResourceManager(validator.liquidity_token);

            let active_stake_amount = stake_vault.sys_amount(api)?;
            let total_lp_supply = lp_token_resman.total_supply(api)?;
            let lp_tokens = self.1;
            let lp_token_amount = lp_tokens.sys_amount(api)?;
            let xrd_amount = if total_lp_supply.is_zero() {
                Decimal::zero()
            } else {
                lp_token_amount * active_stake_amount / total_lp_supply
            };

            lp_token_resman.burn(lp_tokens, api)?;

            let manager_handle = api.lock_substate(
                RENodeId::Global(GlobalAddress::Component(manager)),
                SubstateOffset::EpochManager(EpochManagerOffset::EpochManager),
                LockFlags::read_only(),
            )?;
            let manager_substate = api.get_ref(manager_handle)?;
            let epoch_manager = manager_substate.epoch_manager();
            let current_epoch = epoch_manager.epoch;
            let epoch_unlocked = current_epoch + epoch_manager.num_unstake_epochs;
            api.drop_lock(manager_handle)?;

            let data = UnstakeData {
                epoch_unlocked,
                amount: xrd_amount,
            };

            let bucket = stake_vault.sys_take(xrd_amount, api)?;
            unstake_vault.sys_put(bucket, api)?;
            nft_resman.mint_non_fungible_uuid(data, api)?
        };

        // Update Epoch Manager
        {
            let substate = api.get_ref(handle)?;
            let validator = substate.validator();
            let stake_vault = Vault(validator.stake_xrd_vault_id);
            if validator.is_registered {
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

pub struct ValidatorClaimXrdExecutable(RENodeId, Bucket);

impl ExecutableInvocation for ValidatorClaimXrdInvocation {
    type Exec = ValidatorClaimXrdExecutable;

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
            .push(RENodeId::Bucket(self.unstake_nft.0));

        let actor = ResolvedActor::method(
            NativeFn::Validator(ValidatorFn::ClaimXrd),
            resolved_receiver,
        );
        let executor = ValidatorClaimXrdExecutable(resolved_receiver.receiver, self.unstake_nft);
        Ok((actor, call_frame_update, executor))
    }
}

impl Executor for ValidatorClaimXrdExecutable {
    type Output = Bucket;

    fn execute<Y, W: WasmEngine>(
        self,
        api: &mut Y,
    ) -> Result<(Bucket, CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi + EngineApi<RuntimeError> + InvokableModel<RuntimeError>,
    {
        let offset = SubstateOffset::Validator(ValidatorOffset::Validator);
        let handle = api.lock_substate(self.0, offset, LockFlags::read_only())?;
        let substate = api.get_ref(handle)?;
        let validator = substate.validator();
        let mut nft_resman = ResourceManager(validator.unstake_nft);
        let resource_address = validator.unstake_nft;
        let manager = validator.manager;
        let mut unstake_vault = Vault(validator.pending_xrd_withdraw_vault_id);

        // TODO: Move this check into a more appropriate place
        let bucket = Bucket(self.1 .0);
        if !resource_address.eq(&bucket.sys_resource_address(api)?) {
            return Err(RuntimeError::ApplicationError(
                ApplicationError::ValidatorError(ValidatorError::InvalidClaimResource),
            ));
        }

        let current_epoch = {
            let mgr_handle = api.lock_substate(
                RENodeId::Global(GlobalAddress::Component(manager)),
                SubstateOffset::EpochManager(EpochManagerOffset::EpochManager),
                LockFlags::read_only(),
            )?;
            let mgr_substate = api.get_ref(mgr_handle)?;
            let epoch = mgr_substate.epoch_manager().epoch;
            api.drop_lock(mgr_handle)?;
            epoch
        };

        let mut unstake_amount = Decimal::zero();

        for id in bucket.sys_total_ids(api)? {
            let data: UnstakeData = nft_resman.get_non_fungible_mutable_data(id, api)?;
            if current_epoch < data.epoch_unlocked {
                return Err(RuntimeError::ApplicationError(
                    ApplicationError::ValidatorError(ValidatorError::EpochUnlockHasNotOccurredYet),
                ));
            }
            unstake_amount += data.amount;
        }
        nft_resman.burn(bucket, api)?;

        let claimed_bucket = unstake_vault.sys_take(unstake_amount, api)?;
        let update = CallFrameUpdate::move_node(RENodeId::Bucket(claimed_bucket.0));
        Ok((claimed_bucket, update))
    }
}

pub struct ValidatorUpdateKeyExecutable(RENodeId, EcdsaSecp256k1PublicKey);

impl ExecutableInvocation for ValidatorUpdateKeyInvocation {
    type Exec = ValidatorUpdateKeyExecutable;

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
            NativeFn::Validator(ValidatorFn::UpdateKey),
            resolved_receiver,
        );
        let executor = ValidatorUpdateKeyExecutable(resolved_receiver.receiver, self.key);
        Ok((actor, call_frame_update, executor))
    }
}

impl Executor for ValidatorUpdateKeyExecutable {
    type Output = ();

    fn execute<Y, W: WasmEngine>(self, api: &mut Y) -> Result<((), CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi + EngineApi<RuntimeError> + InvokableModel<RuntimeError>,
    {
        let offset = SubstateOffset::Validator(ValidatorOffset::Validator);
        let handle = api.lock_substate(self.0, offset, LockFlags::MUTABLE)?;
        let mut substate = api.get_ref_mut(handle)?;
        let mut validator = substate.validator();
        validator.key = self.1;
        let key = validator.key;
        let manager = validator.manager;
        let validator_address = validator.address;

        // Update Epoch Manager
        {
            let stake_vault = Vault(validator.stake_xrd_vault_id);
            if validator.is_registered {
                let stake_amount = stake_vault.sys_amount(api)?;
                if !stake_amount.is_zero() {
                    let update = UpdateValidator::Register(key, stake_amount);
                    let invocation = EpochManagerUpdateValidatorInvocation {
                        receiver: manager,
                        validator_address,
                        update,
                    };
                    api.invoke(invocation)?;
                }
            }
        };

        let update = CallFrameUpdate::empty();
        Ok(((), update))
    }
}

pub struct ValidatorUpdateAcceptDelegatedStakeExecutable(RENodeId, bool);

impl ExecutableInvocation for ValidatorUpdateAcceptDelegatedStakeInvocation {
    type Exec = ValidatorUpdateAcceptDelegatedStakeExecutable;

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
            NativeFn::Validator(ValidatorFn::UpdateAcceptDelegatedStake),
            resolved_receiver,
        );
        let executor = ValidatorUpdateAcceptDelegatedStakeExecutable(
            resolved_receiver.receiver,
            self.accept_delegated_stake,
        );
        Ok((actor, call_frame_update, executor))
    }
}

impl Executor for ValidatorUpdateAcceptDelegatedStakeExecutable {
    type Output = ();

    fn execute<Y, W: WasmEngine>(self, api: &mut Y) -> Result<((), CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi + EngineApi<RuntimeError> + InvokableModel<RuntimeError>,
    {
        let rule = if self.1 {
            AccessRuleEntry::AccessRule(AccessRule::AllowAll)
        } else {
            AccessRuleEntry::Group("owner".to_string())
        };

        api.invoke(AccessRulesSetMethodAccessRuleInvocation {
            receiver: self.0,
            index: 0u32,
            key: AccessRuleKey::Native(NativeFn::Validator(ValidatorFn::Stake)),
            rule,
        })?;

        let update = CallFrameUpdate::empty();
        Ok(((), update))
    }
}

pub(crate) struct ValidatorCreator;

impl ValidatorCreator {
    fn create_liquidity_token_with_initial_amount<Y>(
        amount: Decimal,
        api: &mut Y,
    ) -> Result<(ResourceAddress, Bucket), RuntimeError>
    where
        Y: SystemApi + EngineApi<RuntimeError> + InvokableModel<RuntimeError>,
    {
        let mut liquidity_token_auth = BTreeMap::new();
        let non_fungible_id = NonFungibleLocalId::bytes(
            scrypto_encode(&PackageIdentifier::Native(NativePackage::EpochManager)).unwrap(),
        )
        .unwrap();
        let non_fungible_global_id = NonFungibleGlobalId::new(PACKAGE_TOKEN, non_fungible_id);
        liquidity_token_auth.insert(
            Mint,
            (
                rule!(require(non_fungible_global_id.clone())),
                rule!(deny_all),
            ),
        );
        liquidity_token_auth.insert(
            Burn,
            (rule!(require(non_fungible_global_id)), rule!(deny_all)),
        );
        liquidity_token_auth.insert(Withdraw, (rule!(allow_all), rule!(deny_all)));
        liquidity_token_auth.insert(Deposit, (rule!(allow_all), rule!(deny_all)));

        let (liquidity_token_resource_manager, bucket) =
            ResourceManager::new_fungible_with_initial_supply(
                18,
                amount,
                BTreeMap::new(),
                liquidity_token_auth,
                api,
            )?;

        Ok((liquidity_token_resource_manager.0, bucket))
    }

    fn create_liquidity_token<Y>(api: &mut Y) -> Result<ResourceAddress, RuntimeError>
    where
        Y: SystemApi + EngineApi<RuntimeError> + InvokableModel<RuntimeError>,
    {
        let mut liquidity_token_auth = BTreeMap::new();
        let non_fungible_local_id = NonFungibleLocalId::bytes(
            scrypto_encode(&PackageIdentifier::Native(NativePackage::EpochManager)).unwrap(),
        )
        .unwrap();
        let non_fungible_global_id = NonFungibleGlobalId::new(PACKAGE_TOKEN, non_fungible_local_id);
        liquidity_token_auth.insert(
            Mint,
            (
                rule!(require(non_fungible_global_id.clone())),
                rule!(deny_all),
            ),
        );
        liquidity_token_auth.insert(
            Burn,
            (rule!(require(non_fungible_global_id)), rule!(deny_all)),
        );
        liquidity_token_auth.insert(Withdraw, (rule!(allow_all), rule!(deny_all)));
        liquidity_token_auth.insert(Deposit, (rule!(allow_all), rule!(deny_all)));

        let liquidity_token_resource_manager =
            ResourceManager::new_fungible(18, BTreeMap::new(), liquidity_token_auth, api)?;

        Ok(liquidity_token_resource_manager.0)
    }

    fn create_unstake_nft<Y>(api: &mut Y) -> Result<ResourceAddress, RuntimeError>
    where
        Y: SystemApi + EngineApi<RuntimeError> + InvokableModel<RuntimeError>,
    {
        let mut unstake_token_auth = BTreeMap::new();
        let non_fungible_local_id = NonFungibleLocalId::bytes(
            scrypto_encode(&PackageIdentifier::Native(NativePackage::EpochManager)).unwrap(),
        )
        .unwrap();
        let non_fungible_global_id = NonFungibleGlobalId::new(PACKAGE_TOKEN, non_fungible_local_id);
        unstake_token_auth.insert(
            Mint,
            (
                rule!(require(non_fungible_global_id.clone())),
                rule!(deny_all),
            ),
        );
        unstake_token_auth.insert(
            Burn,
            (rule!(require(non_fungible_global_id)), rule!(deny_all)),
        );
        unstake_token_auth.insert(Withdraw, (rule!(allow_all), rule!(deny_all)));
        unstake_token_auth.insert(Deposit, (rule!(allow_all), rule!(deny_all)));

        let unstake_resource_manager = ResourceManager::new_non_fungible(
            NonFungibleIdType::UUID,
            BTreeMap::new(),
            unstake_token_auth,
            api,
        )?;

        Ok(unstake_resource_manager.0)
    }

    fn build_access_rules(owner_access_rule: AccessRule) -> AccessRules {
        let mut access_rules = AccessRules::new();
        access_rules.set_group_access_rule_and_mutability(
            "owner".to_string(),
            owner_access_rule,
            AccessRule::DenyAll,
        );
        access_rules.set_method_access_rule_to_group(
            AccessRuleKey::Native(NativeFn::Metadata(MetadataFn::Set)),
            "owner".to_string(),
        );
        access_rules.set_method_access_rule(
            AccessRuleKey::Native(NativeFn::Validator(ValidatorFn::Register)),
            "owner".to_string(),
        );
        access_rules.set_method_access_rule(
            AccessRuleKey::Native(NativeFn::Validator(ValidatorFn::Unregister)),
            "owner".to_string(),
        );
        access_rules.set_method_access_rule(
            AccessRuleKey::Native(NativeFn::Validator(ValidatorFn::UpdateKey)),
            "owner".to_string(),
        );
        access_rules.set_method_access_rule(
            AccessRuleKey::Native(NativeFn::Validator(ValidatorFn::UpdateAcceptDelegatedStake)),
            "owner".to_string(),
        );

        let non_fungible_local_id = NonFungibleLocalId::bytes(
            scrypto_encode(&PackageIdentifier::Native(NativePackage::EpochManager)).unwrap(),
        )
        .unwrap();
        let non_fungible_global_id = NonFungibleGlobalId::new(PACKAGE_TOKEN, non_fungible_local_id);
        access_rules.set_group_and_mutability(
            AccessRuleKey::Native(NativeFn::Validator(ValidatorFn::Stake)),
            "owner".to_string(),
            rule!(require(non_fungible_global_id)),
        );

        access_rules.set_method_access_rule(
            AccessRuleKey::Native(NativeFn::Metadata(MetadataFn::Get)),
            rule!(allow_all),
        );
        access_rules.set_method_access_rule(
            AccessRuleKey::Native(NativeFn::Validator(ValidatorFn::Unstake)),
            rule!(allow_all),
        );
        access_rules.set_method_access_rule(
            AccessRuleKey::Native(NativeFn::Validator(ValidatorFn::ClaimXrd)),
            rule!(allow_all),
        );

        access_rules
    }

    pub fn create_with_initial_stake<Y>(
        manager: ComponentAddress,
        key: EcdsaSecp256k1PublicKey,
        owner_access_rule: AccessRule,
        initial_stake: Bucket,
        is_registered: bool,
        api: &mut Y,
    ) -> Result<(ComponentAddress, Bucket), RuntimeError>
    where
        Y: SystemApi + EngineApi<RuntimeError> + InvokableModel<RuntimeError>,
    {
        let node_id = api.allocate_node_id(RENodeType::Validator)?;
        let global_node_id = api.allocate_node_id(RENodeType::GlobalValidator)?;
        let address: ComponentAddress = global_node_id.into();
        let initial_liquidity_amount = initial_stake.sys_amount(api)?;
        let mut stake_vault = Vault::sys_new(RADIX_TOKEN, api)?;
        stake_vault.sys_put(initial_stake, api)?;
        let unstake_vault = Vault::sys_new(RADIX_TOKEN, api)?;
        let unstake_nft = Self::create_unstake_nft(api)?;
        let (liquidity_token, liquidity_bucket) =
            Self::create_liquidity_token_with_initial_amount(initial_liquidity_amount, api)?;
        let node = RENodeInit::Validator(
            ValidatorSubstate {
                manager,
                key,
                address,
                liquidity_token,
                unstake_nft,
                stake_xrd_vault_id: stake_vault.0,
                pending_xrd_withdraw_vault_id: unstake_vault.0,
                is_registered,
            },
            MetadataSubstate {
                metadata: BTreeMap::new(),
            },
            AccessRulesChainSubstate {
                access_rules_chain: vec![Self::build_access_rules(owner_access_rule)],
            },
        );
        api.create_node(node_id, node)?;
        api.create_node(
            global_node_id,
            RENodeInit::Global(GlobalAddressSubstate::Validator(node_id.into())),
        )?;

        Ok((global_node_id.into(), liquidity_bucket))
    }

    pub fn create<Y>(
        manager: ComponentAddress,
        key: EcdsaSecp256k1PublicKey,
        owner_access_rule: AccessRule,
        is_registered: bool,
        api: &mut Y,
    ) -> Result<ComponentAddress, RuntimeError>
    where
        Y: SystemApi + EngineApi<RuntimeError> + InvokableModel<RuntimeError>,
    {
        let node_id = api.allocate_node_id(RENodeType::Validator)?;
        let global_node_id = api.allocate_node_id(RENodeType::GlobalValidator)?;
        let address: ComponentAddress = global_node_id.into();
        let stake_vault = Vault::sys_new(RADIX_TOKEN, api)?;
        let unstake_vault = Vault::sys_new(RADIX_TOKEN, api)?;
        let unstake_nft = Self::create_unstake_nft(api)?;
        let liquidity_token = Self::create_liquidity_token(api)?;
        let node = RENodeInit::Validator(
            ValidatorSubstate {
                manager,
                key,
                address,
                liquidity_token,
                unstake_nft,
                stake_xrd_vault_id: stake_vault.0,
                pending_xrd_withdraw_vault_id: unstake_vault.0,
                is_registered,
            },
            MetadataSubstate {
                metadata: BTreeMap::new(),
            },
            AccessRulesChainSubstate {
                access_rules_chain: vec![Self::build_access_rules(owner_access_rule)],
            },
        );
        api.create_node(node_id, node)?;
        api.create_node(
            global_node_id,
            RENodeInit::Global(GlobalAddressSubstate::Validator(node_id.into())),
        )?;

        Ok(global_node_id.into())
    }
}
