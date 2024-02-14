use monkey_tests::two_pool::TwoPoolFuzzAction;
use monkey_tests::{FuzzAction, FuzzTest, SystemTestFuzzer, TxnFuzzer};
use radix_engine_common::prelude::*;

#[test]
fn fuzz_two_pool() {
    struct TwoResourcePoolFuzzer;
    impl TxnFuzzer for TwoResourcePoolFuzzer {
        fn next_txn_intent(fuzzer: &mut SystemTestFuzzer) -> Vec<FuzzAction> {
            let action: TwoPoolFuzzAction =
                TwoPoolFuzzAction::from_repr(fuzzer.next_u8(8u8)).unwrap();
            vec![FuzzAction::TwoResourcePool(action)]
        }
    }

    FuzzTest::<TwoResourcePoolFuzzer>::run_fuzz(32, 100, false);
}
