use radix_engine::transaction::{TransactionOutcome, TransactionReceipt, TransactionResult};
use radix_engine::types::*;
use radix_engine_interface::blueprints::pool::*;
use rayon::iter::IntoParallelIterator;
use rayon::iter::ParallelIterator;
use radix_engine::blueprints::consensus_manager::EpochChangeEvent;
use radix_engine::system::bootstrap::{DEFAULT_TESTING_FAUCET_SUPPLY, GenesisDataChunk, GenesisStakeAllocation, GenesisValidator};
use radix_engine_interface::blueprints::consensus_manager::{VALIDATOR_CLAIM_XRD_IDENT, VALIDATOR_GET_REDEMPTION_VALUE_IDENT, VALIDATOR_STAKE_AS_OWNER_IDENT, VALIDATOR_STAKE_IDENT, VALIDATOR_UNSTAKE_IDENT, ValidatorGetRedemptionValueInput, ValidatorStakeInput};
use resource_tests::ResourceTestFuzzer;
use scrypto_unit::*;
use transaction::prelude::*;

#[test]
fn fuzz_consensus() {
    let results: Vec<BTreeMap<ConsensusFuzzAction, BTreeMap<ConsensusFuzzActionResult, u64>>> = (1u64..64u64).into_par_iter().map(|seed| {
        let mut one_pool_fuzz_test = ConsensusFuzzTest::new(seed);
        one_pool_fuzz_test.run_fuzz()
    }).collect();

    println!("{:#?}", results);

    panic!("oops");
}

#[repr(u8)]
#[derive(Copy, Clone, Debug, FromRepr, Ord, PartialOrd, Eq, PartialEq)]
enum ConsensusFuzzAction {
    GetRedemptionValue,
    Stake,
    Unstake,
    Claim,
    ConsensusRound,
}

#[repr(u8)]
#[derive(Copy, Clone, Debug, FromRepr, Ord, PartialOrd, Eq, PartialEq)]
enum ConsensusFuzzActionResult {
    TrivialSuccess,
    Success,
    TrivialFailure,
    Failure,
}

struct ConsensusFuzzTest {
    fuzzer: ResourceTestFuzzer,
    test_runner: DefaultTestRunner,
    validator_address: ComponentAddress,
    stake_unit_resource: ResourceAddress,
    claim_resource: ResourceAddress,
    account_public_key: PublicKey,
    account_component_address: ComponentAddress,
    cur_round: Round,
}

impl ConsensusFuzzTest {
    fn new(seed: u64) -> Self {
        let mut fuzzer = ResourceTestFuzzer::new(seed);
        let initial_epoch = Epoch::of(5);
        let genesis = CustomGenesis::default_with_xrd_amount(
            Decimal::from(24_000_000_000u64),
            initial_epoch,
            CustomGenesis::default_consensus_manager_config(),
        );
        let (test_runner, validator_set) = TestRunnerBuilder::new()
            .with_custom_genesis(genesis)
            .build_and_get_epoch();
        let public_key = Secp256k1PrivateKey::from_u64(1u64).unwrap().public_key();
        let account = ComponentAddress::virtual_account_from_public_key(&public_key);

        let validator_address = validator_set.validators_by_stake_desc.iter().next().unwrap().0.clone();
        let validator_substate = test_runner.get_validator_info(validator_address);
        let stake_unit_resource = validator_substate.stake_unit_resource;
        let claim_resource = validator_substate.claim_nft;


        Self {
            fuzzer,
            test_runner,
            validator_address,
            stake_unit_resource,
            claim_resource,
            account_public_key: public_key.into(),
            account_component_address: account,
            cur_round: Round::of(1u64),
        }
    }

