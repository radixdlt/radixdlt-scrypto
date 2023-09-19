use monkey_tests::{FuzzAction, FuzzTest, SystemTestFuzzer, TxnFuzzer};
use monkey_tests::access_controller::AccessControllerFuzzAction;

#[test]
fn fuzz_access_controller() {
    struct AccessControllerFuzzer;
    impl TxnFuzzer for AccessControllerFuzzer {
        fn next_txn_intent(fuzzer: &mut SystemTestFuzzer) -> Vec<FuzzAction> {
            match fuzzer.next(0u8..1u8) {
                _ => {
                    let action: AccessControllerFuzzAction =
                        AccessControllerFuzzAction::from_repr(fuzzer.next_u8(1u8)).unwrap();
                    vec![FuzzAction::AccessController(action)]
                }
            }
        }
    }

    FuzzTest::<AccessControllerFuzzer>::run_fuzz(16, 100, false);
}
