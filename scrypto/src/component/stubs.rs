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
//  7. Faucet
// ================================================================================================

extern_blueprint_internal! {
    FAUCET_PACKAGE,
    Faucet,
    "Faucet",
    "OwnedFaucet",
    "GlobalFaucet",
    FaucetFunctions {
        fn new(address_reservation: GlobalAddressReservation, bucket: Bucket) -> Global<Faucet>;
    },
    {
        fn free(&mut self) -> Bucket;
        fn lock_fee(&mut self, amount: Decimal);
    }
}

extern_blueprint_internal! {
    CONSENSUS_MANAGER_PACKAGE,
    ConsensusManager,
    "ConsensusManager",
    "OwnedConsensusManager",
    "GlobalConsensusManager",
    ConsensusManagerFunctions {
        fn create(
            validator_owner_token_address: GlobalAddressReservation,
            component_address: GlobalAddressReservation,
            initial_epoch: u64,
            initial_config: ConsensusManagerConfig,
            initial_time_ms: i64,
            initial_current_leader: Option<u8>
        );
    },
    {
        fn get_current_epoch(&self) -> u64;
        fn start(&mut self);
        fn get_current_time(&self, precision: TimePrecision) -> i64;
        fn compare_current_time(
            &self,
            instant: i64,
            precision: TimePrecision,
            operator: TimeComparisonOperator
        ) -> bool;
        fn next_round(
            &mut self,
            round: u64,
            proposer_timestamp_ms: i64,
            leader_proposal_history: LeaderProposalHistory
        );
        fn create_validator(
            &mut self,
            key: Secp256k1PublicKey,
            fee_factor: Decimal,
            xrd_payment: Bucket
        ) -> (Global<Validator>, Bucket, Bucket);
    }
}

extern_blueprint_internal! {
    CONSENSUS_MANAGER_PACKAGE,
    Validator,
    "Validator",
    "OwnedValidator",
    "GlobalValidator",
    ValidatorFunctions {},
    {
        fn register(&mut self);
        fn unregister(&mut self);
        fn stake_as_owner(&mut self, stake: Bucket) -> Bucket;
        fn stake(&mut self, stake: Bucket) -> Bucket;
        fn unstake(&mut self, stake_unit_bucket: Bucket) -> Bucket;
        fn claim_xrd(&mut self, bucket: Bucket) -> Bucket;
        fn update_key(&mut self, key: Vec<u8>);
        fn update_fee(&mut self, new_fee_factor: Decimal);
        fn update_accept_delegated_stake(&mut self, accept_delegated_stake: bool);
        fn accepts_delegated_stake(&mut self) -> bool;
        fn total_stake_xrd_amount(&self) -> Decimal;
        fn total_stake_unit_supply(&self) -> Decimal;
        fn get_redemption_value(&self, amount_of_stake_units: Decimal) -> Decimal;
        fn signal_protocol_update_readiness(&mut self, vote: String);
        fn get_protocol_update_readiness(&mut self) -> Option<String>;
        fn lock_owner_stake_units(&mut self, stake_unit_bucket: Bucket);
        fn start_unlock_owner_stake_units(&mut self, requested_stake_unit_amount: Decimal);
        fn finish_unlock_owner_stake_units(&mut self) -> Bucket;
        fn apply_emission(
            &mut self,
            xrd_bucket: Bucket,
            epoch: u64,
            proposals_made: u64,
            proposals_missed: u64
        );
        fn apply_reward(&mut self, xrd_bucket: Bucket, epoch: u64);
    }
}

extern_blueprint_internal! {
    IDENTITY_PACKAGE,
    Identity,
    "Identity",
    "OwnedIdentity",
    "GlobalIdentity",
    IdentityFunctions {
        fn create_advanced(owner_role: OwnerRole) -> Global<Identity>;
        fn create() -> (Global<Identity>, Bucket);
    },
    {
        fn securify(&mut self) -> Bucket;
    }
}

