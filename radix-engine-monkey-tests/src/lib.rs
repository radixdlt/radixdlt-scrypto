pub mod consensus_manager;
pub mod multi_pool;
pub mod one_pool;
pub mod resource;
pub mod two_pool;
pub mod validator;

use crate::consensus_manager::ConsensusManagerFuzzAction;
use crate::multi_pool::MultiPoolFuzzAction;
use crate::one_pool::OnePoolFuzzAction;
use crate::resource::{
    FungibleResourceFuzzGetBucketAction, NonFungibleResourceFuzzGetBucketAction,
    ResourceFuzzRandomAction, ResourceFuzzTransformBucketAction, ResourceFuzzUseBucketAction,
    ResourceTestInvoke, BLUEPRINT_NAME, CUSTOM_PACKAGE_CODE_ID,
};
use crate::two_pool::TwoPoolFuzzAction;
use crate::validator::ValidatorFuzzAction;
use core::ops::AddAssign;
use radix_common::prelude::*;
use radix_engine::blueprints::consensus_manager::EpochChangeEvent;
use radix_engine::blueprints::pool::v1::constants::*;
use radix_engine::errors::{NativeRuntimeError, RuntimeError, VmError};
use radix_engine::transaction::{TransactionOutcome, TransactionResult};
use radix_engine::updates::BabylonSettings;
use radix_engine::vm::OverridePackageCode;
use radix_engine_interface::blueprints::package::PackageDefinition;
use radix_engine_interface::blueprints::pool::{
    MultiResourcePoolInstantiateManifestInput, TwoResourcePoolInstantiateManifestInput,
    MULTI_RESOURCE_POOL_INSTANTIATE_IDENT, TWO_RESOURCE_POOL_INSTANTIATE_IDENT,
};
use radix_engine_interface::object_modules::ModuleConfig;
use radix_engine_interface::prelude::*;
use radix_substate_store_impls::memory_db::InMemorySubstateDatabase;
use radix_transactions::builder::ManifestBuilder;
use rand::distributions::uniform::{SampleRange, SampleUniform};
use rand::Rng;
use rand_chacha::rand_core::{RngCore, SeedableRng};
use rand_chacha::ChaCha8Rng;
use rayon::iter::IntoParallelIterator;
use rayon::iter::ParallelIterator;
use scrypto_test::prelude::{LedgerSimulator, LedgerSimulatorBuilder};

pub struct SystemTestFuzzer {
    rng: ChaCha8Rng,
    resources: Vec<ResourceAddress>,
    non_fungibles: Vec<ResourceAddress>,
    fungibles: Vec<ResourceAddress>,
}

impl SystemTestFuzzer {
    pub fn new(seed: u64) -> Self {
        let rng = ChaCha8Rng::seed_from_u64(seed);
        Self {
            rng,
            resources: Vec::new(),
            non_fungibles: Vec::new(),
            fungibles: Vec::new(),
        }
    }

    pub fn next_amount(&mut self) -> Decimal {
        let next_amount_type = self.rng.gen_range(0u32..=4u32);
        match next_amount_type {
            0 => match self.rng.gen_range(0u32..=4u32) {
                0 => Decimal::ZERO,
                1 => Decimal::ONE,
                2 => Decimal::MAX,
                3 => Decimal::MIN,
                _ => Decimal::ONE_ATTO,
            },
            1 => {
                let amount = self.rng.gen_range(0u64..u64::MAX);
                Decimal::from(amount)
            }
            2 => {
                let amount = self.rng.gen_range(1000u64..10000u64);
                Decimal::from(amount)
            }
            3 => {
                let mut bytes = [0u8; 24];
                let (start, _end) = bytes.split_at_mut(8);
                self.rng.fill_bytes(start);
                Decimal::from_attos(I192::from_le_bytes(&bytes))
            }
            _ => {
                let mut bytes = [0u8; 24];
                self.rng.fill_bytes(&mut bytes);
                Decimal::from_attos(I192::from_le_bytes(&bytes))
            }
        }
    }

    pub fn next_fee(&mut self) -> Decimal {
        let next_amount = self.rng.gen_range(0u32..=7u32);
        match next_amount {
            0u32 => self.next_amount(),
            _ => Decimal::from(500),
        }
    }

