use radix_engine::types::*;
use resource_tests::one_pool::OnePoolFuzzAction;
use resource_tests::{FuzzAction, FuzzTest, TestFuzzer, TxnFuzzer};

#[test]
fn fuzz_one_pool() {
    struct OneResourcePoolFuzzer;
    impl TxnFuzzer for OneResourcePoolFuzzer {
        fn next_txn_intent(fuzzer: &mut TestFuzzer) -> Vec<FuzzAction> {
            let action: OnePoolFuzzAction =
                OnePoolFuzzAction::from_repr(fuzzer.next_u8(5u8)).unwrap();
            vec![FuzzAction::OneResourcePool(action)]
        }
    }

    FuzzTest::<OneResourcePoolFuzzer>::run_fuzz(32, 100);
}
