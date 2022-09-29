use scrypto::engine::{api::*, call_engine, types::*};
use scrypto::prelude::*;

blueprint! {
    struct SystemTest;

    impl SystemTest {
        pub fn get_epoch() -> u64 {
            Runtime::current_epoch()
        }

        pub fn set_epoch(epoch: u64) {
            let input = RadixEngineInput::Invoke(
                FnIdent::Method(MethodIdent {
                    receiver: Receiver::Ref(RENodeId::System(SYS_SYSTEM_COMPONENT)),
                    method_fn_ident: MethodFnIdent::Native(NativeMethodFnIdent::System(
                        SystemMethodFnIdent::SetEpoch,
                    )),
                }),
                scrypto_encode(&SystemSetEpochInput { epoch }),
            );
            call_engine(input)
        }
    }
}
