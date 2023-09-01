use radix_engine::blueprints::consensus_manager::EpochChangeEvent;
use resource_tests::consensus_manager::ConsensusManagerFuzzAction;
use resource_tests::validator::{ValidatorFuzzAction, ValidatorMeta};
use resource_tests::{FuzzAction, FuzzTest, FuzzTxnResult, TestFuzzer, TxnFuzzer};
use scrypto_unit::*;
use transaction::prelude::*;

#[test]
fn fuzz_consensus() {
    struct ConsensusFuzzer;
    impl TxnFuzzer for ConsensusFuzzer {
        fn next_action(fuzzer: &mut TestFuzzer) -> FuzzAction {
            match fuzzer.next(0u8..10u8) {
                0u8 => {
                    FuzzAction::ConsensusManager(
                        ConsensusManagerFuzzAction::CreateValidator,
                    )
                }
                _ => {
                    let action: ValidatorFuzzAction =
                        ValidatorFuzzAction::from_repr(fuzzer.next_u8(8u8)).unwrap();
                    FuzzAction::Validator(action)
                }
            }
        }
    }

    FuzzTest::<ConsensusFuzzer>::run_fuzz();
}



