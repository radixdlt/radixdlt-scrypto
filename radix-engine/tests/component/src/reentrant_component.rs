use scrypto::prelude::*;

blueprint! {
    struct ReentrantComponent {}

    impl ReentrantComponent {
        pub fn new() -> ComponentAddress {
            Self {}
                .instantiate()
                .auth("call_self", auth!(allow_all))
                .auth("func", auth!(allow_all))
                .globalize()
        }

        pub fn func(&mut self) {}

        pub fn call_self(&mut self) {
            if let ScryptoActor::Component(addr) = Runtime::actor().actor() {
                let self_component = component!(addr);
                self_component.call("func", vec![])
            }
        }
    }
}
