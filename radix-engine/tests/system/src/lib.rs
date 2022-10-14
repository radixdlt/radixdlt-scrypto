use scrypto::engine::{api::*, call_engine, types::*};
use scrypto::prelude::*;

blueprint! {
    struct SystemTest;

    impl SystemTest {
        pub fn get_epoch() -> u64 {
            Runtime::current_epoch()
        }

        pub fn set_epoch(component_address: ComponentAddress, epoch: u64) {
            let input = RadixEngineInput::invoke_native(NativeFnIdent::Method(NativeMethodIdent {
                    receiver: Receiver::Ref(RENodeId::Global(GlobalAddress::Component(
                        component_address,
                    ))),
                    method_ident: MethodIdent::Native(NativeMethod::System(SystemMethod::SetEpoch)),
                }),
                scrypto_encode(&SystemSetEpochInput { epoch }),
            );
            call_engine(input)
        }
    }
}
