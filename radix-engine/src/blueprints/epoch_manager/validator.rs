use crate::blueprints::epoch_manager::*;
use crate::blueprints::util::SecurifiedAccessRules;
use crate::errors::ApplicationError;
use crate::errors::RuntimeError;
use crate::kernel::kernel_api::KernelNodeApi;
use crate::types::*;
use native_sdk::modules::metadata::Metadata;
use native_sdk::modules::royalty::ComponentRoyalty;
use native_sdk::resource::{ResourceManager, SysBucket, Vault};
use native_sdk::runtime::Runtime;
use radix_engine_interface::api::actor_sorted_index_api::SortedKey;
use radix_engine_interface::api::field_lock_api::LockFlags;
use radix_engine_interface::api::node_modules::auth::{
    AccessRulesSetGroupAccessRuleInput, ACCESS_RULES_SET_GROUP_ACCESS_RULE_IDENT,
};
use radix_engine_interface::api::object_api::ObjectModuleId;
use radix_engine_interface::api::{ClientApi, OBJECT_HANDLE_OUTER_OBJECT, OBJECT_HANDLE_SELF};
use radix_engine_interface::blueprints::epoch_manager::*;
use radix_engine_interface::blueprints::resource::*;
use radix_engine_interface::rule;

use super::{
    ClaimXrdEvent, RegisterValidatorEvent, StakeEvent, UnregisterValidatorEvent, UnstakeEvent,
    UpdateAcceptingStakeDelegationStateEvent,
};

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub struct ValidatorSubstate {
    pub sorted_key: Option<SortedKey>,
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
    pub fn register<Y>(api: &mut Y) -> Result<(), RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        Self::register_update(true, api)
    }

    pub fn unregister<Y>(api: &mut Y) -> Result<(), RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        Self::register_update(false, api)
    }

    pub fn stake<Y>(stake: Bucket, api: &mut Y) -> Result<Bucket, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        // Prepare the event and emit it once the operations succeed
        let event = {
            let amount = stake.sys_amount(api)?;
            StakeEvent { xrd_staked: amount }
        };

        let handle = api.actor_lock_field(
            OBJECT_HANDLE_SELF,
            ValidatorField::Validator.into(),
            LockFlags::MUTABLE,
        )?;

        let mut validator: ValidatorSubstate = api.field_lock_read_typed(handle)?;

        // Stake
        let (lp_token_bucket, new_stake_amount) = {
            let mut lp_token_resman = ResourceManager(validator.liquidity_token);
            let mut xrd_vault = Vault(validator.stake_xrd_vault_id);

            let total_lp_supply = lp_token_resman.total_supply(api)?;
            let active_stake_amount = xrd_vault.sys_amount(api)?;
            let xrd_bucket = stake;
            let stake_amount = xrd_bucket.sys_amount(api)?;
            let lp_mint_amount = if active_stake_amount.is_zero() {
                stake_amount
            } else {
                stake_amount * total_lp_supply / active_stake_amount
            };

            let lp_token_bucket = lp_token_resman.mint_fungible(lp_mint_amount, api)?;
            xrd_vault.sys_put(xrd_bucket, api)?;
            let new_stake_amount = xrd_vault.sys_amount(api)?;
            (lp_token_bucket, new_stake_amount)
        };

        // Update EpochManager
        let new_index_key =
            Self::index_update(&validator, validator.is_registered, new_stake_amount, api)?;

        validator.sorted_key = new_index_key;
        api.field_lock_write_typed(handle, &validator)?;
        Runtime::emit_event(api, event)?;

        Ok(lp_token_bucket)
    }

    pub fn unstake<Y>(lp_tokens: Bucket, api: &mut Y) -> Result<Bucket, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        // Prepare event and emit it once operations finish
        let event = {
            let amount = lp_tokens.sys_amount(api)?;
            UnstakeEvent {
                stake_units: amount,
            }
        };

        let handle = api.actor_lock_field(
            OBJECT_HANDLE_SELF,
            ValidatorField::Validator.into(),
            LockFlags::MUTABLE,
        )?;
        let mut validator: ValidatorSubstate = api.field_lock_read_typed(handle)?;

        // Unstake
        let (unstake_bucket, new_stake_amount) = {
            let mut stake_vault = Vault(validator.stake_xrd_vault_id);
            let mut unstake_vault = Vault(validator.pending_xrd_withdraw_vault_id);
            let nft_resman = ResourceManager(validator.unstake_nft);
            let mut lp_token_resman = ResourceManager(validator.liquidity_token);

            let active_stake_amount = stake_vault.sys_amount(api)?;
            let total_lp_supply = lp_token_resman.total_supply(api)?;
            let lp_token_amount = lp_tokens.sys_amount(api)?;
            let xrd_amount = if total_lp_supply.is_zero() {
                Decimal::zero()
            } else {
                lp_token_amount * active_stake_amount / total_lp_supply
            };

            lp_token_resman.burn(lp_tokens, api)?;

            let manager_handle = api.actor_lock_field(
                OBJECT_HANDLE_OUTER_OBJECT,
                EpochManagerField::EpochManager.into(),
                LockFlags::read_only(),
            )?;
            let epoch_manager: EpochManagerSubstate = api.field_lock_read_typed(manager_handle)?;
            let current_epoch = epoch_manager.epoch;

            let config_handle = api.actor_lock_field(
                OBJECT_HANDLE_OUTER_OBJECT,
                EpochManagerField::Config.into(),
                LockFlags::read_only(),
            )?;
            let config: EpochManagerConfigSubstate = api.field_lock_read_typed(config_handle)?;
            let epoch_unlocked = current_epoch + config.num_unstake_epochs;

            api.field_lock_release(manager_handle)?;

            let data = UnstakeData {
                epoch_unlocked,
                amount: xrd_amount,
            };

            let bucket = stake_vault.sys_take(xrd_amount, api)?;
            unstake_vault.sys_put(bucket, api)?;
            let (unstake_bucket, _) = nft_resman.mint_non_fungible_single_uuid(data, api)?;

            let new_stake_amount = stake_vault.sys_amount(api)?;

            (unstake_bucket, new_stake_amount)
        };

        // Update EpochManager
        let new_index_key =
            Self::index_update(&validator, validator.is_registered, new_stake_amount, api)?;

        validator.sorted_key = new_index_key;
        api.field_lock_write_typed(handle, &validator)?;
        Runtime::emit_event(api, event)?;

        Ok(unstake_bucket)
    }

    fn register_update<Y>(new_registered: bool, api: &mut Y) -> Result<(), RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let substate_key = ValidatorField::Validator.into();
        let handle = api.actor_lock_field(OBJECT_HANDLE_SELF, substate_key, LockFlags::MUTABLE)?;

        let mut validator: ValidatorSubstate = api.field_lock_read_typed(handle)?;
        // No update
        if validator.is_registered == new_registered {
            return Ok(());
        }

        let stake_amount = {
            let stake_vault = Vault(validator.stake_xrd_vault_id);
            stake_vault.sys_amount(api)?
        };

        let index_key = Self::index_update(&validator, new_registered, stake_amount, api)?;

        validator.is_registered = new_registered;
        validator.sorted_key = index_key;
        api.field_lock_write_typed(handle, &validator)?;

        if new_registered {
            Runtime::emit_event(api, RegisterValidatorEvent)?;
        } else {
            Runtime::emit_event(api, UnregisterValidatorEvent)?;
        }

        return Ok(());
    }

    fn index_update<Y>(
        validator: &ValidatorSubstate,
        new_registered: bool,
        new_stake_amount: Decimal,
        api: &mut Y,
    ) -> Result<Option<SortedKey>, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let validator_address: ComponentAddress =
            ComponentAddress::new_or_panic(api.actor_get_global_address()?.into());
        let new_sorted_key =
            Self::to_sorted_key(new_registered, new_stake_amount, validator_address);

        let update = if let Some(cur_index_key) = &validator.sorted_key {
            if let Some(new_index_key) = &new_sorted_key {
                Some(UpdateSecondaryIndex::UpdateStake {
                    index_key: cur_index_key.clone(),
                    new_index_key: new_index_key.clone(),
                    new_stake_amount,
                })
            } else {
                Some(UpdateSecondaryIndex::Remove {
                    index_key: cur_index_key.clone(),
                })
            }
        } else {
            if let Some(new_index_key) = &new_sorted_key {
                Some(UpdateSecondaryIndex::Create {
                    index_key: new_index_key.clone(),
                    stake: new_stake_amount,
                    primary: validator_address,
                    key: validator.key,
                })
            } else {
                None
            }
        };

        if let Some(update) = update {
            Self::update_validator(update, api)?;
        }

        Ok(new_sorted_key)
    }

    pub fn claim_xrd<Y>(bucket: Bucket, api: &mut Y) -> Result<Bucket, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let handle = api.actor_lock_field(
            OBJECT_HANDLE_SELF,
            ValidatorField::Validator.into(),
            LockFlags::read_only(),
        )?;
        let validator: ValidatorSubstate = api.field_lock_read_typed(handle)?;
        let mut nft_resman = ResourceManager(validator.unstake_nft);
        let resource_address = validator.unstake_nft;
        let mut unstake_vault = Vault(validator.pending_xrd_withdraw_vault_id);

        // TODO: Move this check into a more appropriate place
        if !resource_address.eq(&bucket.sys_resource_address(api)?) {
            return Err(RuntimeError::ApplicationError(
                ApplicationError::ValidatorError(ValidatorError::InvalidClaimResource),
            ));
        }

        let current_epoch = {
            let mgr_handle = api.actor_lock_field(
                OBJECT_HANDLE_OUTER_OBJECT,
                EpochManagerField::EpochManager.into(),
                LockFlags::read_only(),
            )?;
            let mgr_substate: EpochManagerSubstate = api.field_lock_read_typed(mgr_handle)?;
            let epoch = mgr_substate.epoch;
            api.field_lock_release(mgr_handle)?;
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

        Ok(claimed_bucket)
    }

    pub fn update_key<Y>(key: EcdsaSecp256k1PublicKey, api: &mut Y) -> Result<(), RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let handle = api.actor_lock_field(
            OBJECT_HANDLE_SELF,
            ValidatorField::Validator.into(),
            LockFlags::MUTABLE,
        )?;
        let mut validator: ValidatorSubstate = api.field_lock_read_typed(handle)?;

        // Update Epoch Manager
        {
            if let Some(index_key) = &validator.sorted_key {
                let update = UpdateSecondaryIndex::UpdatePublicKey {
                    index_key: index_key.clone(),
                    key,
                };

                Self::update_validator(update, api)?;
            }
        }

        validator.key = key;
        api.field_lock_write_typed(handle, &validator)?;

        Ok(())
    }

    pub fn update_accept_delegated_stake<Y>(
        receiver: &NodeId,
        accept_delegated_stake: bool,
        api: &mut Y,
    ) -> Result<(), RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let rule = if accept_delegated_stake {
            AccessRule::AllowAll
        } else {
            rule!(require("owner"))
        };

        api.call_method_advanced(
            receiver,
            false,
            ObjectModuleId::AccessRules,
            ACCESS_RULES_SET_GROUP_ACCESS_RULE_IDENT,
            scrypto_encode(&AccessRulesSetGroupAccessRuleInput {
                object_key: ObjectKey::SELF,
                name: "stake".to_string(),
                rule,
            })
            .unwrap(),
        )?;

        Runtime::emit_event(
            api,
            UpdateAcceptingStakeDelegationStateEvent {
                accepts_delegation: accept_delegated_stake,
            },
        )?;

        Ok(())
    }

    fn to_sorted_key(
        registered: bool,
        stake: Decimal,
        address: ComponentAddress,
    ) -> Option<SortedKey> {
        if !registered || stake.is_zero() {
            None
        } else {
            Some(SortedKey::new(
                create_sort_prefix_from_stake(stake),
                scrypto_encode(&address).unwrap(),
            ))
        }
    }

    pub(crate) fn update_validator<Y>(
        update: UpdateSecondaryIndex,
        api: &mut Y,
    ) -> Result<(), RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        match update {
            UpdateSecondaryIndex::Create {
                index_key,
                primary: address,
                key,
                stake,
            } => {
                api.actor_sorted_index_insert_typed(
                    OBJECT_HANDLE_OUTER_OBJECT,
                    EPOCH_MANAGER_REGISTERED_VALIDATORS_BY_STAKE_INDEX,
                    index_key,
                    (address, Validator { key, stake }),
                )?;
            }
            UpdateSecondaryIndex::UpdatePublicKey { index_key, key } => {
                let (address, mut validator) = api
                    .actor_sorted_index_remove_typed::<(ComponentAddress, Validator)>(
                        OBJECT_HANDLE_OUTER_OBJECT,
                        EPOCH_MANAGER_REGISTERED_VALIDATORS_BY_STAKE_INDEX,
                        &index_key,
                    )?
                    .unwrap();
                validator.key = key;
                api.actor_sorted_index_insert_typed(
                    OBJECT_HANDLE_OUTER_OBJECT,
                    EPOCH_MANAGER_REGISTERED_VALIDATORS_BY_STAKE_INDEX,
                    index_key,
                    (address, validator),
                )?;
            }
            UpdateSecondaryIndex::UpdateStake {
                index_key,
                new_index_key,
                new_stake_amount,
            } => {
                let (address, mut validator) = api
                    .actor_sorted_index_remove_typed::<(ComponentAddress, Validator)>(
                        OBJECT_HANDLE_OUTER_OBJECT,
                        EPOCH_MANAGER_REGISTERED_VALIDATORS_BY_STAKE_INDEX,
                        &index_key,
                    )?
                    .unwrap();
                validator.stake = new_stake_amount;
                api.actor_sorted_index_insert_typed(
                    OBJECT_HANDLE_OUTER_OBJECT,
                    EPOCH_MANAGER_REGISTERED_VALIDATORS_BY_STAKE_INDEX,
                    new_index_key,
                    (address, validator),
                )?;
            }
            UpdateSecondaryIndex::Remove { index_key } => {
                api.actor_sorted_index_remove(
                    OBJECT_HANDLE_OUTER_OBJECT,
                    EPOCH_MANAGER_REGISTERED_VALIDATORS_BY_STAKE_INDEX,
                    &index_key,
                )?;
            }
        }

        Ok(())
    }
}

