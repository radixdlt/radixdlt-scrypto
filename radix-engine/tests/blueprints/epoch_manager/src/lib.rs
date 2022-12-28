use radix_engine_interface::wasm::*;
use scrypto::engine::scrypto_env::*;
use scrypto::prelude::*;

blueprint! {
    struct EpochManagerTest;

    impl EpochManagerTest {
        pub fn get_epoch() -> u64 {
            Runtime::current_epoch()
        }

        pub fn next_round(epoch_manager: SystemAddress, round: u64) {
            let input = RadixEngineInput::Invoke(SerializedInvocation::Native(
                NativeInvocation::EpochManager(EpochManagerInvocation::NextRound(
                    EpochManagerNextRoundInvocation {
                        receiver: epoch_manager,
                        round,
                    },
                )),
            ));
            call_engine(input)
        }
    }
}
