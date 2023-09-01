use radix_engine::blueprints::consensus_manager::EpochChangeEvent;
use radix_engine::transaction::TransactionReceipt;
use radix_engine::types::*;
use rayon::iter::IntoParallelIterator;
use rayon::iter::ParallelIterator;
use resource_tests::validator::{ValidatorFuzzAction, ValidatorMeta};
use resource_tests::{FuzzAction, FuzzTest, FuzzTxnResult, TestFuzzer, TxnFuzzer};
use scrypto_unit::*;
use transaction::prelude::*;

#[test]
fn fuzz_validator() {
    struct ValidatorFuzzer;
    impl TxnFuzzer for ValidatorFuzzer {
        fn next_txn_intent(fuzzer: &mut TestFuzzer) -> Vec<FuzzAction> {
            let action: ValidatorFuzzAction =
                ValidatorFuzzAction::from_repr(fuzzer.next_u8(7u8)).unwrap();
            vec![FuzzAction::Validator(action)]
        }
    }

    FuzzTest::<ValidatorFuzzer>::run_fuzz();
}
