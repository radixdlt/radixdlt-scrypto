use crate::radix_engine_interface::api::blueprints::epoch_manager::EpochManagerNextRoundInvocation;
use radix_engine_interface::api::Invokable;
use scrypto::engine::scrypto_env::*;
use scrypto::prelude::*;

blueprint! {
    struct EpochManagerTest;

    impl EpochManagerTest {
        pub fn get_epoch() -> u64 {
            Runtime::current_epoch()
        }

        pub fn next_round(epoch_manager: ComponentAddress, round: u64) {
            ScryptoEnv
                .invoke(EpochManagerNextRoundInvocation {
                    receiver: epoch_manager,
                    round,
                })
                .unwrap();
        }
    }
}
