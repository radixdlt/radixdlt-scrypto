use scrypto::api::*;
use scrypto::blueprints::consensus_manager::*;
use scrypto::engine::scrypto_env::*;
use scrypto::prelude::*;

#[blueprint]
mod consensus_manager_test {
    struct ConsensusManagerTest;

    impl ConsensusManagerTest {
        pub fn get_epoch() -> u64 {
            Runtime::current_epoch()
        }

        pub fn next_round(consensus_manager: ComponentAddress, round: u64) {
            ScryptoEnv
                .call_method(
                    &consensus_manager.into(),
                    CONSENSUS_MANAGER_NEXT_ROUND_IDENT,
                    scrypto_encode(&ConsensusManagerNextRoundInput::successful(round, 0)).unwrap(),
                )
                .unwrap();
        }
    }
}