    fn run_fuzz(&mut self) -> BTreeMap<ConsensusFuzzAction, BTreeMap<ConsensusFuzzActionResult, u64>> {
        let mut fuzz_results: BTreeMap<ConsensusFuzzAction, BTreeMap<ConsensusFuzzActionResult, u64>> = BTreeMap::new();
        for _ in 0..100 {
            let action = ConsensusFuzzAction::from_repr(self.fuzzer.next_u8(5u8)).unwrap();
            let (trivial, receipt) = match action {
                ConsensusFuzzAction::GetRedemptionValue => {
                    let amount = self.fuzzer.next_amount();
                    (amount.is_zero(), self.get_redemption_value(amount))
                }
                ConsensusFuzzAction::Stake => {
                    let amount = self.fuzzer.next_amount();
                    (amount.is_zero(), self.stake(amount))
                }
                ConsensusFuzzAction::Unstake => {
                    let amount = self.fuzzer.next_amount();
                    (amount.is_zero(), self.unstake(amount))
                }
                ConsensusFuzzAction::Claim => {
                    let amount = self.fuzzer.next_amount();
                    (amount.is_zero(), self.claim(amount))
                }
                ConsensusFuzzAction::ConsensusRound => {
                    let rounds = self.fuzzer.next(1u64..10u64);
                    (false, self.consensus_round(rounds))
                }
            };

            let result = receipt.expect_commit_ignore_outcome();
            let result = match (&result.outcome, trivial) {
                (TransactionOutcome::Success(..), true) => ConsensusFuzzActionResult::TrivialSuccess,
                (TransactionOutcome::Success(..), false) => ConsensusFuzzActionResult::Success,
                (TransactionOutcome::Failure(..), true) => ConsensusFuzzActionResult::TrivialFailure,
                (TransactionOutcome::Failure(..), false) => ConsensusFuzzActionResult::Failure,
            };

            let results = fuzz_results.entry(action).or_default();
            results.entry(result).or_default().add_assign(&1);
        }

        fuzz_results
    }

    fn get_redemption_value(
        &mut self,
        amount_of_stake_units: Decimal,
    ) -> TransactionReceipt {
        let manifest = ManifestBuilder::new()
            .call_method(
                self.validator_address,
                VALIDATOR_GET_REDEMPTION_VALUE_IDENT,
                ValidatorGetRedemptionValueInput {
                    amount_of_stake_units,
                },
            )
            .build();
        self.test_runner
            .execute_manifest_ignoring_fee(manifest, vec![])
    }

    fn stake(
        &mut self,
        amount_to_stake: Decimal,
    ) -> TransactionReceipt {
        let manifest = ManifestBuilder::new()
            .withdraw_from_account(self.account_component_address, XRD, amount_to_stake)
            .take_all_from_worktop(XRD, "xrd")
            .with_bucket("xrd", |builder, bucket| {
                builder.call_method(
                    self.validator_address,
                    VALIDATOR_STAKE_IDENT,
                    manifest_args!(bucket),
                )
            })
            .deposit_batch(self.account_component_address)
            .build();
        self.test_runner
            .execute_manifest_ignoring_fee(manifest, vec![NonFungibleGlobalId::from_public_key(&self.account_public_key)])
    }

    fn unstake(
        &mut self,
        amount: Decimal,
    ) -> TransactionReceipt {
        let manifest = ManifestBuilder::new()
            .withdraw_from_account(self.account_component_address, self.stake_unit_resource, amount)
            .take_all_from_worktop(self.stake_unit_resource, "stake_units")
            .with_bucket("stake_units", |builder, bucket| {
                builder.call_method(
                    self.validator_address,
                    VALIDATOR_UNSTAKE_IDENT,
                    manifest_args!(bucket),
                )
            })
            .deposit_batch(self.account_component_address)
            .build();
        self.test_runner
            .execute_manifest_ignoring_fee(manifest, vec![NonFungibleGlobalId::from_public_key(&self.account_public_key)])
    }

    fn claim(
        &mut self,
        amount: Decimal,
    ) -> TransactionReceipt {
        let manifest = ManifestBuilder::new()
            .withdraw_from_account(self.account_component_address, self.claim_resource, amount)
            .take_all_from_worktop(self.claim_resource, "claim_resource")
            .with_bucket("claim_resource", |builder, bucket| {
                builder.call_method(
                    self.validator_address,
                    VALIDATOR_CLAIM_XRD_IDENT,
                    manifest_args!(bucket),
                )
            })
            .deposit_batch(self.account_component_address)
            .build();
        self.test_runner
            .execute_manifest_ignoring_fee(manifest, vec![NonFungibleGlobalId::from_public_key(&self.account_public_key)])
    }

    fn consensus_round(
        &mut self,
        num_rounds: u64,
    ) -> TransactionReceipt {
        let receipt = self.test_runner.advance_to_round(Round::of(self.cur_round.number() + num_rounds));
        let result = receipt.expect_commit_success();
        let events = result.application_events.clone();
        let epoch_change_event = events
            .into_iter()
            .filter(|(id, _data)| self.test_runner.is_event_name_equal::<EpochChangeEvent>(id))
            .map(|(_id, data)| scrypto_decode::<EpochChangeEvent>(&data).unwrap())
            .collect::<Vec<_>>().into_iter().next();

        if let Some(..) = epoch_change_event {
            self.cur_round = Round::of(1u64);
        } else {
            self.cur_round = Round::of(self.cur_round.number() + num_rounds);
        }

        receipt
    }
}
