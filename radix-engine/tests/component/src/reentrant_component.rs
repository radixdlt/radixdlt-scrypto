use scrypto::engine::{api::*, call_engine, types::*};
use scrypto::prelude::*;

blueprint! {
    struct ReentrantComponent {}

    impl ReentrantComponent {
        pub fn new() -> ComponentAddress {
            Self {}.instantiate().globalize()
        }

        pub fn mut_func(&mut self) {}

        pub fn call_mut_self(&mut self) {
            if let ScryptoActor::Component(component_id, ..) = Runtime::actor() {
                let input = RadixEngineInput::Invoke(
                    FnIdent::Method(ReceiverMethodIdent {
                        receiver: Receiver::Ref(RENodeId::Component(component_id)),
                        method_ident: MethodIdent::Scrypto("mut_func".to_string()),
                    }),
                    args!(),
                );
                call_engine(input)
            }
        }

        pub fn func(&self) {}

        pub fn call_self(&self) {
            if let ScryptoActor::Component(component_id, ..) = Runtime::actor() {
                let input = RadixEngineInput::Invoke(
                    FnIdent::Method(ReceiverMethodIdent {
                        receiver: Receiver::Ref(RENodeId::Component(component_id)),
                        method_ident: MethodIdent::Scrypto("func".to_string()),
                    }),
                    args!(),
                );
                call_engine(input)
            }
        }

        pub fn call_mut_self_2(&self) {
            if let ScryptoActor::Component(component_id, ..) = Runtime::actor() {
                let input = RadixEngineInput::Invoke(
                    FnIdent::Method(ReceiverMethodIdent {
                        receiver: Receiver::Ref(RENodeId::Component(component_id)),
                        method_ident: MethodIdent::Scrypto("mut_func".to_string()),
                    }),
                    args!(),
                );
                call_engine(input)
            }
        }
    }
}
