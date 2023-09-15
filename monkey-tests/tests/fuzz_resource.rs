use monkey_tests::resource::ResourceFuzzRandomAction;
use monkey_tests::{FuzzAction, FuzzTest, SystemTestFuzzer, TxnFuzzer};
use radix_engine::types::*;

#[test]
fn fuzz_resource() {
    struct ResourceFuzzer;
    impl TxnFuzzer for ResourceFuzzer {
        fn next_txn_intent(fuzzer: &mut SystemTestFuzzer) -> Vec<FuzzAction> {
            let action0: ResourceFuzzRandomAction =
                ResourceFuzzRandomAction::from_repr(fuzzer.next(0u8..=2u8)).unwrap();
            let action1: ResourceFuzzRandomAction =
                ResourceFuzzRandomAction::from_repr(fuzzer.next(0u8..=2u8)).unwrap();
            let action2: ResourceFuzzRandomAction =
                ResourceFuzzRandomAction::from_repr(fuzzer.next(3u8..=4u8)).unwrap();

            vec![
                FuzzAction::Resource(action0),
                FuzzAction::Resource(action1),
                FuzzAction::Resource(action2),
            ]
        }
    }

    FuzzTest::<ResourceFuzzer>::run_fuzz(10, 15000, false);
}
