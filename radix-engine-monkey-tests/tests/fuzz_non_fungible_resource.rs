use radix_common::prelude::*;
use radix_engine_monkey_tests::resource::{
    NonFungibleResourceFuzzGetBucketAction, ResourceFuzzUseBucketAction,
};
use radix_engine_monkey_tests::{FuzzAction, FuzzTest, SystemTestFuzzer, TxnFuzzer};

#[test]
fn fuzz_non_fungible_resource() {
    struct NonFungibleResourceFuzzer;
    impl TxnFuzzer for NonFungibleResourceFuzzer {
        fn next_txn_intent(fuzzer: &mut SystemTestFuzzer) -> Vec<FuzzAction> {
            let action1: NonFungibleResourceFuzzGetBucketAction =
                NonFungibleResourceFuzzGetBucketAction::from_repr(fuzzer.next_u8(6u8)).unwrap();

            let action2: ResourceFuzzUseBucketAction =
                ResourceFuzzUseBucketAction::from_repr(fuzzer.next_u8(2u8)).unwrap();

            vec![
                FuzzAction::NonFungibleGetBucket(action1),
                FuzzAction::NonFungibleUseBucket(action2),
            ]
        }
    }

    FuzzTest::<NonFungibleResourceFuzzer>::run_fuzz(32, 1000, false);
}