    pub fn next_usize(&mut self, count: usize) -> usize {
        self.rng.gen_range(0usize..count)
    }

    pub fn next_u8(&mut self, count: u8) -> u8 {
        self.rng.gen_range(0u8..count)
    }

    pub fn next_valid_divisibility(&mut self) -> u8 {
        self.rng.gen_range(0u8..=18u8)
    }

    pub fn next_u32(&mut self, count: u32) -> u32 {
        self.rng.gen_range(0u32..count)
    }

    pub fn next_integer_non_fungible_id(&mut self) -> NonFungibleLocalId {
        NonFungibleLocalId::integer(self.rng.gen_range(0u64..4u64))
    }

    pub fn next_non_fungible_id_set(&mut self) -> BTreeSet<NonFungibleLocalId> {
        match self.rng.gen_range(0u64..4u64) {
            0u64 => {
                btreeset!(NonFungibleLocalId::integer(
                    self.rng.gen_range(0u64..100u64)
                ))
            }
            _ => (0u64..self.rng.gen_range(0u64..4u64))
                .into_iter()
                .map(|_| self.next_integer_non_fungible_id())
                .collect(),
        }
    }

    pub fn next<T, R>(&mut self, range: R) -> T
    where
        T: SampleUniform,
        R: SampleRange<T>,
    {
        self.rng.gen_range(range)
    }

    pub fn next_withdraw_strategy(&mut self) -> WithdrawStrategy {
        match self.next(0u32..=7u32) {
            0u32 => WithdrawStrategy::Exact,
            1u32 => WithdrawStrategy::Rounded(RoundingMode::AwayFromZero),
            2u32 => WithdrawStrategy::Rounded(RoundingMode::ToNearestMidpointAwayFromZero),
            3u32 => WithdrawStrategy::Rounded(RoundingMode::ToNearestMidpointToEven),
            4u32 => WithdrawStrategy::Rounded(RoundingMode::ToNearestMidpointTowardZero),
            5u32 => WithdrawStrategy::Rounded(RoundingMode::ToNegativeInfinity),
            6u32 => WithdrawStrategy::Rounded(RoundingMode::ToPositiveInfinity),
            _ => WithdrawStrategy::Rounded(RoundingMode::ToZero),
        }
    }

    pub fn add_resource(&mut self, resource_address: ResourceAddress) {
        self.resources.push(resource_address);
        if resource_address.is_fungible() {
            self.fungibles.push(resource_address);
        } else {
            self.non_fungibles.push(resource_address);
        }
    }

    pub fn next_resource(&mut self) -> ResourceAddress {
        let index = self.rng.gen_range(0usize..self.resources.len());
        self.resources[index]
    }

    pub fn next_non_fungible(&mut self) -> ResourceAddress {
        let index = self.rng.gen_range(0usize..self.non_fungibles.len());
        self.non_fungibles[index]
    }
}

#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq)]
pub enum FuzzAction {
    ConsensusManager(ConsensusManagerFuzzAction),
    Validator(ValidatorFuzzAction),
    OneResourcePool(OnePoolFuzzAction),
    TwoResourcePool(TwoPoolFuzzAction),
    MultiResourcePool(MultiPoolFuzzAction),
    FungibleGetBucket(FungibleResourceFuzzGetBucketAction),
    FungibleBucketTransform(ResourceFuzzTransformBucketAction),
    FungibleUseBucket(ResourceFuzzUseBucketAction),
    NonFungibleGetBucket(NonFungibleResourceFuzzGetBucketAction),
    NonFungibleUseBucket(ResourceFuzzUseBucketAction),
    Resource(ResourceFuzzRandomAction),
}

