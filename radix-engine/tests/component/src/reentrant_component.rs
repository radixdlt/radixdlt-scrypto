use scrypto::engine::{api::*, call_engine, types::*};
use scrypto::prelude::*;

blueprint! {
    struct ReentrantComponent {}

    impl ReentrantComponent {
        pub fn new() -> ComponentAddress {
            Self {}.instantiate().globalize()
        }

        pub fn func(&mut self) {}

        pub fn call_self(&mut self) {
            if let ScryptoActor::Component(addr, ..) = Runtime::actor() {
                let input = RadixEngineInput::Invoke(
                    FnIdent::Method(MethodIdent {
                        receiver: Receiver::Ref(RENodeId::Component(addr)),
                        method_fn_ident: MethodFnIdent::Scrypto("func".to_string()),
                    }),
                    args!(),
                );
                call_engine(input)
            }
        }
    }
}
