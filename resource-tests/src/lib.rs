pub mod consensus_manager;
pub mod multi_pool;
pub mod one_pool;
pub mod resource;
pub mod two_pool;
pub mod validator;

use crate::consensus_manager::ConsensusManagerFuzzAction;
use crate::one_pool::OnePoolFuzzAction;
use crate::two_pool::TwoPoolFuzzAction;
use crate::validator::ValidatorFuzzAction;
use radix_engine::blueprints::consensus_manager::EpochChangeEvent;
use radix_engine::blueprints::pool::one_resource_pool;
use radix_engine::blueprints::pool::two_resource_pool::TWO_RESOURCE_POOL_BLUEPRINT_IDENT;
use radix_engine::transaction::TransactionOutcome;
use radix_engine::types::*;
use radix_engine_interface::blueprints::pool::{
    TwoResourcePoolInstantiateManifestInput, TWO_RESOURCE_POOL_INSTANTIATE_IDENT,
};
use rand::distributions::uniform::{SampleRange, SampleUniform};
use rand::Rng;
use rand_chacha::rand_core::{RngCore, SeedableRng};
use rand_chacha::ChaCha8Rng;
use rayon::iter::IntoParallelIterator;
use rayon::iter::ParallelIterator;
use scrypto_unit::{CustomGenesis, DefaultTestRunner, TestRunnerBuilder};
use transaction::builder::ManifestBuilder;
use transaction::prelude::Secp256k1PrivateKey;

pub struct TestFuzzer {
    rng: ChaCha8Rng,
}

impl TestFuzzer {
    pub fn new(seed: u64) -> Self {
        let rng = ChaCha8Rng::seed_from_u64(seed);
        Self { rng }
    }

