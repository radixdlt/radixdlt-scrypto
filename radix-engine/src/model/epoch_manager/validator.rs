use crate::engine::{
    deref_and_update, ApplicationError, CallFrameUpdate, ExecutableInvocation, Executor, LockFlags,
    RENode, ResolvedActor, ResolverApi, RuntimeError, SystemApi,
};
use crate::model::{AccessRulesChainSubstate, GlobalAddressSubstate};
use crate::types::*;
use crate::wasm::WasmEngine;
use native_sdk::resource::{NativeVault, ResourceManager, SysBucket};
use radix_engine_interface::api::api::{EngineApi, InvokableModel};
use radix_engine_interface::api::types::{GlobalAddress, NativeFn, RENodeId, SubstateOffset};
use radix_engine_interface::model::*;
use radix_engine_interface::rule;

#[derive(Debug, Clone, PartialEq, Eq)]
#[scrypto(TypeId, Encode, Decode)]
pub struct ValidatorSubstate {
    pub manager: SystemAddress,
    pub address: SystemAddress,
    pub key: EcdsaSecp256k1PublicKey,
    pub is_registered: bool,

    pub unstake_nft: ResourceAddress,
    pub liquidity_token: ResourceAddress,
    pub stake_xrd_vault_id: VaultId,
    pub pending_xrd_withdraw_vault_id: VaultId,
}

#[derive(Debug, Clone, Eq, PartialEq, TypeId, Encode, Decode)]
pub enum ValidatorError {
    InvalidClaimResource,
    EpochUnlockHasNotOccurredYet,
}

pub struct ValidatorRegisterExecutable(RENodeId);

impl<W: WasmEngine> ExecutableInvocation<W> for ValidatorRegisterInvocation {
    type Exec = ValidatorRegisterExecutable;

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
            NativeFn::Validator(ValidatorFn::Register),
            resolved_receiver,
        );
        let executor = ValidatorRegisterExecutable(resolved_receiver.receiver);
        Ok((actor, call_frame_update, executor))
    }
}

impl Executor for ValidatorRegisterExecutable {
    type Output = ();

    fn execute<Y>(self, api: &mut Y) -> Result<((), CallFrameUpdate), RuntimeError>
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

impl<W: WasmEngine> ExecutableInvocation<W> for ValidatorUnregisterInvocation {
    type Exec = ValidatorUnregisterExecutable;

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
            NativeFn::Validator(ValidatorFn::Unregister),
            resolved_receiver,
        );
        let executor = ValidatorUnregisterExecutable(resolved_receiver.receiver);
        Ok((actor, call_frame_update, executor))
    }
}

impl Executor for ValidatorUnregisterExecutable {
    type Output = ();

    fn execute<Y>(self, api: &mut Y) -> Result<((), CallFrameUpdate), RuntimeError>
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

impl<W: WasmEngine> ExecutableInvocation<W> for ValidatorStakeInvocation {
    type Exec = ValidatorStakeExecutable;

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

