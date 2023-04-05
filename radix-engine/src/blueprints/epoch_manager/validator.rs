use crate::blueprints::epoch_manager::EpochManagerSubstate;
use crate::blueprints::util::{MethodType, SecurifiedAccessRules};
use crate::errors::RuntimeError;
use crate::errors::{ApplicationError, InterpreterError};
use crate::kernel::kernel_api::{KernelNodeApi, KernelSubstateApi};
use crate::types::*;
use native_sdk::modules::metadata::Metadata;
use native_sdk::modules::royalty::ComponentRoyalty;
use native_sdk::resource::{ResourceManager, SysBucket, Vault};
use native_sdk::runtime::Runtime;
use radix_engine_interface::api::node_modules::auth::{
    AccessRulesSetMethodAccessRuleInput, ACCESS_RULES_SET_METHOD_ACCESS_RULE_IDENT,
};
use radix_engine_interface::api::substate_api::LockFlags;
use radix_engine_interface::api::ClientApi;
use radix_engine_interface::blueprints::epoch_manager::*;
use radix_engine_interface::blueprints::resource::*;
use radix_engine_interface::rule;

use super::{
    ClaimXrdEvent, RegisterValidatorEvent, StakeEvent, UnregisterValidatorEvent, UnstakeEvent,
    UpdateAcceptingStakeDelegationStateEvent,
};

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub struct ValidatorSubstate {
    pub manager: ComponentAddress,
    pub address: ComponentAddress,
    pub key: EcdsaSecp256k1PublicKey,
    pub is_registered: bool,

    pub unstake_nft: ResourceAddress,
    pub liquidity_token: ResourceAddress,
    pub stake_xrd_vault_id: Own,
    pub pending_xrd_withdraw_vault_id: Own,
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub struct UnstakeData {
    epoch_unlocked: u64,
    amount: Decimal,
}

impl NonFungibleData for UnstakeData {
    const MUTABLE_FIELDS: &'static [&'static str] = &[];
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum ValidatorError {
    InvalidClaimResource,
    EpochUnlockHasNotOccurredYet,
}

pub struct ValidatorBlueprint;

impl ValidatorBlueprint {
    pub fn register<Y>(
        receiver: &NodeId,
        input: &IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        let _input: ValidatorRegisterInput = input.as_typed().map_err(|e| {
            RuntimeError::InterpreterError(InterpreterError::ScryptoInputDecodeError(e))
        })?;

        let substate_key = ValidatorOffset::Validator.into();
        let handle = api.sys_lock_substate(receiver, &substate_key, LockFlags::MUTABLE)?;

        // Update state
        let validator = {
            let mut validator: ValidatorSubstate = api.sys_read_substate_typed(handle)?;

            if validator.is_registered {
                return Ok(IndexedScryptoValue::from_typed(&()));
            }
            validator.is_registered = true;

            api.sys_write_substate_typed(handle, &validator)?;
            validator
        };

        // Update EpochManager
        {
            let stake_vault = Vault(validator.stake_xrd_vault_id);
            let stake_amount = stake_vault.sys_amount(api)?;
            if stake_amount.is_positive() {
                let key = validator.key;
                let validator_address = validator.address;
                let manager = validator.manager;
                api.call_method(
                    manager.as_node_id(),
                    EPOCH_MANAGER_UPDATE_VALIDATOR_IDENT,
                    scrypto_encode(&EpochManagerUpdateValidatorInput {
                        update: UpdateValidator::Register(key, stake_amount),
                        validator_address,
                    })
                    .unwrap(),
                )?;
            }
        }

        Runtime::emit_event(api, RegisterValidatorEvent)?;

        return Ok(IndexedScryptoValue::from_typed(&()));
    }

    pub fn unregister<Y>(
        receiver: &NodeId,
        input: &IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        let _input: ValidatorUnregisterInput = input.as_typed().map_err(|e| {
            RuntimeError::InterpreterError(InterpreterError::ScryptoInputDecodeError(e))
        })?;

        let substate_key = ValidatorOffset::Validator.into();
        let handle = api.sys_lock_substate(receiver, &substate_key, LockFlags::MUTABLE)?;

        // Update state
        let validator = {
            let mut validator: ValidatorSubstate = api.sys_read_substate_typed(handle)?;

            if !validator.is_registered {
                return Ok(IndexedScryptoValue::from_typed(&()));
            }
            validator.is_registered = false;

            api.sys_write_substate_typed(handle, &validator)?;
            validator
        };

        // Update EpochManager
        {
            let manager = validator.manager;
            let validator_address = validator.address;
            api.call_method(
                manager.as_node_id(),
                EPOCH_MANAGER_UPDATE_VALIDATOR_IDENT,
                scrypto_encode(&EpochManagerUpdateValidatorInput {
                    validator_address,
                    update: UpdateValidator::Unregister,
                })
                .unwrap(),
            )?;
        }

        Runtime::emit_event(api, UnregisterValidatorEvent)?;

        return Ok(IndexedScryptoValue::from_typed(&()));
    }

    pub fn stake<Y>(
        receiver: &NodeId,
        input: &IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        let input: ValidatorStakeInput = input.as_typed().map_err(|e| {
            RuntimeError::InterpreterError(InterpreterError::ScryptoInputDecodeError(e))
        })?;

        // Prepare the event and emit it once the operations succeed
        let event = {
            let amount = input.stake.sys_amount(api)?;
            StakeEvent { xrd_staked: amount }
        };

        let handle = api.sys_lock_substate(
            receiver,
            &ValidatorOffset::Validator.into(),
            LockFlags::read_only(),
        )?;

        // Stake
        let lp_token_bucket = {
            let validator: ValidatorSubstate = api.sys_read_substate_typed(handle)?;
            let mut lp_token_resman = ResourceManager(validator.liquidity_token);
            let mut xrd_vault = Vault(validator.stake_xrd_vault_id);

            let total_lp_supply = lp_token_resman.total_supply(api)?;
            let active_stake_amount = xrd_vault.sys_amount(api)?;
            let xrd_bucket = input.stake;
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
            let validator: ValidatorSubstate = api.sys_read_substate_typed(handle)?;
            if validator.is_registered {
                let receiver = validator.manager;
                let key = validator.key;
                let validator_address = validator.address;
                let xrd_vault = Vault(validator.stake_xrd_vault_id);
                let xrd_amount = xrd_vault.sys_amount(api)?;

                api.call_method(
                    receiver.as_node_id(),
                    EPOCH_MANAGER_UPDATE_VALIDATOR_IDENT,
                    scrypto_encode(&EpochManagerUpdateValidatorInput {
                        validator_address,
                        update: UpdateValidator::Register(key, xrd_amount),
                    })
                    .unwrap(),
                )?;
            }
        }

        Runtime::emit_event(api, event)?;

        Ok(IndexedScryptoValue::from_typed(&lp_token_bucket))
    }

    pub fn unstake<Y>(
        receiver: &NodeId,
        input: &IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        let input: ValidatorUnstakeInput = input.as_typed().map_err(|e| {
            RuntimeError::InterpreterError(InterpreterError::ScryptoInputDecodeError(e))
        })?;

        // Prepare event and emit it once operations finish
        let event = {
            let amount = input.lp_tokens.sys_amount(api)?;
            UnstakeEvent {
                stake_units: amount,
            }
        };

        let handle = api.sys_lock_substate(
            receiver,
            &ValidatorOffset::Validator.into(),
            LockFlags::read_only(),
        )?;

        // Unstake
        let (unstake_bucket, _) = {
            let validator: ValidatorSubstate = api.sys_read_substate_typed(handle)?;

            let manager = validator.manager;
            let mut stake_vault = Vault(validator.stake_xrd_vault_id);
            let mut unstake_vault = Vault(validator.pending_xrd_withdraw_vault_id);
            let nft_resman = ResourceManager(validator.unstake_nft);
            let mut lp_token_resman = ResourceManager(validator.liquidity_token);

            let active_stake_amount = stake_vault.sys_amount(api)?;
            let total_lp_supply = lp_token_resman.total_supply(api)?;
            let lp_tokens = input.lp_tokens;
            let lp_token_amount = lp_tokens.sys_amount(api)?;
            let xrd_amount = if total_lp_supply.is_zero() {
                Decimal::zero()
            } else {
                lp_token_amount * active_stake_amount / total_lp_supply
            };

            lp_token_resman.burn(lp_tokens, api)?;

            let manager_handle = api.sys_lock_substate(
                manager.as_node_id(),
                &EpochManagerOffset::EpochManager.into(),
                LockFlags::read_only(),
            )?;
            let epoch_manager: EpochManagerSubstate =
                api.sys_read_substate_typed(manager_handle)?;
            let current_epoch = epoch_manager.epoch;
            let epoch_unlocked = current_epoch + epoch_manager.num_unstake_epochs;
            api.sys_drop_lock(manager_handle)?;

            let data = UnstakeData {
                epoch_unlocked,
                amount: xrd_amount,
            };

            let bucket = stake_vault.sys_take(xrd_amount, api)?;
            unstake_vault.sys_put(bucket, api)?;
            nft_resman.mint_non_fungible_single_uuid(data, api)?
        };

        // Update Epoch Manager
        {
            let validator: ValidatorSubstate = api.sys_read_substate_typed(handle)?;
            let stake_vault = Vault(validator.stake_xrd_vault_id);
            if validator.is_registered {
                let stake_amount = stake_vault.sys_amount(api)?;
                let validator: ValidatorSubstate = api.sys_read_substate_typed(handle)?;
                let manager = validator.manager;
                let validator_address = validator.address;
                let update = if stake_amount.is_zero() {
                    UpdateValidator::Unregister
                } else {
                    UpdateValidator::Register(validator.key, stake_amount)
                };

                api.call_method(
                    manager.as_node_id(),
                    EPOCH_MANAGER_UPDATE_VALIDATOR_IDENT,
                    scrypto_encode(&EpochManagerUpdateValidatorInput {
                        validator_address,
                        update,
                    })
                    .unwrap(),
                )?;
            }
        };

        Runtime::emit_event(api, event)?;

        Ok(IndexedScryptoValue::from_typed(&unstake_bucket))
    }

    pub fn claim_xrd<Y>(
        receiver: &NodeId,
        input: &IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        let input: ValidatorClaimXrdInput = input.as_typed().map_err(|e| {
            RuntimeError::InterpreterError(InterpreterError::ScryptoInputDecodeError(e))
        })?;

        let handle = api.sys_lock_substate(
            receiver,
            &ValidatorOffset::Validator.into(),
            LockFlags::read_only(),
        )?;
        let validator: ValidatorSubstate = api.sys_read_substate_typed(handle)?;
        let mut nft_resman = ResourceManager(validator.unstake_nft);
        let resource_address = validator.unstake_nft;
        let manager = validator.manager;
        let mut unstake_vault = Vault(validator.pending_xrd_withdraw_vault_id);

        // TODO: Move this check into a more appropriate place
        let bucket = input.bucket;
        if !resource_address.eq(&bucket.sys_resource_address(api)?) {
            return Err(RuntimeError::ApplicationError(
                ApplicationError::ValidatorError(ValidatorError::InvalidClaimResource),
            ));
        }

        let current_epoch = {
            let mgr_handle = api.sys_lock_substate(
                manager.as_node_id(),
                &EpochManagerOffset::EpochManager.into(),
                LockFlags::read_only(),
            )?;
            let mgr_substate: EpochManagerSubstate = api.sys_read_substate_typed(mgr_handle)?;
            let epoch = mgr_substate.epoch;
            api.sys_drop_lock(mgr_handle)?;
            epoch
        };

        let mut unstake_amount = Decimal::zero();

        for id in bucket.sys_non_fungible_local_ids(api)? {
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

        let amount = claimed_bucket.sys_amount(api)?;
        Runtime::emit_event(
            api,
            ClaimXrdEvent {
                claimed_xrd: amount,
            },
        )?;

        Ok(IndexedScryptoValue::from_typed(&claimed_bucket))
    }

    pub fn update_key<Y>(
        receiver: &NodeId,
        input: &IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        let input: ValidatorUpdateKeyInput = input.as_typed().map_err(|e| {
            RuntimeError::InterpreterError(InterpreterError::ScryptoInputDecodeError(e))
        })?;

        let handle = api.sys_lock_substate(
            receiver,
            &ValidatorOffset::Validator.into(),
            LockFlags::MUTABLE,
        )?;
        let mut validator: ValidatorSubstate = api.sys_read_substate_typed(handle)?;
        validator.key = input.key;
        let key = validator.key;
        let manager = validator.manager;
        let validator_address = validator.address;
        api.sys_write_substate_typed(handle, &validator)?;

        // Update Epoch Manager
        {
            let stake_vault = Vault(validator.stake_xrd_vault_id);
            if validator.is_registered {
                let stake_amount = stake_vault.sys_amount(api)?;
                if !stake_amount.is_zero() {
                    let update = UpdateValidator::Register(key, stake_amount);
                    api.call_method(
                        manager.as_node_id(),
                        EPOCH_MANAGER_UPDATE_VALIDATOR_IDENT,
                        scrypto_encode(&EpochManagerUpdateValidatorInput {
                            validator_address,
                            update,
                        })
                        .unwrap(),
                    )?;
                }
            }
        }

        Ok(IndexedScryptoValue::from_typed(&()))
    }

    pub fn update_accept_delegated_stake<Y>(
        receiver: &NodeId,
        input: &IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        let input: ValidatorUpdateAcceptDelegatedStakeInput = input.as_typed().map_err(|e| {
            RuntimeError::InterpreterError(InterpreterError::ScryptoInputDecodeError(e))
        })?;

        let rule = if input.accept_delegated_stake {
            AccessRuleEntry::AccessRule(AccessRule::AllowAll)
        } else {
            AccessRuleEntry::Group("owner".to_string())
        };

        api.call_module_method(
            receiver,
            TypedModuleId::AccessRules,
            ACCESS_RULES_SET_METHOD_ACCESS_RULE_IDENT,
            scrypto_encode(&AccessRulesSetMethodAccessRuleInput {
                object_key: ObjectKey::SELF,
                method_key: MethodKey::new(TypedModuleId::ObjectState, VALIDATOR_STAKE_IDENT),
                rule,
            })
            .unwrap(),
        )?;

        Runtime::emit_event(
            api,
            UpdateAcceptingStakeDelegationStateEvent {
                accepts_delegation: input.accept_delegated_stake,
            },
        )?;

        Ok(IndexedScryptoValue::from_typed(&()))
    }
}

struct SecurifiedValidator;

impl SecurifiedAccessRules for SecurifiedValidator {
    const SECURIFY_IDENT: Option<&'static str> = None;
    const OWNER_GROUP_NAME: &'static str = "owner";
    const OWNER_TOKEN: ResourceAddress = VALIDATOR_OWNER_TOKEN;

    fn non_owner_methods() -> Vec<(&'static str, MethodType)> {
        let non_fungible_global_id = NonFungibleGlobalId::package_actor(EPOCH_MANAGER_PACKAGE);
        vec![
            (VALIDATOR_UNSTAKE_IDENT, MethodType::Public),
            (VALIDATOR_CLAIM_XRD_IDENT, MethodType::Public),
            (
                VALIDATOR_STAKE_IDENT,
                MethodType::Custom(
                    AccessRuleEntry::group(Self::OWNER_GROUP_NAME),
                    AccessRuleEntry::AccessRule(rule!(require(non_fungible_global_id))),
                ),
            ),
        ]
    }
}

pub(crate) struct ValidatorCreator;

impl ValidatorCreator {
    fn create_liquidity_token_with_initial_amount<Y>(
        amount: Decimal,
        api: &mut Y,
    ) -> Result<(ResourceAddress, Bucket), RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let mut liquidity_token_auth = BTreeMap::new();
        let non_fungible_id =
            NonFungibleLocalId::bytes(scrypto_encode(&EPOCH_MANAGER_PACKAGE).unwrap()).unwrap();
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
        Y: ClientApi<RuntimeError>,
    {
        let mut liquidity_token_auth = BTreeMap::new();
        let non_fungible_local_id =
            NonFungibleLocalId::bytes(scrypto_encode(&EPOCH_MANAGER_PACKAGE).unwrap()).unwrap();
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
        Y: ClientApi<RuntimeError>,
    {
        let mut unstake_token_auth = BTreeMap::new();
        let non_fungible_local_id =
            NonFungibleLocalId::bytes(scrypto_encode(&EPOCH_MANAGER_PACKAGE).unwrap()).unwrap();
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

        let unstake_resource_manager =
            ResourceManager::new_non_fungible::<UnstakeData, Y, RuntimeError>(
                NonFungibleIdType::UUID,
                BTreeMap::new(),
                unstake_token_auth,
                api,
            )?;

        Ok(unstake_resource_manager.0)
    }

    pub fn create_with_initial_stake<Y>(
        manager: ComponentAddress,
        key: EcdsaSecp256k1PublicKey,
        initial_stake: Bucket,
        is_registered: bool,
        api: &mut Y,
    ) -> Result<(ComponentAddress, Bucket, Bucket), RuntimeError>
    where
        Y: KernelNodeApi + ClientApi<RuntimeError>,
    {
        let global_node_id = api.kernel_allocate_node_id(EntityType::GlobalValidator)?;
        let address: ComponentAddress = ComponentAddress::new_unchecked(global_node_id.into());
        let initial_liquidity_amount = initial_stake.sys_amount(api)?;
        let mut stake_vault = Vault::sys_new(RADIX_TOKEN, api)?;
        stake_vault.sys_put(initial_stake, api)?;
        let unstake_vault = Vault::sys_new(RADIX_TOKEN, api)?;
        let unstake_nft = Self::create_unstake_nft(api)?;
        let (liquidity_token, liquidity_bucket) =
            Self::create_liquidity_token_with_initial_amount(initial_liquidity_amount, api)?;

        let substate = ValidatorSubstate {
            manager,
            key,
            address,
            liquidity_token,
            unstake_nft,
            stake_xrd_vault_id: stake_vault.0,
            pending_xrd_withdraw_vault_id: unstake_vault.0,
            is_registered,
        };

        let validator_id = api.new_object(
            VALIDATOR_BLUEPRINT,
            vec![scrypto_encode(&substate).unwrap()],
        )?;

        let (access_rules, owner_token_bucket) = SecurifiedValidator::create_securified(api)?;
        let metadata = Metadata::sys_create(api)?;
        let royalty = ComponentRoyalty::sys_create(RoyaltyConfig::default(), api)?;

        api.globalize_with_address(
            validator_id,
            btreemap!(
                TypedModuleId::AccessRules => access_rules.0.0,
                TypedModuleId::Metadata => metadata.0,
                TypedModuleId::Royalty => royalty.0,
            ),
            address.into(),
        )?;
        Ok((address.into(), liquidity_bucket, owner_token_bucket))
    }

    pub fn create<Y>(
        manager: ComponentAddress,
        key: EcdsaSecp256k1PublicKey,
        is_registered: bool,
        api: &mut Y,
    ) -> Result<(ComponentAddress, Bucket), RuntimeError>
    where
        Y: KernelNodeApi + ClientApi<RuntimeError>,
    {
        let global_node_id = api.kernel_allocate_node_id(EntityType::GlobalValidator)?;
        let address = ComponentAddress::new_unchecked(global_node_id.into());
        let stake_vault = Vault::sys_new(RADIX_TOKEN, api)?;
        let unstake_vault = Vault::sys_new(RADIX_TOKEN, api)?;
        let unstake_nft = Self::create_unstake_nft(api)?;
        let liquidity_token = Self::create_liquidity_token(api)?;

        let substate = ValidatorSubstate {
            manager,
            key,
            address,
            liquidity_token,
            unstake_nft,
            stake_xrd_vault_id: stake_vault.0,
            pending_xrd_withdraw_vault_id: unstake_vault.0,
            is_registered,
        };

        let validator_id = api.new_object(
            VALIDATOR_BLUEPRINT,
            vec![scrypto_encode(&substate).unwrap()],
        )?;

        let (access_rules, owner_token_bucket) = SecurifiedValidator::create_securified(api)?;
        let metadata = Metadata::sys_create(api)?;
        let royalty = ComponentRoyalty::sys_create(RoyaltyConfig::default(), api)?;

        api.globalize_with_address(
            validator_id,
            btreemap!(
                TypedModuleId::AccessRules => access_rules.0.0,
                TypedModuleId::Metadata => metadata.0,
                TypedModuleId::Royalty => royalty.0,
            ),
            address.into(),
        )?;
        Ok((address.into(), owner_token_bucket))
    }
}
