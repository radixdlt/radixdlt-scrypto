use radix_engine_interface::api::Invokable;
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
                .invoke(ScryptoInvocation {
                    package_address: Runtime::package_address(),
                    blueprint_name: "ReentrantComponent".to_string(),
                    fn_name: "mut_func".to_string(),
                    receiver: Some(ScryptoReceiver::Global(address)),
                    args: args!(),
                })
                .unwrap();
        }

        pub fn call_self(&self, address: ComponentAddress) {
            ScryptoEnv
                .invoke(ScryptoInvocation {
                    package_address: Runtime::package_address(),
                    blueprint_name: "ReentrantComponent".to_string(),
                    fn_name: "func".to_string(),
                    receiver: Some(ScryptoReceiver::Global(address)),
                    args: args!(),
                })
                .unwrap();
        }

        pub fn call_mut_self_2(&self, address: ComponentAddress) {
            ScryptoEnv
                .invoke(ScryptoInvocation {
                    package_address: Runtime::package_address(),
                    blueprint_name: "ReentrantComponent".to_string(),
                    fn_name: "mut_func".to_string(),
                    receiver: Some(ScryptoReceiver::Global(address)),
                    args: args!(),
                })
                .unwrap();
        }
    }
}
