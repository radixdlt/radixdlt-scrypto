use radix_engine::types::*;
use resource_tests::resource::{
    ResourceFuzzRandomAction,
};
use resource_tests::{FuzzAction, FuzzTest, TestFuzzer, TxnFuzzer};

#[test]
fn fuzz_resource() {
    struct ResourceFuzzer;
    impl TxnFuzzer for ResourceFuzzer {
        fn next_txn_intent(fuzzer: &mut TestFuzzer) -> Vec<FuzzAction> {
            let action0: ResourceFuzzRandomAction =
                ResourceFuzzRandomAction::from_repr(fuzzer.next(0u8..=2u8)).unwrap();
            let action1: ResourceFuzzRandomAction =
                ResourceFuzzRandomAction::from_repr(fuzzer.next(0u8..=2u8)).unwrap();

            vec![
                FuzzAction::Resource(action0),
                FuzzAction::Resource(action1),
                FuzzAction::Resource(ResourceFuzzRandomAction::CombineBuckets),
            ]
        }
    }

    FuzzTest::<ResourceFuzzer>::run_fuzz(8, 10000);
}
