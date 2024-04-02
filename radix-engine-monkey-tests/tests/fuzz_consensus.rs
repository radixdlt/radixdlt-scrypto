use radix_engine_monkey_tests::consensus_manager::ConsensusManagerFuzzAction;
use radix_engine_monkey_tests::validator::ValidatorFuzzAction;
use radix_engine_monkey_tests::{FuzzAction, FuzzTest, SystemTestFuzzer, TxnFuzzer};

#[test]
fn fuzz_consensus() {
    struct ConsensusFuzzer;
    impl TxnFuzzer for ConsensusFuzzer {
        fn next_txn_intent(fuzzer: &mut SystemTestFuzzer) -> Vec<FuzzAction> {
            match fuzzer.next(0u8..10u8) {
                0u8 => vec![FuzzAction::ConsensusManager(
                    ConsensusManagerFuzzAction::CreateValidator,
                )],
                _ => {
                    let action: ValidatorFuzzAction =
                        ValidatorFuzzAction::from_repr(fuzzer.next_u8(8u8)).unwrap();
                    vec![FuzzAction::Validator(action)]
                }
            }
        }
    }

    FuzzTest::<ConsensusFuzzer>::run_fuzz(32, 100, false);
}
