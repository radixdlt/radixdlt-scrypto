use monkey_tests::access_controller::{AccessControllerFuzzAction, ProofFromAccountAction};
use monkey_tests::{FuzzAction, FuzzTest, SystemTestFuzzer, TxnFuzzer};

#[test]
fn fuzz_access_controller() {
    struct AccessControllerFuzzer;
    impl TxnFuzzer for AccessControllerFuzzer {
        fn next_txn_intent(fuzzer: &mut SystemTestFuzzer) -> Vec<FuzzAction> {
            let mut actions = vec![];
            for _ in 0..=fuzzer.next_u8(3u8) {
                actions.push(FuzzAction::ProofFromAccount(
                    ProofFromAccountAction::CreateProofOfAmount,
                ));
            }
            let action: AccessControllerFuzzAction =
                AccessControllerFuzzAction::from_repr(fuzzer.next_u8(2u8)).unwrap();
            actions.push(FuzzAction::AccessController(action));
            actions
        }
    }

    FuzzTest::<AccessControllerFuzzer>::run_fuzz(32, 1000, false);
}
