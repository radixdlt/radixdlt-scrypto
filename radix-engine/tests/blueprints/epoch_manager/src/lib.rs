use radix_engine_interface::api::Invokable;
use scrypto::engine::scrypto_env::*;
use scrypto::prelude::*;

#[blueprint]
mod epoch_manager_test {
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
