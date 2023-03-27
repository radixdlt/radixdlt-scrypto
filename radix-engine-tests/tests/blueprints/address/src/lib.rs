use scrypto::prelude::*;

#[blueprint]
mod address {
    struct Address {}

    impl Address {
        pub fn create() -> ComponentAddress {
            Self {}.instantiate().globalize()
        }

        pub fn get_address(&self) -> ComponentAddress {
            let address = Runtime::get_global_address();
            address.into()
        }
    }
}
