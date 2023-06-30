use radix_engine_interface::blueprints::access_controller::*;
use radix_engine_interface::blueprints::account::*;
use radix_engine_interface::blueprints::consensus_manager::*;

use crate::prelude::*;

// ================================================================================================
// Note: Please update this file if you change anything about the interface of the following
// blueprints:
//  1. Pools
//  2. Accounts
//  3. Identity
//  4. Access Controller
//  5. Consensus Manager
//  6. Validator
// ================================================================================================

extern_blueprint_internal! {
    POOL_PACKAGE,
    MultiResourcePool,
    "MultiResourcePool",
    "OwnedMultiResourcePool",
    "GlobalMultiResourcePool",
    MultiResourcePoolFunctions
    {
        fn instantiate(resource_addresses: Vec<ResourceAddress>, pool_manager_rule: AccessRule) -> Global<MultiResourcePool>;
    },
    {
        fn contribute(&mut self, buckets: Vec<Bucket>) -> (Bucket, Vec<Bucket>);
        fn get_redemption_value(&self, amount_of_pool_units: Decimal) -> BTreeMap<ResourceAddress, Decimal>;
        fn get_vault_amounts(&self) -> BTreeMap<ResourceAddress, Decimal>;
        fn protected_deposit(&mut self, bucket: Bucket);

        /// # Warning
        ///
        /// This method does not check the divisibility of the resource that you are attempting to
        /// withdraw; thus, this method can panic at runtime if the divisibility of the resource is
        /// not compatible with the amount you're attempting to withdraw. As an example, attempting
        /// to withdraw `1.1111` of a resource with a divisibility of 2 would lead this method to
        /// panic at runtime.
        ///
        /// It is the responsibility of the applications using the pool blueprint to ensure that
        /// this function is called with a [`Decimal`] of an appropriate number of decimal places
        /// for the given resource.
        fn protected_withdraw(&mut self, resource_address: ResourceAddress, amount: Decimal) -> Bucket;
        fn redeem(&mut self, bucket: Bucket) -> Vec<Bucket>;
    }
}

extern_blueprint_internal! {
    POOL_PACKAGE,
    OneResourcePool,
    "OneResourcePool",
    "OwnedOneResourcePool",
    "GlobalOneResourcePool",
    OneResourcePoolFunctions
    {
        fn instantiate(resource_address: ResourceAddress, pool_manager_rule: AccessRule) -> Global<OneResourcePool>;
    },
    {
        fn contribute(&mut self, bucket: Bucket) -> Bucket;
        fn get_redemption_value(&self, amount_of_pool_units: Decimal) -> Decimal;
        fn get_vault_amount(&self) -> Decimal;
        fn protected_deposit(&mut self, bucket: Bucket);

        /// # Warning
        ///
        /// This method does not check the divisibility of the resource that you are attempting to
        /// withdraw; thus, this method can panic at runtime if the divisibility of the resource is
        /// not compatible with the amount you're attempting to withdraw. As an example, attempting
        /// to withdraw `1.1111` of a resource with a divisibility of 2 would lead this method to
        /// panic at runtime.
        ///
        /// It is the responsibility of the applications using the pool blueprint to ensure that
        /// this function is called with a [`Decimal`] of an appropriate number of decimal places
        /// for the given resource.
        fn protected_withdraw(&mut self, amount: Decimal) -> Bucket;
        fn redeem(&mut self, bucket: Bucket) -> Bucket;
    }
}

