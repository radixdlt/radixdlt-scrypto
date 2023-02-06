use scrypto::engine::scrypto_env::*;
use scrypto::prelude::*;

#[blueprint]
mod reentrant_component {
    struct ReentrantComponent {}

    impl ReentrantComponent {
        pub fn new() -> ComponentAddress {
            Self {}.instantiate().globalize()
        }

        pub fn func(&self) {}

        pub fn mut_func(&mut self) {}

        pub fn call_mut_self(&mut self, address: ComponentAddress) {
            ScryptoEnv
                .call_method(ScryptoReceiver::Global(address), "mut_func", args!())
                .unwrap();
        }

        pub fn call_self(&self, address: ComponentAddress) {
            ScryptoEnv
                .call_method(ScryptoReceiver::Global(address), "func", args!())
                .unwrap();
        }

        pub fn call_mut_self_2(&self, address: ComponentAddress) {
            ScryptoEnv
                .call_method(ScryptoReceiver::Global(address), "mut_func", args!())
                .unwrap();
        }
    }
}
