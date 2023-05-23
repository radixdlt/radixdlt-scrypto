use crate::blueprints::epoch_manager::*;
use crate::blueprints::util::SecurifiedAccessRules;
use crate::errors::ApplicationError;
use crate::errors::RuntimeError;
use crate::kernel::kernel_api::KernelNodeApi;
use crate::types::*;
use native_sdk::modules::metadata::Metadata;
use native_sdk::modules::royalty::ComponentRoyalty;
use native_sdk::resource::NativeVault;
use native_sdk::resource::ResourceManager;
use native_sdk::resource::{NativeBucket, NativeNonFungibleBucket};
use native_sdk::runtime::Runtime;
use radix_engine_interface::api::actor_sorted_index_api::SortedKey;
use radix_engine_interface::api::field_lock_api::LockFlags;
use radix_engine_interface::api::node_modules::auth::{
    AccessRulesSetAuthorityRuleInput, ACCESS_RULES_SET_AUTHORITY_RULE_IDENT,
};
use radix_engine_interface::api::object_api::ObjectModuleId;
use radix_engine_interface::api::{ClientApi, OBJECT_HANDLE_OUTER_OBJECT, OBJECT_HANDLE_SELF};
use radix_engine_interface::blueprints::epoch_manager::*;
use radix_engine_interface::blueprints::resource::*;
use radix_engine_interface::rule;
use sbor::rust::mem;

use super::{
    ClaimXrdEvent, RegisterValidatorEvent, StakeEvent, UnregisterValidatorEvent, UnstakeEvent,
    UpdateAcceptingStakeDelegationStateEvent,
};

