use radix_engine::types::*;
use resource_tests::two_pool::TwoPoolFuzzAction;
use resource_tests::{FuzzAction, FuzzTest, TestFuzzer, TxnFuzzer};

#[test]
fn fuzz_two_pool() {
    struct TwoResourcePoolFuzzer;
    impl TxnFuzzer for TwoResourcePoolFuzzer {
        fn next_txn_intent(fuzzer: &mut TestFuzzer) -> Vec<FuzzAction> {
            let action: TwoPoolFuzzAction =
                TwoPoolFuzzAction::from_repr(fuzzer.next_u8(8u8)).unwrap();
            vec![FuzzAction::TwoResourcePool(action)]
        }
    }

    FuzzTest::<TwoResourcePoolFuzzer>::run_fuzz(32, 100);
}
