use radix_engine::blueprints::pool::two_resource_pool::TWO_RESOURCE_POOL_BLUEPRINT_IDENT;
use radix_engine::types::*;
use radix_engine_interface::blueprints::pool::*;
use rayon::iter::IntoParallelIterator;
use rayon::iter::ParallelIterator;
use resource_tests::two_pool::TwoPoolFuzzAction;
use resource_tests::{FuzzAction, FuzzTest, FuzzTxnResult, TestFuzzer, TxnFuzzer};
use scrypto_unit::*;

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

    FuzzTest::<TwoResourcePoolFuzzer>::run_fuzz();
}
