use radix_engine::types::*;
use rayon::iter::IntoParallelIterator;
use rayon::iter::ParallelIterator;
use resource_tests::one_pool::OnePoolFuzzAction;
use resource_tests::{FuzzAction, FuzzTest, FuzzTxnResult, TestFuzzer, TxnFuzzer};

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

    FuzzTest::<OneResourcePoolFuzzer>::run_fuzz();
}
