use radix_engine_interface::api::types::*;
use radix_engine_interface::wasm::*;
use scrypto::engine::scrypto_env::*;
use scrypto::prelude::*;

blueprint! {
    struct ReentrantComponent {}

    impl ReentrantComponent {
        pub fn new() -> ComponentAddress {
            Self {}.instantiate().globalize()
        }

        pub fn mut_func(&mut self) {}

        pub fn call_mut_self(&mut self, address: ComponentAddress) {
            let input =
                RadixEngineInput::Invoke(CallTableInvocation::Scrypto(ScryptoInvocation {
                    package_address: Runtime::package_address(),
                    blueprint_name: "ReentrantComponent".to_string(),
                    fn_name: "mut_func".to_string(),
                    receiver: Some(Receiver::Global(address)),
                    args: args!(),
                }));
            let _: ScryptoValue = call_engine(input);
        }

        pub fn func(&self) {}

        pub fn call_self(&self, address: ComponentAddress) {
            let input =
                RadixEngineInput::Invoke(CallTableInvocation::Scrypto(ScryptoInvocation {
                    package_address: Runtime::package_address(),
                    blueprint_name: "ReentrantComponent".to_string(),
                    fn_name: "func".to_string(),
                    receiver: Some(Receiver::Global(address)),
                    args: args!(),
                }));
            let _: ScryptoValue = call_engine(input);
        }

        pub fn call_mut_self_2(&self, address: ComponentAddress) {
            let input =
                RadixEngineInput::Invoke(CallTableInvocation::Scrypto(ScryptoInvocation {
                    package_address: Runtime::package_address(),
                    blueprint_name: "ReentrantComponent".to_string(),
                    fn_name: "mut_func".to_string(),
                    receiver: Some(Receiver::Global(address)),
                    args: args!(),
                }));
            let _: ScryptoValue = call_engine(input);
        }
    }
}
