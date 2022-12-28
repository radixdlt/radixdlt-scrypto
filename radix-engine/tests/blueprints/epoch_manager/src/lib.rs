use radix_engine_interface::wasm::*;
use scrypto::engine::scrypto_env::*;
use scrypto::prelude::*;

blueprint! {
    struct EpochManagerTest;

    impl EpochManagerTest {
        pub fn get_epoch() -> u64 {
            Runtime::current_epoch()
        }

        pub fn set_epoch(epoch_manager: SystemAddress, epoch: u64) {
            let input = RadixEngineInput::Invoke(SerializedInvocation::Native(
                NativeFnInvocation::EpochManager(EpochManagerMethodInvocation::SetEpoch(
                    EpochManagerSetEpochInvocation {
                        receiver: epoch_manager,
                        epoch,
                    },
                )),
            ));
            call_engine(input)
        }
    }
}
