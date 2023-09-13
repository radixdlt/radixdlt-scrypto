use monkey_tests::one_pool::OnePoolFuzzAction;
use monkey_tests::{FuzzAction, FuzzTest, SystemTestFuzzer, TxnFuzzer};
use radix_engine::types::*;

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

    FuzzTest::<OneResourcePoolFuzzer>::run_fuzz(32, 100);
}