impl FuzzAction {
    pub fn add_to_manifest(
        &self,
        uuid: u64,
        builder: ManifestBuilder,
        fuzzer: &mut SystemTestFuzzer,
        validators: &Vec<ValidatorMeta>,
        one_resource_pool: &OnePoolMeta,
        two_resource_pool: &TwoPoolMeta,
        multi_resource_pool: &MultiPoolMeta,
        fungible_component: &ResourceComponentMeta,
        non_fungible_component: &ResourceComponentMeta,
        account_address: ComponentAddress,
    ) -> (ManifestBuilder, bool) {
        match self {
            FuzzAction::ConsensusManager(action) => {
                action.add_to_manifest(uuid, builder, fuzzer, validators, account_address)
            }
            FuzzAction::Validator(action) => {
                action.add_to_manifest(uuid, builder, fuzzer, validators, account_address)
            }
            FuzzAction::OneResourcePool(action) => {
                action.add_to_manifest(builder, fuzzer, account_address, one_resource_pool)
            }
            FuzzAction::TwoResourcePool(action) => {
                action.add_to_manifest(builder, fuzzer, account_address, two_resource_pool)
            }
            FuzzAction::MultiResourcePool(action) => {
                action.add_to_manifest(builder, fuzzer, account_address, multi_resource_pool)
            }
            FuzzAction::FungibleGetBucket(action) => {
                action.add_to_manifest(builder, fuzzer, fungible_component)
            }
            FuzzAction::FungibleBucketTransform(action) => {
                action.add_to_manifest(builder, fuzzer, fungible_component)
            }
            FuzzAction::FungibleUseBucket(action) => {
                action.add_to_manifest(builder, fuzzer, fungible_component)
            }
            FuzzAction::NonFungibleGetBucket(action) => {
                action.add_to_manifest(builder, fuzzer, non_fungible_component)
            }
            FuzzAction::NonFungibleUseBucket(action) => {
                action.add_to_manifest(builder, fuzzer, non_fungible_component)
            }
            FuzzAction::Resource(action) => action.add_to_manifest(
                builder,
                fuzzer,
                account_address,
                fungible_component,
                non_fungible_component,
            ),
        }
    }
}

#[repr(u8)]
#[derive(Copy, Clone, Debug, FromRepr, Ord, PartialOrd, Eq, PartialEq)]
pub enum FuzzTxnResult {
    TrivialSuccess,
    Success,
    TrivialFailure,
    Failure,
    Reject,
}

impl FuzzTxnResult {
    pub fn from_outcome(outcome: &TransactionOutcome, trivial: bool) -> Self {
        match (outcome, trivial) {
            (TransactionOutcome::Success(..), true) => FuzzTxnResult::TrivialSuccess,
            (TransactionOutcome::Success(..), false) => FuzzTxnResult::Success,
            (TransactionOutcome::Failure(..), true) => FuzzTxnResult::TrivialFailure,
            (TransactionOutcome::Failure(..), false) => FuzzTxnResult::Failure,
        }
    }
}

pub type FuzzTxnIntent = Vec<FuzzAction>;

pub trait TxnFuzzer {
    fn next_txn_intent(fuzzer: &mut SystemTestFuzzer) -> FuzzTxnIntent;
}

#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq)]
pub struct ValidatorMeta {
    pub account_address: ComponentAddress,
    pub validator_address: ComponentAddress,
    pub stake_unit_resource: ResourceAddress,
    pub claim_resource: ResourceAddress,
}

#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq)]
pub struct OnePoolMeta {
    pub pool_address: ComponentAddress,
    pub pool_unit_resource_address: ResourceAddress,
    pub resource_address: ResourceAddress,
}

#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq)]
pub struct TwoPoolMeta {
    pub pool_address: ComponentAddress,
    pub pool_unit_resource_address: ResourceAddress,
    pub resource_address1: ResourceAddress,
    pub resource_address2: ResourceAddress,
}

#[derive(Clone, Debug, Ord, PartialOrd, Eq, PartialEq)]
pub struct MultiPoolMeta {
    pub pool_address: ComponentAddress,
    pub pool_unit_resource_address: ResourceAddress,
    pub pool_resources: Vec<ResourceAddress>,
}

#[derive(Clone, Debug, Ord, PartialOrd, Eq, PartialEq)]
pub struct ResourceComponentMeta {
    pub component_address: ComponentAddress,
    pub resource_address: ResourceAddress,
    pub vault_address: InternalAddress,
}

