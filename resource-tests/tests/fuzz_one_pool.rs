use radix_engine::transaction::{TransactionOutcome, TransactionReceipt};
use radix_engine::types::*;
use radix_engine_interface::blueprints::pool::*;
use rayon::iter::IntoParallelIterator;
use rayon::iter::ParallelIterator;
use resource_tests::one_pool::OnePoolFuzzAction;
use resource_tests::{FuzzTxnResult, TestFuzzer};
use scrypto_unit::*;
use transaction::prelude::*;

#[test]
fn fuzz_one_pool() {
    let mut summed_results: BTreeMap<OnePoolFuzzAction, BTreeMap<FuzzTxnResult, u64>> =
        BTreeMap::new();

    let results: Vec<BTreeMap<OnePoolFuzzAction, BTreeMap<FuzzTxnResult, u64>>> = (1u64..=64u64)
        .into_par_iter()
        .map(|seed| {
            let mut fuzz_test = OnePoolFuzzTest::new(seed);
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

struct OnePoolFuzzTest {
    fuzzer: TestFuzzer,
    test_runner: DefaultTestRunner,
    pool_component_address: ComponentAddress,
    pool_unit_resource_address: ResourceAddress,
    resource_address: ResourceAddress,
    account_public_key: PublicKey,
    account_component_address: ComponentAddress,
}

impl OnePoolFuzzTest {
    fn new(seed: u64) -> Self {
        let mut fuzzer = TestFuzzer::new(seed);
        let mut test_runner = TestRunnerBuilder::new().without_trace().build();
        let (public_key, _, account) = test_runner.new_account(false);
        let virtual_signature_badge = NonFungibleGlobalId::from_public_key(&public_key);

        let resource_address = test_runner.create_freely_mintable_and_burnable_fungible_resource(
            OwnerRole::None,
            None,
            fuzzer.next_u8(19),
            account,
        );

        let (pool_component, pool_unit_resource) = test_runner
            .create_one_resource_pool(resource_address, rule!(require(virtual_signature_badge)));

        Self {
            fuzzer,
            test_runner,
            pool_component_address: pool_component,
            pool_unit_resource_address: pool_unit_resource,
            resource_address,
            account_public_key: public_key.into(),
            account_component_address: account,
        }
    }

    fn run_fuzz(&mut self) -> BTreeMap<OnePoolFuzzAction, BTreeMap<FuzzTxnResult, u64>> {
        let mut fuzz_results: BTreeMap<OnePoolFuzzAction, BTreeMap<FuzzTxnResult, u64>> =
            BTreeMap::new();

        for _ in 0..100 {
            let builder = ManifestBuilder::new();
            let action: OnePoolFuzzAction =
                OnePoolFuzzAction::from_repr(self.fuzzer.next_u8(5u8)).unwrap();
            let (builder, trivial) = action.add_to_manifest(
                builder,
                &mut self.fuzzer,
                self.account_component_address,
                self.pool_component_address,
                self.pool_unit_resource_address,
                self.resource_address,
            );
            let manifest = builder
                .deposit_batch(self.account_component_address)
                .build();
            let receipt = self.test_runner.execute_manifest_ignoring_fee(
                manifest,
                vec![NonFungibleGlobalId::from_public_key(
                    &self.account_public_key,
                )],
            );
            let result = receipt.expect_commit_ignore_outcome();
            let result = FuzzTxnResult::from_outcome(&result.outcome, trivial);

            let results = fuzz_results.entry(action).or_default();
            results.entry(result).or_default().add_assign(&1);
        }

        fuzz_results
    }
}
