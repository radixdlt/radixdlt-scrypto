use scrypto::engine::scrypto_env::*;
use scrypto::engine_lib::engine::types::*;
use scrypto::engine_lib::engine::wasm_input::*;
use scrypto::prelude::*;

blueprint! {
    struct SystemTest;

    impl SystemTest {
        pub fn get_epoch() -> u64 {
            Runtime::current_epoch()
        }

        pub fn set_epoch(epoch_manager: SystemAddress, epoch: u64) {
            let input = RadixEngineInput::InvokeNativeFn(NativeFnInvocation::Method(
                NativeMethodInvocation::EpochManager(EpochManagerMethodInvocation::SetEpoch(
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