extern_blueprint_internal! {
    ACCOUNT_PACKAGE,
    Account,
    "Account",
    "OwnedAccount",
    "GlobalAccount",
    AccountFunctions {
        fn create_advanced(
            owner_role: OwnerRole,
            address_reservation: Option<GlobalAddressReservation>
        ) -> Global<Account>;
        fn create() -> (Global<Account>, Bucket);
    },
    {
        fn securify(&mut self) -> Bucket;
        fn lock_fee(&mut self, amount: Decimal);
        fn lock_contingent_fee(&mut self, amount: Decimal);
        fn deposit(&mut self, bucket: Bucket);
        fn deposit_batch(&mut self, buckets: Vec<Bucket>);
        fn withdraw(&mut self, resource_address: ResourceAddress, amount: Decimal) -> Bucket;
        fn withdraw_non_fungibles(
            &mut self,
            resource_address: ResourceAddress,
            ids: Vec<NonFungibleLocalId>
        ) -> Bucket;
        fn burn(&mut self, resource_address: ResourceAddress, amount: Decimal);
        fn burn_non_fungibles(
            &mut self,
            resource_address: ResourceAddress,
            ids: Vec<NonFungibleLocalId>
        );
        fn lock_fee_and_withdraw(
            &mut self,
            amount_to_lock: Decimal,
            resource_address: ResourceAddress,
            amount: Decimal
        ) -> Bucket;
        fn lock_fee_and_withdraw_non_fungibles(
            &mut self,
            amount_to_lock: Decimal,
            resource_address: ResourceAddress,
            ids: Vec<NonFungibleLocalId>
        ) -> Bucket;
        fn create_proof_of_amount(&self, resource_address: ResourceAddress, amount: Decimal) -> Proof;
        fn create_proof_of_non_fungibles(
            &self,
            resource_address: ResourceAddress,
            ids: Vec<NonFungibleLocalId>
        ) -> Proof;
        fn set_default_deposit_rule(&self, default: DefaultDepositRule);
        fn set_resource_preference(
            &self,
            resource_address: ResourceAddress,
            resource_preference: ResourcePreference
        );
        fn remove_resource_preference(&self, resource_address: ResourceAddress);
        fn try_deposit_or_refund(
            &mut self,
            bucket: Bucket,
            authorized_depositor_badge: Option<ResourceOrNonFungible>
        ) -> Option<Bucket>;
        fn try_deposit_batch_or_refund(
            &mut self,
            buckets: Vec<Bucket>,
            authorized_depositor_badge: Option<ResourceOrNonFungible>
        ) -> Option<Vec<Bucket>>;
        fn try_deposit_or_abort(
            &mut self,
            bucket: Bucket,
            authorized_depositor_badge: Option<ResourceOrNonFungible>
        );
        fn try_deposit_batch_or_abort(
            &mut self,
            buckets: Vec<Bucket>,
            authorized_depositor_badge: Option<ResourceOrNonFungible>
        );
        fn add_authorized_depositor(&mut self, badge: ResourceOrNonFungible);
        fn remove_authorized_depositor(&mut self, badge: ResourceOrNonFungible);
    }
}

extern_blueprint_internal! {
    POOL_PACKAGE,
    MultiResourcePool,
    "MultiResourcePool",
    "OwnedMultiResourcePool",
    "GlobalMultiResourcePool",
    MultiResourcePoolFunctions {
        fn instantiate(
            owner_role: OwnerRole,
            pool_manager_rule: AccessRule,
            resource_addresses: Vec<ResourceAddress>,
            address_reservation: Option<GlobalAddressReservation>
        ) -> Global<MultiResourcePool>;
    },
    {
        fn contribute(&mut self, buckets: Vec<Bucket>) -> (Bucket, Vec<Bucket>);
        fn redeem(&mut self, bucket: Bucket) -> Vec<Bucket>;
        fn protected_deposit(&mut self, bucket: Bucket);
        fn protected_withdraw(
            &mut self,
            resource_address: ResourceAddress,
            amount: Decimal,
            withdraw_strategy: WithdrawStrategy
        ) -> Bucket;
        fn get_redemption_value(
            &self,
            amount_of_pool_units: Decimal
        ) -> IndexMap<ResourceAddress, Decimal>;
        fn get_vault_amounts(&self) -> IndexMap<ResourceAddress, Decimal>;
    }
}