fn create_sort_prefix_from_stake(stake: Decimal) -> u16 {
    // Note: XRD max supply is 24bn
    // 24bn / MAX::16 = 366210.9375 - so 100k as a divisor here is sensible.
    // If all available XRD was staked to one validator, they'd have 3.6 * u16::MAX * 100k stake
    // In reality, validators will have far less than u16::MAX * 100k stake, but let's handle that case just in case
    let stake_100k = stake / Decimal::from(100000);
    let stake_100k_whole_units = (stake_100k / Decimal::from(10).powi(Decimal::SCALE.into())).0;
    let stake_u16 = if stake_100k_whole_units > BnumI256::from(u16::MAX) {
        u16::MAX
    } else {
        stake_100k_whole_units.try_into().unwrap()
    };
    // We invert the key because we need high stake to appear first and it's ordered ASC
    u16::MAX - stake_u16
}

struct SecurifiedValidator;

impl SecurifiedAccessRules for SecurifiedValidator {
    const OWNER_BADGE: ResourceAddress = VALIDATOR_OWNER_BADGE;
    const OWNER_AUTHORITY: &'static str = "owner";
    const SECURIFY_AUTHORITY: Option<&'static str> = None;

    fn method_authorities() -> MethodAuthorities {
        let mut method_authorities = MethodAuthorities::new();
        method_authorities.set_public(VALIDATOR_UNSTAKE_IDENT);
        method_authorities.set_public(VALIDATOR_CLAIM_XRD_IDENT);
        method_authorities.set_main_method_authority(VALIDATOR_STAKE_IDENT, "stake");
        method_authorities.set_main_method_authority(VALIDATOR_REGISTER_IDENT, "owner");
        method_authorities.set_main_method_authority(VALIDATOR_UNREGISTER_IDENT, "owner");
        method_authorities.set_main_method_authority(VALIDATOR_UPDATE_KEY_IDENT, "owner");
        method_authorities.set_main_method_authority(VALIDATOR_UPDATE_ACCEPT_DELEGATED_STAKE_IDENT, "owner");
        method_authorities
    }

