use scrypto::prelude::*;

#[blueprint]
mod reentrant_component {
    struct ReentrantComponent {}

    impl ReentrantComponent {
        pub fn new() -> Global<ReentrantComponent> {
            Self {}
                .instantiate()
                .prepare_to_globalize(OwnerRole::None)
                .globalize()
        }

        pub fn func(&self) {}

        pub fn mut_func(&mut self) {}

        pub fn call_mut_self(&mut self, address: ComponentAddress) {
            ScryptoVmV1Api::object_call(&address.into(), "mut_func", scrypto_args!());
        }

        pub fn call_self(&self, address: ComponentAddress) {
            ScryptoVmV1Api::object_call(&address.into(), "func", scrypto_args!());
        }

        pub fn call_mut_self_2(&self, address: ComponentAddress) {
            ScryptoVmV1Api::object_call(&address.into(), "mut_func", scrypto_args!());
        }
    }
}
