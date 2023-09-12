use radix_engine::types::*;
use resource_tests::validator::ValidatorFuzzAction;
use resource_tests::{FuzzAction, FuzzTest, SystemTestFuzzer, TxnFuzzer};

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

    FuzzTest::<ValidatorFuzzer>::run_fuzz(32, 100);
}
