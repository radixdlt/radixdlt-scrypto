use radix_engine::blueprints::pool::multi_resource_pool::MULTI_RESOURCE_POOL_BLUEPRINT_IDENT;
use radix_engine::transaction::TransactionReceipt;
use radix_engine::types::*;
use radix_engine_interface::blueprints::pool::*;
use rayon::iter::IntoParallelIterator;
use rayon::iter::ParallelIterator;
use resource_tests::multi_pool::MultiPoolFuzzAction;
use resource_tests::{FuzzTxnResult, TestFuzzer};
use scrypto_unit::*;
use transaction::prelude::*;

#[test]
fn fuzz_multi_pool() {
    let mut summed_results: BTreeMap<MultiPoolFuzzAction, BTreeMap<FuzzTxnResult, u64>> =
        BTreeMap::new();

    let results: Vec<BTreeMap<MultiPoolFuzzAction, BTreeMap<FuzzTxnResult, u64>>> = (1u64..=1u64)
        .into_par_iter()
        .map(|seed| {
            let mut fuzz_test = MultiPoolFuzzTest::new(seed);
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

struct MultiPoolFuzzTest {
    fuzzer: TestFuzzer,
    test_runner: DefaultTestRunner,
    pool_address: ComponentAddress,
    pool_unit_resource_address: ResourceAddress,
    pool_resources: Vec<ResourceAddress>,
    account_public_key: PublicKey,
    account_address: ComponentAddress,
}

impl MultiPoolFuzzTest {
    pub fn new(seed: u64) -> Self {
        let mut fuzzer = TestFuzzer::new(seed);
        let mut test_runner = TestRunnerBuilder::new().without_trace().build();
        let (public_key, _, account) = test_runner.new_account(false);
        let virtual_signature_badge = NonFungibleGlobalId::from_public_key(&public_key);

        let divisibility = vec![
            fuzzer.next_valid_divisibility(),
            fuzzer.next_valid_divisibility(),
            fuzzer.next_valid_divisibility(),
        ];

        let pool_resources: Vec<ResourceAddress> = divisibility
            .into_iter()
            .map(|divisibility| {
                test_runner.create_freely_mintable_and_burnable_fungible_resource(
                    OwnerRole::None,
                    None,
                    divisibility,
                    account,
                )
            })
            .collect();

        let (pool_component, pool_unit_resource) = {
            let manifest = ManifestBuilder::new()
                .call_function(
                    POOL_PACKAGE,
                    MULTI_RESOURCE_POOL_BLUEPRINT_IDENT,
                    MULTI_RESOURCE_POOL_INSTANTIATE_IDENT,
                    MultiResourcePoolInstantiateManifestInput {
                        resource_addresses: pool_resources.clone().into_iter().collect(),
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

        Self {
            fuzzer,
            test_runner,
            pool_address: pool_component,
            pool_unit_resource_address: pool_unit_resource,
            pool_resources,
            account_public_key: public_key.into(),
            account_address: account,
        }
    }

    fn run_fuzz(&mut self) -> BTreeMap<MultiPoolFuzzAction, BTreeMap<FuzzTxnResult, u64>> {
        let mut fuzz_results: BTreeMap<MultiPoolFuzzAction, BTreeMap<FuzzTxnResult, u64>> =
            BTreeMap::new();

        for _ in 0..100 {
            let builder = ManifestBuilder::new();
            let action: MultiPoolFuzzAction =
                MultiPoolFuzzAction::from_repr(self.fuzzer.next_u8(5u8)).unwrap();
            let (builder, trivial) = action.add_to_manifest(
                builder,
                &mut self.fuzzer,
                self.account_address,
                self.pool_address,
                self.pool_unit_resource_address,
                &self.pool_resources,
            );
            let manifest = builder.deposit_batch(self.account_address).build();
            let receipt = self.test_runner.execute_manifest_ignoring_fee(
                manifest,
                vec![NonFungibleGlobalId::from_public_key(
                    &self.account_public_key,
                )],
            );
            let commit_result = receipt.expect_commit_ignore_outcome();
            let result = FuzzTxnResult::from_outcome(&commit_result.outcome, trivial);

            let results = fuzz_results.entry(action).or_default();
            results.entry(result).or_default().add_assign(&1);
        }

        fuzz_results
    }
}
