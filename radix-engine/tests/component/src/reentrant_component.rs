use scrypto::prelude::*;

blueprint! {
    struct ReentrantComponent {}

    impl ReentrantComponent {
        pub fn new() -> ComponentAddress {
            Self {}.instantiate().globalize()
        }

        pub fn func(&mut self) {}

        pub fn call_self(&mut self) {
            if let ScryptoActor::Component(addr, _) = Runtime::actor() {
                let self_component = borrow_component!(addr);
                self_component.call("func", vec![])
            }
        }
    }
}
