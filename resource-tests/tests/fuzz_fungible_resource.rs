use radix_engine::types::*;
use resource_tests::resource::{
    FungibleResourceFuzzGetBucketAction, ResourceFuzzRandomAction,
    ResourceFuzzTransformBucketAction, ResourceFuzzUseBucketAction,
};
use resource_tests::{FuzzAction, FuzzTest, TestFuzzer, TxnFuzzer};

#[test]
fn fuzz_fungible_resource() {
    struct FungibleResourceFuzzer;
    impl TxnFuzzer for FungibleResourceFuzzer {
        fn next_txn_intent(fuzzer: &mut TestFuzzer) -> Vec<FuzzAction> {
            let action1: FungibleResourceFuzzGetBucketAction =
                FungibleResourceFuzzGetBucketAction::from_repr(fuzzer.next_u8(4u8)).unwrap();

            let action2: ResourceFuzzUseBucketAction =
                ResourceFuzzUseBucketAction::from_repr(fuzzer.next_u8(2u8)).unwrap();

            vec![
                FuzzAction::FungibleGetBucket(action1),
                FuzzAction::FungibleBucketTransform(ResourceFuzzTransformBucketAction::Combine),
                FuzzAction::FungibleUseBucket(action2),
            ]
        }
    }

    FuzzTest::<FungibleResourceFuzzer>::run_fuzz(16, 500);
}

#[test]
fn fuzz_resource() {
    struct ResourceFuzzer;
    impl TxnFuzzer for ResourceFuzzer {
        fn next_txn_intent(fuzzer: &mut TestFuzzer) -> Vec<FuzzAction> {
            let action0: ResourceFuzzRandomAction =
                ResourceFuzzRandomAction::from_repr(fuzzer.next(0u8..=1u8)).unwrap();
            let action1: ResourceFuzzRandomAction =
                ResourceFuzzRandomAction::from_repr(fuzzer.next(0u8..=1u8)).unwrap();

            vec![
                FuzzAction::Resource(action0),
                FuzzAction::Resource(action1),
                FuzzAction::Resource(ResourceFuzzRandomAction::CombineBuckets),
            ]
        }
    }

    FuzzTest::<ResourceFuzzer>::run_fuzz(8, 10000);
}