pub struct FuzzTest<T: TxnFuzzer> {
    ledger: LedgerSimulator<OverridePackageCode<ResourceTestInvoke>, InMemorySubstateDatabase>,
    fuzzer: SystemTestFuzzer,
    validators: Vec<ValidatorMeta>,
    one_resource_pool: OnePoolMeta,
    two_resource_pool: TwoPoolMeta,
    multi_resource_pool: MultiPoolMeta,
    fungible_meta: ResourceComponentMeta,
    non_fungible_meta: ResourceComponentMeta,
    account_address: ComponentAddress,
    account_public_key: PublicKey,
    cur_round: Round,
    txn_fuzzer: PhantomData<T>,
}

impl<T: TxnFuzzer> FuzzTest<T> {
    fn new(seed: u64) -> Self {
        let mut fuzzer = SystemTestFuzzer::new(seed);
        let initial_epoch = Epoch::of(5);
        let pub_key = Secp256k1PrivateKey::from_u64(1u64).unwrap().public_key();
        let genesis = BabylonSettings::single_validator_and_staker(
            pub_key,
            Decimal::one(),
            Decimal::from(24_000_000_000u64),
            ComponentAddress::preallocated_account_from_public_key(&pub_key),
            initial_epoch,
            ConsensusManagerConfig::test_default(),
        );

        let (mut ledger, epoch_change) = LedgerSimulatorBuilder::new()
            .with_custom_protocol(|builder| {
                builder
                    .configure_babylon(|_| genesis)
                    .from_bootstrap_to_latest()
            })
            .with_custom_extension(OverridePackageCode::new(
                CUSTOM_PACKAGE_CODE_ID,
                ResourceTestInvoke,
            ))
            .build_and_get_post_genesis_epoch_change();
        let validator_set = epoch_change.unwrap().validator_set;
        let public_key = Secp256k1PrivateKey::from_u64(1u64).unwrap().public_key();
        let account = ComponentAddress::preallocated_account_from_public_key(&public_key);
        let virtual_signature_badge = NonFungibleGlobalId::from_public_key(&public_key);

        fuzzer.add_resource(XRD);

        let validator_meta = {
            let validator_address = *validator_set
                .validators_by_stake_desc
                .iter()
                .next()
                .unwrap()
                .0;
            let validator_substate = ledger.get_validator_info(validator_address);
            let stake_unit_resource = validator_substate.stake_unit_resource;
            let claim_resource = validator_substate.claim_nft;

            ValidatorMeta {
                validator_address,
                stake_unit_resource,
                claim_resource,
                account_address: account,
            }
        };

        let one_resource_pool = {
            let one_pool_resource = ledger.create_freely_mintable_and_burnable_fungible_resource(
                OwnerRole::None,
                None,
                fuzzer.next(0u8..=18u8),
                account,
            );

            let (pool_address, pool_unit_resource_address) = ledger.create_one_resource_pool(
                one_pool_resource,
                rule!(require(virtual_signature_badge.clone())),
            );

            OnePoolMeta {
                pool_address,
                pool_unit_resource_address,
                resource_address: one_pool_resource,
            }
        };

        let two_resource_pool = {
            let pool_resource1 = ledger.create_freely_mintable_and_burnable_fungible_resource(
                OwnerRole::None,
                None,
                fuzzer.next_valid_divisibility(),
                account,
            );
            let pool_resource2 = ledger.create_freely_mintable_and_burnable_fungible_resource(
                OwnerRole::None,
                None,
                fuzzer.next_valid_divisibility(),
                account,
            );

            let (pool_component, pool_unit_resource) = {
                let manifest = ManifestBuilder::new()
                    .lock_fee_from_faucet()
                    .call_function(
                        POOL_PACKAGE,
                        TWO_RESOURCE_POOL_BLUEPRINT_IDENT,
                        TWO_RESOURCE_POOL_INSTANTIATE_IDENT,
                        TwoResourcePoolInstantiateManifestInput {
                            resource_addresses: (pool_resource1.into(), pool_resource2.into()),
                            pool_manager_rule: rule!(require(virtual_signature_badge.clone())),
                            owner_role: OwnerRole::None,
                            address_reservation: None,
                        },
                    )
                    .build();
                let receipt = ledger.execute_manifest(manifest, vec![]);
                let commit_result = receipt.expect_commit_success();

                (
                    commit_result.new_component_addresses()[0],
                    commit_result.new_resource_addresses()[0],
                )
            };
            TwoPoolMeta {
                pool_address: pool_component,
                pool_unit_resource_address: pool_unit_resource,
                resource_address1: pool_resource1,
                resource_address2: pool_resource2,
            }
        };

        let multi_resource_pool = {
            let divisibility = vec![
                fuzzer.next_valid_divisibility(),
                fuzzer.next_valid_divisibility(),
                fuzzer.next_valid_divisibility(),
            ];

            let pool_resources: Vec<ResourceAddress> = divisibility
                .into_iter()
                .map(|divisibility| {
                    ledger.create_freely_mintable_and_burnable_fungible_resource(
                        OwnerRole::None,
                        None,
                        divisibility,
                        account,
                    )
                })
                .collect();

            let (pool_component, pool_unit_resource) = {
                let manifest = ManifestBuilder::new()
                    .lock_fee_from_faucet()
                    .call_function(
                        POOL_PACKAGE,
                        MULTI_RESOURCE_POOL_BLUEPRINT_IDENT,
                        MULTI_RESOURCE_POOL_INSTANTIATE_IDENT,
                        MultiResourcePoolInstantiateManifestInput {
                            resource_addresses: pool_resources
                                .clone()
                                .into_iter()
                                .map(Into::into)
                                .collect(),
                            pool_manager_rule: rule!(require(virtual_signature_badge.clone())),
                            owner_role: OwnerRole::None,
                            address_reservation: None,
                        },
                    )
                    .build();
                let receipt = ledger.execute_manifest(manifest, vec![]);
                let commit_result = receipt.expect_commit_success();

                (
                    commit_result.new_component_addresses()[0],
                    commit_result.new_resource_addresses()[0],
                )
            };

            MultiPoolMeta {
                pool_address: pool_component,
                pool_unit_resource_address: pool_unit_resource,
                pool_resources,
            }
        };

        let package_address = ledger.publish_native_package(
            CUSTOM_PACKAGE_CODE_ID,
            PackageDefinition::new_with_field_test_definition(
                BLUEPRINT_NAME,
                vec![
                    ("call_vault", "call_vault", true),
                    ("new", "new", false),
                    ("new_with_bucket", "new_with_bucket", false),
                    ("combine_buckets", "combine_buckets", true),
                ],
            ),
        );

        let fungible_vault_component = {
            let receipt = ledger.execute_manifest(
                ManifestBuilder::new()
                    .lock_fee_from_faucet()
                    .create_fungible_resource(
                        OwnerRole::None,
                        true,
                        18u8,
                        FungibleResourceRoles {
                            mint_roles: mint_roles! {
                                minter => rule!(allow_all);
                                minter_updater => rule!(deny_all);
                            },
                            burn_roles: burn_roles! {
                                burner => rule!(allow_all);
                                burner_updater => rule!(deny_all);
                            },
                            recall_roles: recall_roles! {
                                recaller => rule!(allow_all);
                                recaller_updater => rule!(deny_all);
                            },
                            ..Default::default()
                        },
                        metadata!(),
                        None,
                    )
                    .build(),
                vec![],
            );
            let resource_address = receipt.expect_commit_success().new_resource_addresses()[0];

            fuzzer.add_resource(resource_address);

            let receipt = ledger.execute_manifest(
                ManifestBuilder::new()
                    .lock_fee_from_faucet()
                    .call_function(
                        package_address,
                        BLUEPRINT_NAME,
                        "new",
                        manifest_args!(resource_address),
                    )
                    .build(),
                vec![],
            );
            let component_address = receipt.expect_commit_success().new_component_addresses()[0];

            let vault_id = ledger.get_component_vaults(component_address, resource_address)[0];

            ResourceComponentMeta {
                component_address,
                resource_address,
                vault_address: InternalAddress::try_from(vault_id).unwrap(),
            }
        };

        let non_fungible_vault_component = {
            let ids: Vec<(NonFungibleLocalId, ())> = fuzzer
                .next_non_fungible_id_set()
                .into_iter()
                .map(|id| (id, ()))
                .collect();
            let amount = ids.len();
            let receipt = ledger.execute_manifest(
                ManifestBuilder::new()
                    .lock_fee_from_faucet()
                    .create_non_fungible_resource(
                        OwnerRole::None,
                        NonFungibleIdType::Integer,
                        true,
                        NonFungibleResourceRoles {
                            mint_roles: mint_roles! {
                                minter => rule!(allow_all);
                                minter_updater => rule!(deny_all);
                            },
                            burn_roles: burn_roles! {
                                burner => rule!(allow_all);
                                burner_updater => rule!(deny_all);
                            },
                            recall_roles: recall_roles! {
                                recaller => rule!(allow_all);
                                recaller_updater => rule!(deny_all);
                            },
                            ..Default::default()
                        },
                        metadata!(),
                        Some(ids),
                    )
                    .deposit_entire_worktop(account)
                    .build(),
                vec![virtual_signature_badge.clone()],
            );
            let resource_address = receipt.expect_commit_success().new_resource_addresses()[0];

            fuzzer.add_resource(resource_address);

            let manifest = {
                let mut builder = ManifestBuilder::new().lock_fee_from_faucet();
                if amount != 0 {
                    builder = builder.withdraw_from_account(account, resource_address, amount);
                }
                builder
                    .take_all_from_worktop(resource_address, "bkt")
                    .with_bucket("bkt", |builder, bucket| {
                        builder.call_function(
                            package_address,
                            BLUEPRINT_NAME,
                            "new_with_bucket",
                            manifest_args!(bucket),
                        )
                    })
                    .build()
            };

            let receipt = ledger.execute_manifest(manifest, vec![virtual_signature_badge]);
            let component_address = receipt.expect_commit_success().new_component_addresses()[0];

            let vault_id = ledger.get_component_vaults(component_address, resource_address)[0];

            ResourceComponentMeta {
                component_address,
                resource_address,
                vault_address: InternalAddress::try_from(vault_id).unwrap(),
            }
        };

        Self {
            fuzzer,
            ledger,
            validators: vec![validator_meta],
            one_resource_pool,
            two_resource_pool,
            multi_resource_pool,
            fungible_meta: fungible_vault_component,
            non_fungible_meta: non_fungible_vault_component,
            account_address: account,
            account_public_key: public_key.into(),
            cur_round: Round::of(1u64),
            txn_fuzzer: PhantomData::default(),
        }
    }