    fn authority_rules() -> AuthorityRules {
        let mut authority_rules = AuthorityRules::new();
        authority_rules.set_rule("stake", rule!(require("owner")), rule!(require(package_of_direct_caller(EPOCH_MANAGER_PACKAGE))));
        authority_rules.set_rule("update_metadata", rule!(require("owner")), rule!(deny_all));
        authority_rules
    }
}

pub(crate) struct ValidatorCreator;

impl ValidatorCreator {
    fn create_liquidity_token<Y>(
        address: GlobalAddress,
        api: &mut Y,
    ) -> Result<ResourceAddress, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let mut liquidity_token_auth = BTreeMap::new();
        liquidity_token_auth.insert(
            Mint,
            (rule!(require(global_caller(address))), rule!(deny_all)),
        );
        liquidity_token_auth.insert(
            Burn,
            (rule!(require(global_caller(address))), rule!(deny_all)),
        );
        liquidity_token_auth.insert(Withdraw, (rule!(allow_all), rule!(deny_all)));
        liquidity_token_auth.insert(Deposit, (rule!(allow_all), rule!(deny_all)));

        let liquidity_token_resource_manager =
            ResourceManager::new_fungible(18, BTreeMap::new(), liquidity_token_auth, api)?;

        Ok(liquidity_token_resource_manager.0)
    }

    fn create_unstake_nft<Y>(
        address: GlobalAddress,
        api: &mut Y,
    ) -> Result<ResourceAddress, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let mut unstake_token_auth = BTreeMap::new();

        unstake_token_auth.insert(
            Mint,
            (rule!(require(global_caller(address))), rule!(deny_all)),
        );
        unstake_token_auth.insert(
            Burn,
            (rule!(require(global_caller(address))), rule!(deny_all)),
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

    pub fn create<Y>(
        key: EcdsaSecp256k1PublicKey,
        is_registered: bool,
        api: &mut Y,
    ) -> Result<(ComponentAddress, Bucket), RuntimeError>
    where
        Y: KernelNodeApi + ClientApi<RuntimeError>,
    {
        let address = GlobalAddress::new_or_panic(
            api.kernel_allocate_node_id(EntityType::GlobalValidator)?.0,
        );

        let stake_vault = Vault::sys_new(RADIX_TOKEN, api)?;
        let unstake_vault = Vault::sys_new(RADIX_TOKEN, api)?;
        let unstake_nft = Self::create_unstake_nft(address, api)?;
        let liquidity_token = Self::create_liquidity_token(address, api)?;

        let substate = ValidatorSubstate {
            sorted_key: None,
            key,
            liquidity_token,
            unstake_nft,
            stake_xrd_vault_id: stake_vault.0,
            pending_xrd_withdraw_vault_id: unstake_vault.0,
            is_registered,
        };

        let validator_id = api.new_simple_object(
            VALIDATOR_BLUEPRINT,
            vec![scrypto_encode(&substate).unwrap()],
        )?;

        let (access_rules, owner_token_bucket) = SecurifiedValidator::create_securified(api)?;
        let metadata = Metadata::sys_create(api)?;
        let royalty = ComponentRoyalty::sys_create(RoyaltyConfig::default(), api)?;

        api.globalize_with_address(
            btreemap!(
                ObjectModuleId::Main => validator_id,
                ObjectModuleId::AccessRules => access_rules.0.0,
                ObjectModuleId::Metadata => metadata.0,
                ObjectModuleId::Royalty => royalty.0,
            ),
            address,
        )?;

        Ok((
            ComponentAddress::new_or_panic(address.into()),
            owner_token_bucket,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sort_key_is_calculated_correctly() {
        assert_eq!(create_sort_prefix_from_stake(Decimal::ZERO), u16::MAX);
        assert_eq!(create_sort_prefix_from_stake(dec!(99_999)), u16::MAX);
        assert_eq!(create_sort_prefix_from_stake(dec!(100_000)), u16::MAX - 1);
        assert_eq!(create_sort_prefix_from_stake(dec!(199_999)), u16::MAX - 1);
        assert_eq!(create_sort_prefix_from_stake(dec!(200_000)), u16::MAX - 2);
        // https://learn.radixdlt.com/article/start-here-radix-tokens-and-tokenomics
        let max_xrd_supply = dec!(24) * dec!(10).powi(12);
        assert_eq!(create_sort_prefix_from_stake(max_xrd_supply), 0);
    }
}