extern_blueprint_internal! {
    POOL_PACKAGE,
    OneResourcePool,
    "OneResourcePool",
    "OwnedOneResourcePool",
    "GlobalOneResourcePool",
    OneResourcePoolFunctions {
        fn instantiate(
            owner_role: OwnerRole,
            pool_manager_rule: AccessRule,
            resource_address: ResourceAddress,
            address_reservation: Option<GlobalAddressReservation>
        ) -> Global<OneResourcePool>;
    },
    {
        fn contribute(&mut self, bucket: Bucket) -> Bucket;
        fn redeem(&mut self, bucket: Bucket) -> Bucket;
        fn protected_deposit(&mut self, bucket: Bucket);
        fn protected_withdraw(
            &mut self,
            amount: Decimal,
            withdraw_strategy: WithdrawStrategy
        ) -> Bucket;
        fn get_redemption_value(&self, amount_of_pool_units: Decimal) -> Decimal;
        fn get_vault_amount(&self) -> Decimal;
    }
}

extern_blueprint_internal! {
    POOL_PACKAGE,
    TwoResourcePool,
    "TwoResourcePool",
    "OwnedTwoResourcePool",
    "GlobalTwoResourcePool",
    TwoResourcePoolFunctions {
        fn instantiate(
            owner_role: OwnerRole,
            pool_manager_rule: AccessRule,
            resource_addresses: (ResourceAddress, ResourceAddress),
            address_reservation: Option<GlobalAddressReservation>
        ) -> Global<TwoResourcePool>;
    },
    {
        fn contribute(&mut self, buckets: (Bucket, Bucket)) -> (Bucket, Option<Bucket>);
        fn redeem(&mut self, bucket: Bucket) -> (Bucket, Bucket);
        fn protected_deposit(&mut self, bucket: Bucket);
        fn protected_withdraw(
            &mut self,
            resource_address: ResourceAddress,
            amount: Decimal,
            withdraw_strategy: WithdrawStrategy
        ) -> Bucket;
        fn get_redemption_value(
            &self,
            amount_of_pool_units: Decimal
        ) -> IndexMap<ResourceAddress, Decimal>;
        fn get_vault_amounts(&self) -> IndexMap<ResourceAddress, Decimal>;
    }
}

extern_blueprint_internal! {
    ACCESS_CONTROLLER_PACKAGE,
    AccessController,
    "AccessController",
    "OwnedAccessController",
    "GlobalAccessController",
    AccessControllerFunctions {
        fn create(
            controlled_asset: Bucket,
            rule_set: RuleSet,
            timed_recovery_delay_in_minutes: Option<u32>,
            address_reservation: Option<GlobalAddressReservation>
        ) -> Global<AccessController>;
    },
    {
        fn create_proof(&mut self) -> Proof;
        fn initiate_recovery_as_primary(
            &mut self,
            rule_set: RuleSet,
            timed_recovery_delay_in_minutes: Option<u32>
        );
        fn initiate_recovery_as_recovery(
            &mut self,
            rule_set: RuleSet,
            timed_recovery_delay_in_minutes: Option<u32>
        );
        fn quick_confirm_primary_role_recovery_proposal(
            &mut self,
            rule_set: RuleSet,
            timed_recovery_delay_in_minutes: Option<u32>
        );
        fn quick_confirm_recovery_role_recovery_proposal(
            &mut self,
            rule_set: RuleSet,
            timed_recovery_delay_in_minutes: Option<u32>
        );
        fn timed_confirm_recovery(
            &mut self,
            rule_set: RuleSet,
            timed_recovery_delay_in_minutes: Option<u32>
        );
        fn cancel_primary_role_recovery_proposal(&mut self);
        fn cancel_recovery_role_recovery_proposal(&mut self);
        fn lock_primary_role(&mut self);
        fn unlock_primary_role(&mut self);
        fn stop_timed_recovery(
            &mut self,
            rule_set: RuleSet,
            timed_recovery_delay_in_minutes: Option<u32>
        );
        fn initiate_badge_withdraw_attempt_as_primary(&mut self);
        fn initiate_badge_withdraw_attempt_as_recovery(&mut self);
        fn quick_confirm_primary_role_badge_withdraw_attempt(&mut self) -> Bucket;
        fn quick_confirm_recovery_role_badge_withdraw_attempt(&mut self) -> Bucket;
        fn cancel_primary_role_badge_withdraw_attempt(&mut self);
        fn cancel_recovery_role_badge_withdraw_attempt(&mut self);
        fn mint_recovery_badges(&mut self, non_fungible_local_ids: Vec<NonFungibleLocalId>) -> Bucket;
    }
}
