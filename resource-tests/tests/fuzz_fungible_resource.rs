use native_sdk::resource::NativeVault;
use radix_engine::prelude::node_modules::auth::RoleDefinition;
use radix_engine::types::*;
use radix_engine::vm::{OverridePackageCode};
use radix_engine_interface::api::node_modules::auth::ToRoleEntry;
use radix_engine_interface::blueprints::package::PackageDefinition;
use radix_engine_interface::prelude::node_modules::ModuleConfig;
use radix_engine_stores::memory_db::InMemorySubstateDatabase;
use rayon::iter::IntoParallelIterator;
use rayon::iter::ParallelIterator;
use resource_tests::resource::{
    FungibleResourceFuzzGetBucketAction, ResourceFuzzUseBucketAction, VaultTestInvoke,
    BLUEPRINT_NAME, CUSTOM_PACKAGE_CODE_ID,
};
use resource_tests::{FuzzTxnResult, TestFuzzer};
use scrypto_unit::*;
use transaction::prelude::*;

#[test]
fn fuzz_fungible_resource() {
    let mut summed_results: BTreeMap<ResourceFuzzAction, BTreeMap<FuzzTxnResult, u64>> =
        BTreeMap::new();

    let results: Vec<BTreeMap<ResourceFuzzAction, BTreeMap<FuzzTxnResult, u64>>> = (1u64..=64u64)
        .into_par_iter()
        .map(|seed| {
            let mut fuzz_test = FungibleResourceFuzzTest::new(seed);
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
struct ResourceFuzzAction(
    FungibleResourceFuzzGetBucketAction,
    ResourceFuzzUseBucketAction,
);

struct FungibleResourceFuzzTest {
    fuzzer: TestFuzzer,
    test_runner: TestRunner<OverridePackageCode<VaultTestInvoke>, InMemorySubstateDatabase>,
    resource_address: ResourceAddress,
    component_address: ComponentAddress,
    vault_id: InternalAddress,
    account_public_key: PublicKey,
    account_component_address: ComponentAddress,
}

impl FungibleResourceFuzzTest {
    fn new(seed: u64) -> Self {
        let fuzzer = TestFuzzer::new(seed);
        let mut test_runner = TestRunnerBuilder::new()
            .with_custom_extension(OverridePackageCode::new(
                CUSTOM_PACKAGE_CODE_ID,
                VaultTestInvoke,
            ))
            .build();
        let package_address = test_runner.publish_native_package(
            CUSTOM_PACKAGE_CODE_ID,
            PackageDefinition::new_with_field_test_definition(
                BLUEPRINT_NAME,
                vec![("call_vault", "call_vault", true), ("new", "new", false)],
            ),
        );

        let (public_key, _, account) = test_runner.new_account(false);

        let receipt = test_runner.execute_manifest_ignoring_fee(
            ManifestBuilder::new()
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

        let receipt = test_runner.execute_manifest_ignoring_fee(
            ManifestBuilder::new()
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

        let vault_id = test_runner.get_component_vaults(component_address, resource_address)[0];

        Self {
            fuzzer,
            test_runner,
            resource_address,
            component_address,
            vault_id: InternalAddress::try_from(vault_id).unwrap(),
            account_public_key: public_key.into(),
            account_component_address: account,
        }
    }

    fn run_fuzz(&mut self) -> BTreeMap<ResourceFuzzAction, BTreeMap<FuzzTxnResult, u64>> {
        let mut fuzz_results: BTreeMap<ResourceFuzzAction, BTreeMap<FuzzTxnResult, u64>> =
            BTreeMap::new();
        for _ in 0..500 {
            let builder = ManifestBuilder::new();
            let get_bucket_action =
                FungibleResourceFuzzGetBucketAction::from_repr(self.fuzzer.next_u8(4u8)).unwrap();
            let (mut builder, mut trivial) = get_bucket_action.add_to_manifest(
                builder,
                &mut self.fuzzer,
                self.component_address,
                self.resource_address,
                self.vault_id,
            );

            let use_action =
                ResourceFuzzUseBucketAction::from_repr(self.fuzzer.next_u8(2u8)).unwrap();
            let (mut builder, end_trivial) = use_action.add_to_manifest(
                builder,
                &mut self.fuzzer,
                self.resource_address,
                self.component_address,
            );
            trivial = trivial || end_trivial;

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

            let results = fuzz_results
                .entry(ResourceFuzzAction(get_bucket_action, use_action))
                .or_default();
            results.entry(result).or_default().add_assign(&1);
        }

        fuzz_results
    }
}
