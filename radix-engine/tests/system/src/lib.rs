use scrypto::engine::{types::*, api::*, call_engine};
use scrypto::prelude::*;

blueprint! {
    struct SystemTest;

    impl SystemTest {
        pub fn get_epoch() -> u64 {
            Runtime::current_epoch()
        }

        pub fn set_epoch(epoch: u64) {
            let input = RadixEngineInput::InvokeMethod(
                Receiver::NativeRENodeRef(RENodeId::System),
                "set_epoch".to_string(),
                scrypto_encode(&SystemSetEpochInput { epoch }),
            );
            call_engine(input)
        }
    }
}
