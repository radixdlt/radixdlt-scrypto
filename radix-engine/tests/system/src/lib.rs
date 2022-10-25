use scrypto::engine::{api::*, types::*, utils::*};
use scrypto::prelude::*;

blueprint! {
    struct SystemTest;

    impl SystemTest {
        pub fn get_epoch() -> u64 {
            Runtime::current_epoch()
        }

        pub fn set_epoch(epoch_manager: SystemAddress, epoch: u64) {
            let input = RadixEngineInput::InvokeNativeMethod(
                NativeMethod::EpochManager(EpochManagerMethod::SetEpoch),
                Receiver::Ref(RENodeId::Global(GlobalAddress::System(epoch_manager))),
                scrypto_encode(&EpochManagerSetEpochInput { epoch }),
            );
            call_engine(input)
        }
    }
}