    fn execute<Y>(self, api: &mut Y) -> Result<((), CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi + EngineApi<RuntimeError> + InvokableModel<RuntimeError>,
    {
        let offset = SubstateOffset::Validator(ValidatorOffset::Validator);
        let handle = api.lock_substate(self.0, offset, LockFlags::read_only())?;

        // Stake
        {
            let substate = api.get_ref(handle)?;
            let validator = substate.validator();
            let mut xrd_vault = Vault(validator.stake_xrd_vault_id);
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

        Ok(((), CallFrameUpdate::empty()))
    }
}

pub struct ValidatorUnstakeExecutable(RENodeId, Decimal);

impl<W: WasmEngine> ExecutableInvocation<W> for ValidatorUnstakeInvocation {
    type Exec = ValidatorUnstakeExecutable;

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

        let actor =
            ResolvedActor::method(NativeFn::Validator(ValidatorFn::Unstake), resolved_receiver);
        let executor = ValidatorUnstakeExecutable(resolved_receiver.receiver, self.amount);
        Ok((actor, call_frame_update, executor))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[scrypto(TypeId, Encode, Decode)]
pub struct UnstakeData {
    epoch_unlocked: u64,
    amount: Decimal,
}

impl Executor for ValidatorUnstakeExecutable {
    type Output = Bucket;

    fn execute<Y>(self, api: &mut Y) -> Result<(Bucket, CallFrameUpdate), RuntimeError>
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
            let manager_handle = api.lock_substate(
                RENodeId::Global(GlobalAddress::System(manager)),
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
                amount: self.1,
            };

            let bucket = stake_vault.sys_take(self.1, api)?;
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

impl<W: WasmEngine> ExecutableInvocation<W> for ValidatorClaimXrdInvocation {
    type Exec = ValidatorClaimXrdExecutable;

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
        call_frame_update
            .nodes_to_move
            .push(RENodeId::Bucket(self.bucket.0));

        let actor = ResolvedActor::method(
            NativeFn::Validator(ValidatorFn::ClaimXrd),
            resolved_receiver,
        );
        let executor = ValidatorClaimXrdExecutable(resolved_receiver.receiver, self.bucket);
        Ok((actor, call_frame_update, executor))
    }
}

impl Executor for ValidatorClaimXrdExecutable {
    type Output = Bucket;

    fn execute<Y>(self, api: &mut Y) -> Result<(Bucket, CallFrameUpdate), RuntimeError>
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
                RENodeId::Global(GlobalAddress::System(manager)),
                SubstateOffset::EpochManager(EpochManagerOffset::EpochManager),
                LockFlags::read_only(),
            )?;
            let mgr_substate = api.get_ref(mgr_handle)?;
            let epoch = mgr_substate.epoch_manager().epoch;
            api.drop_lock(mgr_handle)?;
            epoch
        };

        let mut unstake_amount = Decimal::zero();

        for id in bucket.sys_non_fungible_ids(api)? {
            let data: UnstakeData = nft_resman.get_non_fungible_data(id, api)?;
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
        let non_fungible_id = NonFungibleId::Bytes(
            scrypto_encode(&PackageIdentifier::Native(NativePackage::EpochManager)).unwrap(),
        );
        let non_fungible_address = NonFungibleAddress::new(PACKAGE_TOKEN, non_fungible_id);
        liquidity_token_auth.insert(
            Mint,
            (
                rule!(require(non_fungible_address.clone())),
                rule!(deny_all),
            ),
        );
        liquidity_token_auth.insert(
            Burn,
            (rule!(require(non_fungible_address)), rule!(deny_all)),
        );
        liquidity_token_auth.insert(Withdraw, (rule!(allow_all), rule!(deny_all)));
        liquidity_token_auth.insert(Deposit, (rule!(allow_all), rule!(deny_all)));

        let (unstake_resource_manager, bucket) = ResourceManager::sys_new(
            ResourceType::Fungible { divisibility: 0 },
            BTreeMap::new(),
            liquidity_token_auth,
            Some(MintParams::Fungible { amount }),
            api,
        )?;

        Ok((unstake_resource_manager.0, bucket.unwrap()))
    }

    fn create_liquidity_token<Y>(api: &mut Y) -> Result<ResourceAddress, RuntimeError>
    where
        Y: SystemApi + EngineApi<RuntimeError> + InvokableModel<RuntimeError>,
    {
        let mut liquidity_token_auth = BTreeMap::new();
        let non_fungible_id = NonFungibleId::Bytes(
            scrypto_encode(&PackageIdentifier::Native(NativePackage::EpochManager)).unwrap(),
        );
        let non_fungible_address = NonFungibleAddress::new(PACKAGE_TOKEN, non_fungible_id);
        liquidity_token_auth.insert(
            Mint,
            (
                rule!(require(non_fungible_address.clone())),
                rule!(deny_all),
            ),
        );
        liquidity_token_auth.insert(
            Burn,
            (rule!(require(non_fungible_address)), rule!(deny_all)),
        );
        liquidity_token_auth.insert(Withdraw, (rule!(allow_all), rule!(deny_all)));
        liquidity_token_auth.insert(Deposit, (rule!(allow_all), rule!(deny_all)));

        let (unstake_resource_manager, _) = ResourceManager::sys_new(
            ResourceType::Fungible { divisibility: 0 },
            BTreeMap::new(),
            liquidity_token_auth,
            None,
            api,
        )?;

        Ok(unstake_resource_manager.0)
    }

    fn create_unstake_nft<Y>(api: &mut Y) -> Result<ResourceAddress, RuntimeError>
    where
        Y: SystemApi + EngineApi<RuntimeError> + InvokableModel<RuntimeError>,
    {
        let mut unstake_token_auth = BTreeMap::new();
        let non_fungible_id = NonFungibleId::Bytes(
            scrypto_encode(&PackageIdentifier::Native(NativePackage::EpochManager)).unwrap(),
        );
        let non_fungible_address = NonFungibleAddress::new(PACKAGE_TOKEN, non_fungible_id);
        unstake_token_auth.insert(
            Mint,
            (
                rule!(require(non_fungible_address.clone())),
                rule!(deny_all),
            ),
        );
        unstake_token_auth.insert(
            Burn,
            (rule!(require(non_fungible_address)), rule!(deny_all)),
        );
        unstake_token_auth.insert(Withdraw, (rule!(allow_all), rule!(deny_all)));
        unstake_token_auth.insert(Deposit, (rule!(allow_all), rule!(deny_all)));

        let (unstake_resource_manager, _) = ResourceManager::sys_new(
            ResourceType::NonFungible {
                id_type: NonFungibleIdType::UUID,
            },
            BTreeMap::new(),
            unstake_token_auth,
            None,
            api,
        )?;

        Ok(unstake_resource_manager.0)
    }

    fn build_access_rules(key: EcdsaSecp256k1PublicKey) -> AccessRules {
        let mut access_rules = AccessRules::new();
        access_rules.set_method_access_rule(
            AccessRuleKey::Native(NativeFn::Validator(ValidatorFn::Register)),
            rule!(require(NonFungibleAddress::from_public_key(&key))),
        );
        access_rules.set_method_access_rule(
            AccessRuleKey::Native(NativeFn::Validator(ValidatorFn::Unregister)),
            rule!(require(NonFungibleAddress::from_public_key(&key))),
        );
        access_rules.set_method_access_rule(
            AccessRuleKey::Native(NativeFn::Validator(ValidatorFn::Stake)),
            rule!(require(NonFungibleAddress::from_public_key(&key))),
        );
        access_rules.set_method_access_rule(
            AccessRuleKey::Native(NativeFn::Validator(ValidatorFn::Unstake)),
            rule!(require(NonFungibleAddress::from_public_key(&key))),
        );
        access_rules.set_method_access_rule(
            AccessRuleKey::Native(NativeFn::Validator(ValidatorFn::ClaimXrd)),
            rule!(allow_all),
        );

        access_rules
    }

    pub fn create_with_initial_stake<Y>(
        manager: SystemAddress,
        key: EcdsaSecp256k1PublicKey,
        initial_stake: Bucket,
        is_registered: bool,
        api: &mut Y,
    ) -> Result<(SystemAddress, Bucket), RuntimeError>
    where
        Y: SystemApi + EngineApi<RuntimeError> + InvokableModel<RuntimeError>,
    {
        let node_id = api.allocate_node_id(RENodeType::Validator)?;
        let global_node_id = api.allocate_node_id(RENodeType::GlobalValidator)?;
        let address: SystemAddress = global_node_id.into();
        let initial_liquidity_amount = initial_stake.sys_amount(api)?;
        let mut stake_vault = Vault::sys_new(RADIX_TOKEN, api)?;
        stake_vault.sys_put(initial_stake, api)?;
        let unstake_vault = Vault::sys_new(RADIX_TOKEN, api)?;
        let unstake_nft = Self::create_unstake_nft(api)?;
        let (liquidity_token, liquidity_bucket) =
            Self::create_liquidity_token_with_initial_amount(initial_liquidity_amount, api)?;
        let node = RENode::Validator(
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
            AccessRulesChainSubstate {
                access_rules_chain: vec![Self::build_access_rules(key)],
            },
        );
        api.create_node(node_id, node)?;
        api.create_node(
            global_node_id,
            RENode::Global(GlobalAddressSubstate::Validator(node_id.into())),
        )?;

        Ok((global_node_id.into(), liquidity_bucket))
    }

    pub fn create<Y>(
        manager: SystemAddress,
        key: EcdsaSecp256k1PublicKey,
        is_registered: bool,
        api: &mut Y,
    ) -> Result<SystemAddress, RuntimeError>
    where
        Y: SystemApi + EngineApi<RuntimeError> + InvokableModel<RuntimeError>,
    {
        let node_id = api.allocate_node_id(RENodeType::Validator)?;
        let global_node_id = api.allocate_node_id(RENodeType::GlobalValidator)?;
        let address: SystemAddress = global_node_id.into();
        let stake_vault = Vault::sys_new(RADIX_TOKEN, api)?;
        let unstake_vault = Vault::sys_new(RADIX_TOKEN, api)?;
        let unstake_nft = Self::create_unstake_nft(api)?;
        let liquidity_token = Self::create_liquidity_token(api)?;
        let node = RENode::Validator(
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
            AccessRulesChainSubstate {
                access_rules_chain: vec![Self::build_access_rules(key)],
            },
        );
        api.create_node(node_id, node)?;
        api.create_node(
            global_node_id,
            RENode::Global(GlobalAddressSubstate::Validator(node_id.into())),
        )?;

        Ok(global_node_id.into())
    }
}
