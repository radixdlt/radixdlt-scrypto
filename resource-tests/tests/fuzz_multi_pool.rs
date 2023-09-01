use radix_engine::types::*;
use resource_tests::multi_pool::MultiPoolFuzzAction;
use resource_tests::{FuzzAction, FuzzTest, TestFuzzer, TxnFuzzer};

#[test]
fn fuzz_multi_pool() {
    struct MultiResourcePoolFuzzer;
    impl TxnFuzzer for MultiResourcePoolFuzzer {
        fn next_txn_intent(fuzzer: &mut TestFuzzer) -> Vec<FuzzAction> {
            let action: MultiPoolFuzzAction =
                MultiPoolFuzzAction::from_repr(fuzzer.next_u8(5u8)).unwrap();
            vec![FuzzAction::MultiResourcePool(action)]
        }
    }

    FuzzTest::<MultiResourcePoolFuzzer>::run_fuzz();
}