    pub fn next_amount(&mut self) -> Decimal {
        let next_amount_type = self.rng.gen_range(0u32..=8u32);
        match next_amount_type {
            0 => Decimal::ZERO,
            1 => Decimal::ONE,
            2 => Decimal::MAX,
            3 => Decimal::MIN,
            4 => Decimal(I192::ONE),
            5 => {
                let amount = self.rng.gen_range(0u64..u64::MAX);
                Decimal::from(amount)
            }
            6 => {
                let amount = self.rng.gen_range(1000u64..10000u64);
                Decimal::from(amount)
            }
            7 => {
                let mut bytes = [0u8; 24];
                let (start, _end) = bytes.split_at_mut(8);
                self.rng.fill_bytes(start);
                Decimal(I192::from_le_bytes(&bytes))
            }
            _ => {
                let mut bytes = [0u8; 24];
                self.rng.fill_bytes(&mut bytes);
                Decimal(I192::from_le_bytes(&bytes))
            }
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
        (0u64..self.rng.gen_range(0u64..4u64))
            .into_iter()
            .map(|_| self.next_integer_non_fungible_id())
            .collect()
    }

    pub fn next<T, R>(&mut self, range: R) -> T
    where
        T: SampleUniform,
        R: SampleRange<T>,
    {
        self.rng.gen_range(range)
    }

    pub fn next_withdraw_strategy(&mut self) -> WithdrawStrategy {
        match self.next_u32(4) {
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
}

#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq)]
pub enum FuzzAction {
    ConsensusManager(ConsensusManagerFuzzAction),
    Validator(ValidatorFuzzAction),
    OneResourcePool(OnePoolFuzzAction),
    TwoResourcePool(TwoPoolFuzzAction),
}

impl FuzzAction {
    pub fn add_to_manifest(
        &self,
        uuid: u64,
        builder: ManifestBuilder,
        fuzzer: &mut TestFuzzer,
        validators: &Vec<ValidatorMeta>,
        one_resource_pool: &OnePoolMeta,
        two_resource_pool: &TwoPoolMeta,
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

pub trait TxnFuzzer {
    fn next_action(fuzzer: &mut TestFuzzer) -> FuzzAction;
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

pub struct FuzzTest<T: TxnFuzzer> {
    test_runner: DefaultTestRunner,
    fuzzer: TestFuzzer,
    validators: Vec<ValidatorMeta>,
    one_resource_pool: OnePoolMeta,
    two_resource_pool: TwoPoolMeta,
    account_address: ComponentAddress,
    account_public_key: PublicKey,
    cur_round: Round,
    txn_fuzzer: PhantomData<T>,
}

impl<T: TxnFuzzer> FuzzTest<T> {
    fn new(seed: u64) -> Self {
        let mut fuzzer = TestFuzzer::new(seed);
        let initial_epoch = Epoch::of(5);
        let genesis = CustomGenesis::default_with_xrd_amount(
            Decimal::from(24_000_000_000u64),
            initial_epoch,
            CustomGenesis::default_consensus_manager_config(),
        );
        let (mut test_runner, validator_set) = TestRunnerBuilder::new()
            .with_custom_genesis(genesis)
            .build_and_get_epoch();
        let public_key = Secp256k1PrivateKey::from_u64(1u64).unwrap().public_key();
        let account = ComponentAddress::virtual_account_from_public_key(&public_key);
        let virtual_signature_badge = NonFungibleGlobalId::from_public_key(&public_key);

        let validator_address = validator_set
            .validators_by_stake_desc
            .iter()
            .next()
            .unwrap()
            .0
            .clone();
        let validator_substate = test_runner.get_validator_info(validator_address);
        let stake_unit_resource = validator_substate.stake_unit_resource;
        let claim_resource = validator_substate.claim_nft;

        let one_resource_pool = {
            let one_pool_resource = test_runner
                .create_freely_mintable_and_burnable_fungible_resource(
                    OwnerRole::None,
                    None,
                    fuzzer.next(0u8..=18u8),
                    account,
                );

            let (pool_address, pool_unit_resource_address) = test_runner.create_one_resource_pool(
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
            let pool_resource1 = test_runner.create_freely_mintable_and_burnable_fungible_resource(
                OwnerRole::None,
                None,
                fuzzer.next_valid_divisibility(),
                account,
            );
            let pool_resource2 = test_runner.create_freely_mintable_and_burnable_fungible_resource(
                OwnerRole::None,
                None,
                fuzzer.next_valid_divisibility(),
                account,
            );

            let (pool_component, pool_unit_resource) = {
                let manifest = ManifestBuilder::new()
                    .call_function(
                        POOL_PACKAGE,
                        TWO_RESOURCE_POOL_BLUEPRINT_IDENT,
                        TWO_RESOURCE_POOL_INSTANTIATE_IDENT,
                        TwoResourcePoolInstantiateManifestInput {
                            resource_addresses: (pool_resource1, pool_resource2),
                            pool_manager_rule: rule!(require(virtual_signature_badge)),
                            owner_role: OwnerRole::None,
                            address_reservation: None,
                        },
                    )
                    .build();
                let receipt = test_runner.execute_manifest_ignoring_fee(manifest, vec![]);
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

        Self {
            fuzzer,
            test_runner,
            validators: vec![ValidatorMeta {
                validator_address,
                stake_unit_resource,
                claim_resource,
                account_address: account,
            }],
            one_resource_pool,
            two_resource_pool,
            account_address: account,
            account_public_key: public_key.into(),
            cur_round: Round::of(1u64),
            txn_fuzzer: PhantomData::default(),
        }
    }

    pub fn run_fuzz() {
        let mut summed_results: BTreeMap<FuzzAction, BTreeMap<FuzzTxnResult, u64>> =
            BTreeMap::new();

        let results: Vec<BTreeMap<FuzzAction, BTreeMap<FuzzTxnResult, u64>>> = (1u64..=32u64)
            .into_par_iter()
            .map(|seed| {
                let mut fuzz_test = Self::new(seed);
                fuzz_test.run_single_fuzz()
            })
            .collect();

        for run_result in results {
            for (txn, txn_results) in run_result {
                for (txn_result, count) in txn_results {
                    summed_results
                        .entry(txn)
                        .or_default()
                        .entry(txn_result)
                        .or_default()
                        .add_assign(&count);
                }
            }
        }

        for (action, results) in &summed_results {
            if !results.contains_key(&FuzzTxnResult::Success) {
                panic!("No successful {:?}", action)
            }
        }

        println!("{:#?}", summed_results);
    }

    fn run_single_fuzz(&mut self) -> BTreeMap<FuzzAction, BTreeMap<FuzzTxnResult, u64>> {
        let mut fuzz_results: BTreeMap<FuzzAction, BTreeMap<FuzzTxnResult, u64>> = BTreeMap::new();

        for uuid in 0u64..100u64 {
            // Build new transaction
            let builder = ManifestBuilder::new();
            let fuzz_action = T::next_action(&mut self.fuzzer);
            let (builder, trivial) = fuzz_action.add_to_manifest(
                uuid,
                builder,
                &mut self.fuzzer,
                &self.validators,
                &self.one_resource_pool,
                &self.two_resource_pool,
                self.account_address,
            );

            // Execute transaction
            let result = {
                let manifest = builder
                    .deposit_batch(self.validators[0].account_address)
                    .build();
                let receipt = self.test_runner.execute_manifest_ignoring_fee(
                    manifest,
                    vec![NonFungibleGlobalId::from_public_key(
                        &self.account_public_key,
                    )],
                );
                let result = receipt.expect_commit_ignore_outcome();

                result
                    .new_component_addresses()
                    .iter()
                    .filter(|a| a.as_node_id().is_global_validator())
                    .for_each(|validator_address| {
                        let validator_substate =
                            self.test_runner.get_validator_info(*validator_address);
                        let stake_unit_resource = validator_substate.stake_unit_resource;
                        let claim_resource = validator_substate.claim_nft;

                        self.validators.push(ValidatorMeta {
                            account_address: self.validators[0].account_address,
                            stake_unit_resource,
                            claim_resource,
                            validator_address: *validator_address,
                        });
                    });

                FuzzTxnResult::from_outcome(&result.outcome, trivial)
            };

            // Execute a consensus round around every 4 transactions
            if self.fuzzer.next(0u8..8u8) == 0u8 {
                let rounds = self.fuzzer.next(1u64..10u64);
                self.consensus_round(rounds);
            }

            let results = fuzz_results.entry(fuzz_action).or_default();
            results.entry(result).or_default().add_assign(&1);
        }

        fuzz_results
    }

    fn consensus_round(&mut self, num_rounds: u64) {
        let receipt = self
            .test_runner
            .advance_to_round(Round::of(self.cur_round.number() + num_rounds));
        let result = receipt.expect_commit_success();
        let events = result.application_events.clone();
        let epoch_change_event = events
            .into_iter()
            .filter(|(id, _data)| self.test_runner.is_event_name_equal::<EpochChangeEvent>(id))
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
