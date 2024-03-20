use radix_common::prelude::*;
use radix_engine_monkey_tests::validator::ValidatorFuzzAction;
use radix_engine_monkey_tests::{FuzzAction, FuzzTest, SystemTestFuzzer, TxnFuzzer};

#[test]
fn fuzz_validator() {
    struct ValidatorFuzzer;
    impl TxnFuzzer for ValidatorFuzzer {
        fn next_txn_intent(fuzzer: &mut SystemTestFuzzer) -> Vec<FuzzAction> {
            let action: ValidatorFuzzAction =
                ValidatorFuzzAction::from_repr(fuzzer.next_u8(7u8)).unwrap();
            vec![FuzzAction::Validator(action)]
        }
    }

    FuzzTest::<ValidatorFuzzer>::run_fuzz(32, 100, false);
}
