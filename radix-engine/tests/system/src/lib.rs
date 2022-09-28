use scrypto::engine::{api::*, call_engine, types::*};
use scrypto::prelude::*;

blueprint! {
    struct SystemTest;

    impl SystemTest {
        pub fn get_epoch() -> u64 {
            Runtime::current_epoch()
        }

        pub fn set_epoch(epoch: u64) {
            let input = RadixEngineInput::InvokeMethod(MethodIdent {
                receiver: Receiver::Ref(RENodeId::System(SYS_SYSTEM_COMPONENT)),
                fn_ident: FunctionIdent::Native(NativeFnIdentifier::System(SystemFnIdentifier::SetEpoch)),
            },
                scrypto_encode(&SystemSetEpochInput { epoch }),
            );
            call_engine(input)
        }
    }
}