    pub fn run_fuzz(num_tests: u64, num_txns: u64, inject_costing_error: bool) {
        let mut summed_results: BTreeMap<FuzzTxnIntent, BTreeMap<FuzzTxnResult, u64>> =
            BTreeMap::new();

        let results: Vec<BTreeMap<FuzzTxnIntent, BTreeMap<FuzzTxnResult, u64>>> = (1u64
            ..=num_tests)
            .into_par_iter()
            .map(|seed| {
                let mut fuzz_test = Self::new(seed);
                let err_after_account = if inject_costing_error {
                    let err_after_count = fuzz_test.fuzzer.rng.gen_range(200u64..500u64);
                    Some(err_after_count)
                } else {
                    None
                };
                fuzz_test.run_single_fuzz(num_txns, err_after_account)
            })
            .collect();

        if !inject_costing_error {
            for run_result in results {
                for (txn, txn_results) in run_result {
                    for (txn_result, count) in txn_results {
                        summed_results
                            .entry(txn.clone())
                            .or_default()
                            .entry(txn_result)
                            .or_default()
                            .add_assign(&count);
                    }
                }
            }

            let mut missing_success = BTreeSet::new();
            for (intent, results) in &summed_results {
                if !results.contains_key(&FuzzTxnResult::Success) {
                    missing_success.insert(intent);
                }
            }

            if !missing_success.is_empty() {
                panic!("Missing intent success: {:#?}", missing_success);
            }

            println!("{:#?}", summed_results);
        }
    }

