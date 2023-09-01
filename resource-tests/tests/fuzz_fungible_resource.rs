use radix_engine::prelude::node_modules::auth::RoleDefinition;
use radix_engine::types::*;
use radix_engine::vm::OverridePackageCode;
use radix_engine_interface::api::node_modules::auth::ToRoleEntry;
use radix_engine_interface::blueprints::package::PackageDefinition;
use radix_engine_interface::prelude::node_modules::ModuleConfig;
use radix_engine_stores::memory_db::InMemorySubstateDatabase;
use rayon::iter::IntoParallelIterator;
use rayon::iter::ParallelIterator;
use resource_tests::resource::{
    FungibleResourceFuzzGetBucketAction, ResourceFuzzUseBucketAction, VaultTestInvoke,
};
use resource_tests::{FuzzAction, FuzzTest, FuzzTxnResult, TestFuzzer, TxnFuzzer};
use scrypto_unit::*;

#[test]
fn fuzz_fungible_resource() {
    struct FungibleResourceFuzzer;
    impl TxnFuzzer for FungibleResourceFuzzer {
        fn next_txn_intent(fuzzer: &mut TestFuzzer) -> Vec<FuzzAction> {
            let action1: FungibleResourceFuzzGetBucketAction =
                FungibleResourceFuzzGetBucketAction::from_repr(fuzzer.next_u8(4u8)).unwrap();

            let action2: ResourceFuzzUseBucketAction =
                ResourceFuzzUseBucketAction::from_repr(fuzzer.next_u8(2u8)).unwrap();

            vec![FuzzAction::FungibleGetBucket(action1), FuzzAction::UseBucket(action2)]
        }
    }

    FuzzTest::<FungibleResourceFuzzer>::run_fuzz();
}