/// A performance-driven limit on the number of simultaneously pending "delayed withdrawal"
/// operations on any validator's owner's stake units vault.
pub const OWNER_STAKE_UNITS_PENDING_WITHDRAWALS_LIMIT: usize = 100;

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub struct ValidatorSubstate {
    /// A key used internally for storage of registered validators sorted by their stake descending.
    /// It is only useful when the validator is registered and has non-zero stake - hence, the field
    /// is [`None`] otherwise.
    /// Note: in theory, this value could be always computed from the [`is_registered`] status and
    /// the amount stored in [`stake_xrd_vault_id`]; we simply keep it cached to simplify certain
    /// updates.
    pub sorted_key: Option<SortedKey>,

    /// This validator's public key.
    pub key: EcdsaSecp256k1PublicKey,

    /// Whether this validator is currently interested in participating in the consensus.
    pub is_registered: bool,

    /// A type of fungible resource representing stake units specific to this validator.
    /// Conceptually, "staking to validator A" means "contributing to the validator's staking pool,
    /// and receiving the validator's stake units which act as the pool units for the staking pool".
    pub stake_unit_resource: ResourceAddress,

    /// A vault holding the XRDs currently staked to this validator.
    pub stake_xrd_vault_id: Own,

    /// A type of non-fungible token used as a receipt for unstaked stake units.
    /// Unstaking burns the SUs and inactivates the staked XRDs (i.e. moves it from the regular
    /// [`stake_xrd_vault_id`] to the [`pending_xrd_withdraw_vault_id`]), and then requires to claim
    /// the XRDs using this NFT after a delay (see [`UnstakeData.epoch_unlocked`]).
    pub unstake_nft: ResourceAddress,

    /// A vault holding the XRDs that were unstaked (see the [`unstake_nft`]) but not yet claimed.
    pub pending_xrd_withdraw_vault_id: Own,

    /// A vault holding the SUs that this validator's owner voluntarily decided to temporarily lock
    /// here, as a public display of their confidence in this validator's future reliability.
    /// Withdrawing SUs from this vault is subject to a delay (which is configured separately from
    /// the regular unstaking delay, see [`EpochManagerConfigSubstate.num_owner_stake_units_unlock_epochs`]).
    /// This vault is private to the owner (i.e. the owner's badge is required for any interaction
    /// with this vault).
    pub locked_owner_stake_unit_vault_id: Own,

    /// A vault holding the SUs which the owner has decided to withdraw from their "public display"
    /// vault (see [`locked_owner_stake_unit_vault_id`]) but which have not yet been unlocked after
    /// the mandatory delay (see [`pending_owner_stake_unit_withdrawals`]).
    pub pending_owner_stake_unit_unlock_vault_id: Own,

    /// All currently pending "delayed withdrawal" operations of the owner's stake units vault (see
    /// [`locked_owner_stake_unit_vault_id`]).
    /// This maps an epoch number to an amount of stake units that become unlocked at that epoch.
    /// Note: because of performance considerations, a maximum size of this map is limited to
    /// [`OWNER_STAKE_UNITS_PENDING_WITHDRAWALS_LIMIT`]: starting another withdrawal will first
    /// attempt to move any already-available amount to [`already_unlocked_owner_stake_unit_amount`]
    /// and only then will fail if the limit is exceeded.
    pub pending_owner_stake_unit_withdrawals: BTreeMap<u64, Decimal>,

    /// An amount of owner's stake units that has already waited for a sufficient number of epochs
    /// in the [`pending_owner_stake_unit_withdrawals`] and was automatically moved from there.
    /// The very next [`finish_unlock_owner_stake_units()`] operation will release this amount.
    pub already_unlocked_owner_stake_unit_amount: Decimal,
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub struct UnstakeData {
    /// An epoch number at (or after) which the pending unstaked XRD may be claimed.
    /// Note: on unstake, it is fixed to be [`EpochManagerConfigSubstate.num_unstake_epochs`] away.
    epoch_unlocked: u64,

    /// An XRD amount to be claimed.
    amount: Decimal,
}

impl NonFungibleData for UnstakeData {
    const MUTABLE_FIELDS: &'static [&'static str] = &[];
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum ValidatorError {
    InvalidClaimResource,
    EpochUnlockHasNotOccurredYet,
    PendingOwnerStakeWithdrawalLimitReached,
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

    pub fn stake<Y>(xrd_bucket: Bucket, api: &mut Y) -> Result<Bucket, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let xrd_bucket_amount = xrd_bucket.amount(api)?;

        let handle = api.actor_lock_field(
            OBJECT_HANDLE_SELF,
            ValidatorField::Validator.into(),
            LockFlags::MUTABLE,
        )?;

        let mut validator: ValidatorSubstate = api.field_lock_read_typed(handle)?;

        // Stake
        let (stake_unit_bucket, new_stake_amount) = {
            let mut stake_unit_resman = ResourceManager(validator.stake_unit_resource);
            let mut xrd_vault = Vault(validator.stake_xrd_vault_id);

            let total_stake_unit_supply = stake_unit_resman.total_supply(api)?;
            let active_stake_amount = xrd_vault.amount(api)?;
            let stake_unit_mint_amount = if active_stake_amount.is_zero() {
                xrd_bucket_amount
            } else {
                xrd_bucket_amount * total_stake_unit_supply / active_stake_amount
            };

            let stake_unit_bucket = stake_unit_resman.mint_fungible(stake_unit_mint_amount, api)?;
            xrd_vault.put(xrd_bucket, api)?;
            let new_stake_amount = xrd_vault.amount(api)?;
            (stake_unit_bucket, new_stake_amount)
        };

        // Update EpochManager
        let new_index_key =
            Self::index_update(&validator, validator.is_registered, new_stake_amount, api)?;

        validator.sorted_key = new_index_key;
        api.field_lock_write_typed(handle, &validator)?;

        Runtime::emit_event(
            api,
            StakeEvent {
                xrd_staked: xrd_bucket_amount,
            },
        )?;

        Ok(stake_unit_bucket)
    }

    pub fn unstake<Y>(stake_unit_bucket: Bucket, api: &mut Y) -> Result<Bucket, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let stake_unit_bucket_amount = stake_unit_bucket.amount(api)?;

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
            let mut stake_unit_resman = ResourceManager(validator.stake_unit_resource);

            let active_stake_amount = stake_vault.amount(api)?;
            let total_stake_unit_supply = stake_unit_resman.total_supply(api)?;
            let xrd_amount = if total_stake_unit_supply.is_zero() {
                Decimal::zero()
            } else {
                stake_unit_bucket_amount * active_stake_amount / total_stake_unit_supply
            };

            stake_unit_resman.burn(stake_unit_bucket, api)?;

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

            let bucket = stake_vault.take(xrd_amount, api)?;
            unstake_vault.put(bucket, api)?;
            let (unstake_bucket, _) = nft_resman.mint_non_fungible_single_uuid(data, api)?;

            let new_stake_amount = stake_vault.amount(api)?;

            (unstake_bucket, new_stake_amount)
        };

        // Update EpochManager
        let new_index_key =
            Self::index_update(&validator, validator.is_registered, new_stake_amount, api)?;

        validator.sorted_key = new_index_key;
        api.field_lock_write_typed(handle, &validator)?;

        Runtime::emit_event(
            api,
            UnstakeEvent {
                stake_units: stake_unit_bucket_amount,
            },
        )?;

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
            stake_vault.amount(api)?
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
        if !resource_address.eq(&bucket.resource_address(api)?) {
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

        for id in bucket.non_fungible_local_ids(api)? {
            let data: UnstakeData = nft_resman.get_non_fungible_data(id, api)?;
            if current_epoch < data.epoch_unlocked {
                return Err(RuntimeError::ApplicationError(
                    ApplicationError::ValidatorError(ValidatorError::EpochUnlockHasNotOccurredYet),
                ));
            }
            unstake_amount += data.amount;
        }
        nft_resman.burn(bucket, api)?;

        let claimed_bucket = unstake_vault.take(unstake_amount, api)?;

        let amount = claimed_bucket.amount(api)?;
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
            rule!(require_owner())
        };

        api.call_method_advanced(
            receiver,
            false,
            ObjectModuleId::AccessRules,
            ACCESS_RULES_SET_AUTHORITY_RULE_IDENT,
            scrypto_encode(&AccessRulesSetAuthorityRuleInput {
                object_key: ObjectKey::SELF,
                authority_key: AuthorityKey::main("stake"),
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

    /// Locks the given stake units in an internal "delayed withdrawal" vault (which is the owner's
    /// way of showing their commitment to running this validator in an orderly fashion - see
    /// [`ValidatorSubstate.locked_owner_stake_unit_vault_id`]).
    pub fn lock_owner_stake_units<Y>(
        stake_unit_bucket: Bucket,
        api: &mut Y,
    ) -> Result<(), RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let handle = api.actor_lock_field(
            OBJECT_HANDLE_SELF,
            ValidatorField::Validator.into(),
            LockFlags::read_only(),
        )?;
        let substate: ValidatorSubstate = api.field_lock_read_typed(handle)?;

        Vault(substate.locked_owner_stake_unit_vault_id).put(stake_unit_bucket, api)?;

        api.field_lock_release(handle)?;
        Ok(())
    }

    /// Starts the process of unlocking the owner's stake units stored in the internal vault.
    /// The requested amount of stake units (if available) will be ready for withdrawal after the
    /// network-configured [`EpochManagerConfigSubstate.num_owner_stake_units_unlock_epochs`] via a
    /// call to [`finish_unlock_owner_stake_units()`].
    pub fn start_unlock_owner_stake_units<Y>(
        requested_stake_unit_amount: Decimal,
        api: &mut Y,
    ) -> Result<(), RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        // read the current epoch (needed for a drive-by "finish unlocking" of available withdrawals)
        let epoch_manager_handle = api.actor_lock_field(
            OBJECT_HANDLE_OUTER_OBJECT,
            EpochManagerField::EpochManager.into(),
            LockFlags::read_only(),
        )?;
        let epoch_manager: EpochManagerSubstate =
            api.field_lock_read_typed(epoch_manager_handle)?;
        let current_epoch = epoch_manager.epoch;
        api.field_lock_release(epoch_manager_handle)?;

        // read the configured unlock epochs delay
        let config_handle = api.actor_lock_field(
            OBJECT_HANDLE_OUTER_OBJECT,
            EpochManagerField::Config.into(),
            LockFlags::read_only(),
        )?;
        let config: EpochManagerConfigSubstate = api.field_lock_read_typed(config_handle)?;
        let num_owner_stake_units_unlock_epochs = config.num_owner_stake_units_unlock_epochs;
        api.field_lock_release(config_handle)?;

        // begin the read+modify+write of the validator substate...
        let handle = api.actor_lock_field(
            OBJECT_HANDLE_SELF,
            ValidatorField::Validator.into(),
            LockFlags::MUTABLE,
        )?;
        let mut substate: ValidatorSubstate = api.field_lock_read_typed(handle)?;

        // - move the already-available withdrawals to a dedicated field
        let available_withdrawal_epochs = substate
            .pending_owner_stake_unit_withdrawals
            .range(..=current_epoch)
            .map(|(epoch, _available_amount)| epoch.clone())
            .collect::<Vec<_>>();
        for available_withdrawal_epoch in available_withdrawal_epochs {
            // no batch delete in a BTree
            let available_amount = substate
                .pending_owner_stake_unit_withdrawals
                .remove(&available_withdrawal_epoch)
                .expect("key was just returned by the iterator");
            substate.already_unlocked_owner_stake_unit_amount += available_amount;
        }

        // - insert the requested withdrawal as pending (if possible)
        let currently_pending_withdrawals = substate.pending_owner_stake_unit_withdrawals.len();
        if currently_pending_withdrawals >= OWNER_STAKE_UNITS_PENDING_WITHDRAWALS_LIMIT {
            return Err(RuntimeError::ApplicationError(
                ApplicationError::ValidatorError(
                    ValidatorError::PendingOwnerStakeWithdrawalLimitReached,
                ),
            ));
        }
        substate
            .pending_owner_stake_unit_withdrawals
            .entry(current_epoch + num_owner_stake_units_unlock_epochs)
            .and_modify(|pending_amount| pending_amount.add_assign(requested_stake_unit_amount))
            .or_insert(requested_stake_unit_amount);

        // ...end the read+modify+write of the validator substate
        let mut locked_owner_stake_unit_vault = Vault(substate.locked_owner_stake_unit_vault_id);
        let mut pending_owner_stake_unit_unlock_vault =
            Vault(substate.pending_owner_stake_unit_unlock_vault_id);
        api.field_lock_write_typed(handle, substate)?;

        // move the requested stake units from the "locked vault" to the "pending withdrawal vault"
        let pending_unlock_stake_unit_bucket =
            locked_owner_stake_unit_vault.take(requested_stake_unit_amount, api)?;
        pending_owner_stake_unit_unlock_vault.put(pending_unlock_stake_unit_bucket, api)?;

        api.field_lock_release(handle)?;
        Ok(())
    }

    /// Finishes the process of unlocking the owner's stake units by withdrawing *all* the pending
    /// amounts which have reached their target epoch and thus are already available (potentially
    /// none).
    pub fn finish_unlock_owner_stake_units<Y>(api: &mut Y) -> Result<Bucket, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        // read the current epoch
        let epoch_manager_handle = api.actor_lock_field(
            OBJECT_HANDLE_OUTER_OBJECT,
            EpochManagerField::EpochManager.into(),
            LockFlags::read_only(),
        )?;
        let epoch_manager: EpochManagerSubstate =
            api.field_lock_read_typed(epoch_manager_handle)?;
        let current_epoch = epoch_manager.epoch;
        api.field_lock_release(epoch_manager_handle)?;

        // drain the already-available withdrawals
        let handle = api.actor_lock_field(
            OBJECT_HANDLE_SELF,
            ValidatorField::Validator.into(),
            LockFlags::MUTABLE,
        )?;
        let mut substate: ValidatorSubstate = api.field_lock_read_typed(handle)?;

        let available_withdrawal_epochs = substate
            .pending_owner_stake_unit_withdrawals
            .range(..=current_epoch)
            .map(|(epoch, _available_amount)| epoch.clone())
            .collect::<Vec<_>>();
        let mut total_already_available_amount = mem::replace(
            &mut substate.already_unlocked_owner_stake_unit_amount,
            Decimal::zero(),
        );
        for available_withdrawal_epoch in available_withdrawal_epochs {
            // no batch delete in a BTree
            let available_amount = substate
                .pending_owner_stake_unit_withdrawals
                .remove(&available_withdrawal_epoch)
                .expect("key was just returned by the iterator");
            total_already_available_amount += available_amount;
        }

        let mut pending_owner_stake_unit_unlock_vault =
            Vault(substate.pending_owner_stake_unit_unlock_vault_id);
        api.field_lock_write_typed(handle, substate)?;

        // return the already-available withdrawals
        let already_available_stake_unit_bucket =
            pending_owner_stake_unit_unlock_vault.take(total_already_available_amount, api)?;

        api.field_lock_release(handle)?;
        Ok(already_available_stake_unit_bucket)
    }

    /// Puts the given bucket into this validator's stake XRD vault, effectively increasing the
    /// value of all its stake units.
    /// Note: the concluded epoch's number and the validator's proposal statistics passed to this
    /// ethod are used only for creating an event (i.e. they are only informational and do not drive
    /// any logic at this point).
    pub fn apply_emission<Y>(
        xrd_bucket: Bucket,
        epoch: u64,
        proposals_made: u64,
        proposals_missed: u64,
        api: &mut Y,
    ) -> Result<(), RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let handle = api.actor_lock_field(
            OBJECT_HANDLE_SELF,
            ValidatorField::Validator.into(),
            LockFlags::MUTABLE,
        )?;
        let mut substate: ValidatorSubstate = api.field_lock_read_typed(handle)?;

        let stake_pool_added_xrd = xrd_bucket.amount(api)?;
        let total_stake_unit_supply =
            ResourceManager(substate.stake_unit_resource).total_supply(api)?;

        let mut stake_xrd_vault = Vault(substate.stake_xrd_vault_id);
        let starting_stake_pool_xrd = stake_xrd_vault.amount(api)?;
        stake_xrd_vault.put(xrd_bucket, api)?;

        let new_stake_xrd = starting_stake_pool_xrd + stake_pool_added_xrd;
        let new_index_key =
            Self::index_update(&substate, substate.is_registered, new_stake_xrd, api)?;
        substate.sorted_key = new_index_key;
        api.field_lock_write_typed(handle, &substate)?;
        api.field_lock_release(handle)?;

        Runtime::emit_event(
            api,
            ValidatorEmissionAppliedEvent {
                epoch,
                starting_stake_pool_xrd,
                stake_pool_added_xrd,
                total_stake_unit_supply,
                validator_fee_xrd: Decimal::zero(), // TODO(emissions): update after implementing validator fees
                proposals_made,
                proposals_missed,
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
                    EpochRegisteredValidatorByStakeEntry {
                        component_address: address,
                        validator: Validator { key, stake },
                    },
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
                    EpochRegisteredValidatorByStakeEntry {
                        component_address: address,
                        validator,
                    },
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
                    EpochRegisteredValidatorByStakeEntry {
                        component_address: address,
                        validator,
                    },
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
    const SECURIFY_AUTHORITY: Option<&'static str> = None;

    fn authority_rules() -> AuthorityRules {
        let mut authority_rules = AuthorityRules::new();
        authority_rules.set_metadata_authority(rule!(require_owner()), rule!(deny_all));
        authority_rules.set_royalty_authority(rule!(deny_all), rule!(deny_all));

        authority_rules
            .set_fixed_main_authority_rule(VALIDATOR_REGISTER_IDENT, rule!(require_owner()));
        authority_rules
            .set_fixed_main_authority_rule(VALIDATOR_UNREGISTER_IDENT, rule!(require_owner()));
        authority_rules.set_fixed_main_authority_rule(
            VALIDATOR_LOCK_OWNER_STAKE_UNITS_IDENT,
            rule!(require_owner()),
        );
        authority_rules.set_fixed_main_authority_rule(
            VALIDATOR_START_UNLOCK_OWNER_STAKE_UNITS_IDENT,
            rule!(require_owner()),
        );
        authority_rules.set_fixed_main_authority_rule(
            VALIDATOR_FINISH_UNLOCK_OWNER_STAKE_UNITS_IDENT,
            rule!(require_owner()),
        );
        authority_rules
            .set_fixed_main_authority_rule(VALIDATOR_UPDATE_KEY_IDENT, rule!(require_owner()));
        authority_rules.set_fixed_main_authority_rule(
            VALIDATOR_UPDATE_ACCEPT_DELEGATED_STAKE_IDENT,
            rule!(require_owner()),
        );
        authority_rules.set_main_authority_rule(
            VALIDATOR_STAKE_IDENT,
            rule!(require_owner()),
            rule!(require(package_of_direct_caller(EPOCH_MANAGER_PACKAGE))),
        );
        authority_rules.set_fixed_main_authority_rule(
            VALIDATOR_APPLY_EMISSION_IDENT,
            rule!(require(global_caller(EPOCH_MANAGER))),
        );
        authority_rules
    }
}

pub(crate) struct ValidatorCreator;

impl ValidatorCreator {
    fn create_stake_unit_resource<Y>(
        address: GlobalAddress,
        api: &mut Y,
    ) -> Result<ResourceAddress, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let mut stake_unit_resource_auth = BTreeMap::new();
        stake_unit_resource_auth.insert(
            Mint,
            (rule!(require(global_caller(address))), rule!(deny_all)),
        );
        stake_unit_resource_auth.insert(
            Burn,
            (rule!(require(global_caller(address))), rule!(deny_all)),
        );
        stake_unit_resource_auth.insert(Withdraw, (rule!(allow_all), rule!(deny_all)));
        stake_unit_resource_auth.insert(Deposit, (rule!(allow_all), rule!(deny_all)));

        let stake_unit_resman =
            ResourceManager::new_fungible(18, BTreeMap::new(), stake_unit_resource_auth, api)?;

        Ok(stake_unit_resman.0)
    }

    fn create_unstake_nft<Y>(
        address: GlobalAddress,
        api: &mut Y,
    ) -> Result<ResourceAddress, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let mut unstake_nft_auth = BTreeMap::new();

        unstake_nft_auth.insert(
            Mint,
            (rule!(require(global_caller(address))), rule!(deny_all)),
        );
        unstake_nft_auth.insert(
            Burn,
            (rule!(require(global_caller(address))), rule!(deny_all)),
        );
        unstake_nft_auth.insert(Withdraw, (rule!(allow_all), rule!(deny_all)));
        unstake_nft_auth.insert(Deposit, (rule!(allow_all), rule!(deny_all)));

        let unstake_resman = ResourceManager::new_non_fungible::<UnstakeData, Y, RuntimeError>(
            NonFungibleIdType::UUID,
            BTreeMap::new(),
            unstake_nft_auth,
            api,
        )?;

        Ok(unstake_resman.0)
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

        let stake_xrd_vault = Vault::create(RADIX_TOKEN, api)?;
        let pending_xrd_withdraw_vault = Vault::create(RADIX_TOKEN, api)?;
        let unstake_nft = Self::create_unstake_nft(address, api)?;
        let stake_unit_resource = Self::create_stake_unit_resource(address, api)?;
        let locked_owner_stake_unit_vault = Vault::create(stake_unit_resource, api)?;
        let pending_owner_stake_unit_unlock_vault = Vault::create(stake_unit_resource, api)?;
        let pending_owner_stake_unit_withdrawals = BTreeMap::new();
        // TODO(emissions): add `lock(), withdraw(), unlock()` owner-only methods for the 3 above

        let substate = ValidatorSubstate {
            sorted_key: None,
            key,
            is_registered,
            stake_unit_resource,
            unstake_nft,
            stake_xrd_vault_id: stake_xrd_vault.0,
            pending_xrd_withdraw_vault_id: pending_xrd_withdraw_vault.0,
            locked_owner_stake_unit_vault_id: locked_owner_stake_unit_vault.0,
            pending_owner_stake_unit_unlock_vault_id: pending_owner_stake_unit_unlock_vault.0,
            pending_owner_stake_unit_withdrawals,
            already_unlocked_owner_stake_unit_amount: Decimal::zero(),
        };

        let validator_id = api.new_simple_object(
            VALIDATOR_BLUEPRINT,
            vec![scrypto_encode(&substate).unwrap()],
        )?;

        let (access_rules, owner_token_bucket) = SecurifiedValidator::create_securified(api)?;
        let metadata = Metadata::create(api)?;
        let royalty = ComponentRoyalty::create(RoyaltyConfig::default(), api)?;

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