extern_blueprint_internal! {
    POOL_PACKAGE,
    TwoResourcePool,
    "TwoResourcePool",
    "OwnedTwoResourcePool",
    "GlobalTwoResourcePool",
    TwoResourcePoolFunctions
    {
        fn instantiate(resource_addresses: (ResourceAddress, ResourceAddress), pool_manager_rule: AccessRule) -> Global<TwoResourcePool>;
    },
    {
        fn contribute(&mut self, buckets: (Bucket, Bucket)) -> (Bucket, Option<Bucket>);
        fn get_redemption_value(&self, amount_of_pool_units: Decimal) -> BTreeMap<ResourceAddress, Decimal>;
        fn get_vault_amounts(&self) -> BTreeMap<ResourceAddress, Decimal>;
        fn protected_deposit(&mut self, bucket: Bucket);

        /// # Warning
        ///
        /// This method does not check the divisibility of the resource that you are attempting to
        /// withdraw; thus, this method can panic at runtime if the divisibility of the resource is
        /// not compatible with the amount you're attempting to withdraw. As an example, attempting
        /// to withdraw `1.1111` of a resource with a divisibility of 2 would lead this method to
        /// panic at runtime.
        ///
        /// It is the responsibility of the applications using the pool blueprint to ensure that
        /// this function is called with a [`Decimal`] of an appropriate number of decimal places
        /// for the given resource.
        fn protected_withdraw(&mut self, resource_address: ResourceAddress, amount: Decimal) -> Bucket;
        fn redeem(&mut self, bucket: Bucket) -> (Bucket, Bucket);
    }
}

extern_blueprint_internal! {
    ACCOUNT_PACKAGE,
    Account,
    "Account",
    "OwnedAccount",
    "GlobalAccount",
    AccountFunctions
    {
        fn create() -> (Global<Account>, Bucket);
        fn create_advanced(owner_role: OwnerRole) -> Global<Account>;
    },
    {
        fn burn(&mut self, resource_address: ResourceAddress, amount: Decimal);
        fn burn_non_fungibles(&mut self, resource_address: ResourceAddress, ids: Vec<NonFungibleLocalId>);
        fn change_account_default_deposit_rule(&self, default_deposit_rule: AccountDefaultDepositRule);
        fn configure_resource_deposit_rule(&self, resource_address: ResourceAddress, resource_deposit_configuration: ResourceDepositRule);
        fn create_proof(&self, resource_address: ResourceAddress) -> Proof;
        fn create_proof_of_amount(&self, resource_address: ResourceAddress, amount: Decimal) -> Proof;
        fn create_proof_of_non_fungibles(&self, resource_address: ResourceAddress, ids: Vec<NonFungibleLocalId>) -> Proof;
        fn deposit(&mut self, bucket: Bucket);
        fn deposit_batch(&mut self, buckets: Vec<Bucket>);
        fn lock_contingent_fee(&mut self, amount: Decimal);
        fn lock_fee(&mut self, amount: Decimal);
        fn lock_fee_and_withdraw(&mut self, amount_to_lock: Decimal, resource_address: ResourceAddress, amount: Decimal) -> Bucket;
        fn lock_fee_and_withdraw_non_fungibles(&mut self, amount_to_lock: Decimal, resource_address: ResourceAddress, ids: Vec<NonFungibleLocalId>) -> Bucket;
        fn securify(&mut self) -> Bucket;
        fn try_deposit_batch_or_abort(&mut self, buckets: Vec<Bucket>);
        fn try_deposit_batch_or_refund(&mut self, buckets: Vec<Bucket>) -> Vec<Bucket>;
        fn try_deposit_or_abort(&mut self, bucket: Bucket);
        fn try_deposit_or_refund(&mut self, bucket: Bucket) -> Option<Bucket>;
        fn withdraw(&mut self, resource_address: ResourceAddress, amount: Decimal) -> Bucket;
        fn withdraw_non_fungibles(&mut self, resource_address: ResourceAddress, ids: Vec<NonFungibleLocalId>) -> Bucket;
    }
}

extern_blueprint_internal! {
    IDENTITY_PACKAGE,
    Identity,
    "Identity",
    "OwnedIdentity",
    "GlobalIdentity",
    IdentityFunctions
    {
        fn create() -> (Global<Identity>, Bucket);
        fn create_advanced(owner_rule: OwnerRole) -> Global<Identity>;
    },
    {
        fn securify(&mut self) -> Bucket;
    }
}