    fn run_single_fuzz(
        &mut self,
        num_txns: u64,
        error_after_system_callback_count: Option<u64>,
    ) -> BTreeMap<FuzzTxnIntent, BTreeMap<FuzzTxnResult, u64>> {
        let mut fuzz_results: BTreeMap<FuzzTxnIntent, BTreeMap<FuzzTxnResult, u64>> =
            BTreeMap::new();

        for uuid in 0u64..num_txns {
            // Build new transaction
            let mut builder = ManifestBuilder::new().lock_fee_from_faucet();
            let mut trivial = false;
            let fuzz_txn_intent = T::next_txn_intent(&mut self.fuzzer);
            for fuzz_action in &fuzz_txn_intent {
                let (next_builder, next_trivial) = fuzz_action.add_to_manifest(
                    uuid,
                    builder,
                    &mut self.fuzzer,
                    &self.validators,
                    &self.one_resource_pool,
                    &self.two_resource_pool,
                    &self.multi_resource_pool,
                    &self.fungible_meta,
                    &self.non_fungible_meta,
                    self.account_address,
                );

                trivial = trivial || next_trivial;
                builder = next_builder;
            }

            // Execute transaction
            let result = {
                let manifest = builder
                    .deposit_entire_worktop(self.validators[0].account_address)
                    .build();

                let receipt = if let Some(error_after_count) = error_after_system_callback_count {
                    self.ledger.execute_manifest_with_injected_error(
                        manifest,
                        vec![NonFungibleGlobalId::from_public_key(
                            &self.account_public_key,
                        )],
                        error_after_count,
                    )
                } else {
                    self.ledger.execute_manifest(
                        manifest,
                        vec![NonFungibleGlobalId::from_public_key(
                            &self.account_public_key,
                        )],
                    )
                };

                let result = receipt.result;
                match result {
                    TransactionResult::Commit(commit_result) => {
                        commit_result
                            .new_component_addresses()
                            .iter()
                            .filter(|a| a.as_node_id().is_global_validator())
                            .for_each(|validator_address| {
                                let validator_substate =
                                    self.ledger.get_validator_info(*validator_address);
                                let stake_unit_resource = validator_substate.stake_unit_resource;
                                let claim_resource = validator_substate.claim_nft;

                                self.validators.push(ValidatorMeta {
                                    account_address: self.validators[0].account_address,
                                    stake_unit_resource,
                                    claim_resource,
                                    validator_address: *validator_address,
                                });
                            });

                        if let TransactionOutcome::Failure(RuntimeError::VmError(
                            VmError::Native(NativeRuntimeError::Trap {
                                export_name,
                                input,
                                error,
                            }),
                        )) = &commit_result.outcome
                        {
                            panic!("Native panic: {:?} {:?} {:?}", export_name, input, error);
                        }

                        FuzzTxnResult::from_outcome(&commit_result.outcome, trivial)
                    }
                    TransactionResult::Reject(_) => FuzzTxnResult::Reject,
                    TransactionResult::Abort(_) => panic!("Transaction was aborted"),
                }
            };

            // Execute a consensus round around every 4 transactions
            if self.fuzzer.next(0u8..8u8) == 0u8 {
                let rounds = self.fuzzer.next(1u64..10u64);
                self.consensus_round(rounds);
            }

            let results = fuzz_results.entry(fuzz_txn_intent).or_default();
            results.entry(result).or_default().add_assign(&1);
        }

        fuzz_results
    }

    fn consensus_round(&mut self, num_rounds: u64) {
        let receipt = self
            .ledger
            .advance_to_round(Round::of(self.cur_round.number() + num_rounds));
        let result = receipt.expect_commit_success();
        let events = result.application_events.clone();
        let epoch_change_event = events
            .into_iter()
            .filter(|(id, _data)| self.ledger.is_event_name_equal::<EpochChangeEvent>(id))
            .map(|(_id, data)| scrypto_decode::<EpochChangeEvent>(&data).unwrap())
            .collect::<Vec<_>>()
            .into_iter()
            .next();

        if let Some(..) = epoch_change_event {
            self.cur_round = Round::of(1u64);
        } else {
            self.cur_round = Round::of(self.cur_round.number() + num_rounds);
        }
    }
}
