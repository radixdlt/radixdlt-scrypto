use scrypto::api::*;
use scrypto::blueprints::epoch_manager::*;
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
                .call_method(
                    &epoch_manager.into(),
                    EPOCH_MANAGER_NEXT_ROUND_IDENT,
                    scrypto_encode(&EpochManagerNextRoundInput { round }).unwrap(),
                )
                .unwrap();
        }
    }
}
