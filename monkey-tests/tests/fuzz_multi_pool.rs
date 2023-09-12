use monkey_tests::multi_pool::MultiPoolFuzzAction;
use monkey_tests::{FuzzAction, FuzzTest, SystemTestFuzzer, TxnFuzzer};
use radix_engine::types::*;

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

    FuzzTest::<MultiResourcePoolFuzzer>::run_fuzz(32, 100);
}