extern_blueprint_internal! {
    ACCESS_CONTROLLER_PACKAGE,
    AccessController,
    "AccessController",
    "OwnedAccessController",
    "GlobalAccessController",
    AccessControllerFunctions
    {
        fn create_global(controlled_asset: Bucket, rule_set: RuleSet, timed_recovery_delay_in_minutes: Option<u32>) -> Global<AccessController>;
    },
    {
        fn cancel_primary_role_badge_withdraw_attempt(&mut self);
        fn cancel_primary_role_recovery_proposal(&mut self);
        fn cancel_recovery_role_badge_withdraw_attempt(&mut self);
        fn cancel_recovery_role_recovery_proposal(&mut self);
        fn create_proof(&mut self) -> Proof;
        fn initiate_badge_withdraw_attempt_as_primary(&mut self);
        fn initiate_badge_withdraw_attempt_as_recovery(&mut self);
        fn initiate_recovery_as_primary(&mut self, rule_set: RuleSet, timed_recovery_delay_in_minutes: Option<u32>);
        fn initiate_recovery_as_recovery(&mut self, rule_set: RuleSet, timed_recovery_delay_in_minutes: Option<u32>);
        fn lock_primary_role(&mut self);
        fn mint_recovery_badges(&mut self, non_fungible_local_ids: Vec<NonFungibleLocalId>) -> Bucket;
        fn quick_confirm_primary_role_badge_withdraw_attempt(&mut self) -> Bucket;
        fn quick_confirm_primary_role_recovery_proposal(&mut self, rule_set: RuleSet, timed_recovery_delay_in_minutes: Option<u32>);
        fn quick_confirm_recovery_role_badge_withdraw_attempt(&mut self) -> Bucket;
        fn quick_confirm_recovery_role_recovery_proposal(&mut self, rule_set: RuleSet, timed_recovery_delay_in_minutes: Option<u32>);
        fn stop_timed_recovery(&mut self, rule_set: RuleSet, timed_recovery_delay_in_minutes: Option<u32>);
        fn timed_confirm_recovery(&mut self, rule_set: RuleSet, timed_recovery_delay_in_minutes: Option<u32>);
        fn unlock_primary_role(&mut self);
    }
}

extern_blueprint_internal! {
    CONSENSUS_MANAGER_PACKAGE,
    ConsensusManager,
    "ConsensusManager",
    "OwnedConsensusManager",
    "GlobalConsensusManager",
    ConsensusManagerFunctions
    {
        fn create(validator_owner_token_address: GlobalAddressReservation, component_address: GlobalAddressReservation, initial_epoch: Epoch, initial_config: ConsensusManagerConfig, initial_time_ms: i64, initial_current_leader: Option<ValidatorIndex>);
    },
    {
        fn compare_current_time(&self, instant: Instant, precision: TimePrecision, operator: TimeComparisonOperator) -> bool;
        fn create_validator(&mut self, key: Secp256k1PublicKey, fee_factor: Decimal) -> (Global<Validator>, Bucket);
        fn get_current_epoch(&self) -> Epoch;
        fn get_current_time(&self, precision: TimePrecision) -> Instant;
        fn next_round(&mut self, round: Round, proposer_timestamp_ms: i64, leader_proposal_history: LeaderProposalHistory);
        fn start(&mut self);
    }
}

extern_blueprint_internal! {
    CONSENSUS_MANAGER_PACKAGE,
    Validator,
    "Validator",
    "OwnedValidator",
    "GlobalValidator",
    ValidatorFunctions
    {},
    {
        fn apply_emission(&mut self, xrd_bucket: Bucket, epoch: Epoch, proposals_made: u64, proposals_missed: u64);
        fn apply_reward(&mut self, xrd_bucket: Bucket, epoch: Epoch);
        fn claim_xrd(&mut self, bucket: Bucket) -> Bucket;
        fn finish_unlock_owner_stake_units(&mut self) -> Bucket;
        fn lock_owner_stake_units(&mut self, stake_unit_bucket: Bucket);
        fn register(&mut self);
        fn stake(&mut self, stake: Bucket) -> Bucket;
        fn stake_as_owner(&mut self, stake: Bucket) -> Bucket;
        fn start_unlock_owner_stake_units(&mut self, requested_stake_unit_amount: Decimal);
        fn unregister(&mut self);
        fn unstake(&mut self, stake_unit_bucket: Bucket) -> Bucket;
        fn update_accept_delegated_stake(&mut self, accept_delegated_stake: bool);
        fn update_fee(&mut self, new_fee_factor: Decimal);
        fn update_key(&mut self, key: Secp256k1PublicKey);
    }
}
