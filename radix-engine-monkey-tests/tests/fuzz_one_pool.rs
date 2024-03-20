use radix_common::prelude::*;
use radix_engine_monkey_tests::one_pool::OnePoolFuzzAction;
use radix_engine_monkey_tests::{FuzzAction, FuzzTest, SystemTestFuzzer, TxnFuzzer};

#[test]
fn fuzz_one_pool() {
    struct OneResourcePoolFuzzer;
    impl TxnFuzzer for OneResourcePoolFuzzer {
        fn next_txn_intent(fuzzer: &mut SystemTestFuzzer) -> Vec<FuzzAction> {
            let action: OnePoolFuzzAction =
                OnePoolFuzzAction::from_repr(fuzzer.next_u8(5u8)).unwrap();
            vec![FuzzAction::OneResourcePool(action)]
        }
    }

    FuzzTest::<OneResourcePoolFuzzer>::run_fuzz(32, 100, false);
}
