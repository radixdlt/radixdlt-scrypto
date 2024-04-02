use radix_common::prelude::*;
use radix_engine_monkey_tests::multi_pool::MultiPoolFuzzAction;
use radix_engine_monkey_tests::{FuzzAction, FuzzTest, SystemTestFuzzer, TxnFuzzer};

#[test]
fn fuzz_multi_pool() {
    struct MultiResourcePoolFuzzer;
    impl TxnFuzzer for MultiResourcePoolFuzzer {
        fn next_txn_intent(fuzzer: &mut SystemTestFuzzer) -> Vec<FuzzAction> {
            let action: MultiPoolFuzzAction =
                MultiPoolFuzzAction::from_repr(fuzzer.next_u8(5u8)).unwrap();
            vec![FuzzAction::MultiResourcePool(action)]
        }
    }

    FuzzTest::<MultiResourcePoolFuzzer>::run_fuzz(32, 100, false);
}
