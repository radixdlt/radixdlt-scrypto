use scrypto::blueprints::consensus_manager::*;
use scrypto::prelude::*;

#[blueprint]
mod consensus_manager_test {
    struct ConsensusManagerTest;

    impl ConsensusManagerTest {
        pub fn get_epoch() -> Epoch {
            Runtime::current_epoch()
        }

        pub fn next_round(consensus_manager: ComponentAddress, round: Round) {
            ScryptoVmV1Api::object_call(
                &consensus_manager.into(),
                CONSENSUS_MANAGER_NEXT_ROUND_IDENT,
                scrypto_encode(&ConsensusManagerNextRoundInput::successful(
                    round, 0, 240000i64,
                ))
                .unwrap(),
            );
        }
    }
}
