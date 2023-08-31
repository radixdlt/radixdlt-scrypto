use radix_engine::blueprints::consensus_manager::EpochChangeEvent;
use radix_engine::transaction::TransactionOutcome;
use radix_engine::types::*;
use rayon::iter::IntoParallelIterator;
use rayon::iter::ParallelIterator;
use resource_tests::consensus_manager::ConsensusManagerFuzzAction;
use resource_tests::validator::{ValidatorFuzzAction, ValidatorMeta};
use resource_tests::{FuzzTxnResult, TestFuzzer};
use scrypto_unit::*;
use transaction::prelude::*;

#[test]
fn fuzz_consensus() {
    let mut summed_results: BTreeMap<ConsensusFuzzAction, BTreeMap<FuzzTxnResult, u64>> =
        BTreeMap::new();

    let results: Vec<BTreeMap<ConsensusFuzzAction, BTreeMap<FuzzTxnResult, u64>>> = (1u64..=64u64)
        .into_par_iter()
        .map(|seed| {
            let mut fuzz_test = ConsensusFuzzTest::new(seed);
            fuzz_test.run_fuzz()
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

    println!("{:#?}", summed_results);
}

#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq)]
enum ConsensusFuzzAction {
    ConsensusManager(ConsensusManagerFuzzAction),
    Validator(ValidatorFuzzAction),
}

struct ConsensusFuzzTest {
    fuzzer: TestFuzzer,
    test_runner: DefaultTestRunner,
    validator_meta: Vec<ValidatorMeta>,
    account_public_key: PublicKey,
    cur_round: Round,
}

impl ConsensusFuzzTest {
    fn new(seed: u64) -> Self {
        let fuzzer = TestFuzzer::new(seed);
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

        Self {
            fuzzer,
            test_runner,
            validator_meta: vec![ValidatorMeta {
                validator_address,
                stake_unit_resource,
                claim_resource,
                account_address: account,
            }],
            account_public_key: public_key.into(),
            cur_round: Round::of(1u64),
        }
    }

    fn run_fuzz(&mut self) -> BTreeMap<ConsensusFuzzAction, BTreeMap<FuzzTxnResult, u64>> {
        let mut fuzz_results: BTreeMap<ConsensusFuzzAction, BTreeMap<FuzzTxnResult, u64>> =
            BTreeMap::new();

        for i in 0u64..1000u64 {
            // Build new transaction
            let (fuzz_action, builder, trivial) = {
                let builder = ManifestBuilder::new();

                match self.fuzzer.next(0u8..10u8) {
                    0u8 => {
                        let fuzz_action = ConsensusFuzzAction::ConsensusManager(
                            ConsensusManagerFuzzAction::CreateValidator,
                        );
                        let (builder, trivial) = ConsensusManagerFuzzAction::CreateValidator
                            .add_to_manifest(
                                i,
                                builder,
                                &mut self.fuzzer,
                                self.validator_meta[0].account_address,
                            );
                        (fuzz_action, builder, trivial)
                    }
                    _ => {
                        let next_validator = self.fuzzer.next(0usize..self.validator_meta.len());
                        let action: ValidatorFuzzAction =
                            ValidatorFuzzAction::from_repr(self.fuzzer.next_u8(8u8)).unwrap();
                        let (builder, trivial) = action.add_to_manifest(
                            builder,
                            &mut self.fuzzer,
                            self.validator_meta[next_validator],
                        );
                        (ConsensusFuzzAction::Validator(action), builder, trivial)
                    }
                }
            };

            // Execute transaction
            let result = {
                let manifest = builder
                    .deposit_batch(self.validator_meta[0].account_address)
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

                        self.validator_meta.push(ValidatorMeta {
                            account_address: self.validator_meta[0].account_address,
                            stake_unit_resource,
                            claim_resource,
                            validator_address: *validator_address,
                        });
                    });

                FuzzTxnResult::from_outcome(&result.outcome, trivial)
            };

            // Execute a consensus round every 4 transactions
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
