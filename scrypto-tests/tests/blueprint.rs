#![cfg_attr(not(feature = "std"), no_std)]

use scrypto::prelude::*;

#[blueprint]
mod blueprint {
    struct Empty {}

    impl Empty {}
}

#[blueprint]
mod blueprint {
    struct Simple {
        state: u32,
    }

    impl Simple {
        pub fn new() -> ComponentAddress {
            Self { state: 0 }.instantiate().globalize()
        }

        pub fn get_state(&self) -> u32 {
            self.state
        }

        pub fn set_state(&mut self, new_state: u32) {
            self.state = new_state;
        }

        pub fn custom_types() -> (
            Decimal,
            PackageAddress,
            KeyValueStore<String, String>,
            Hash,
            Bucket,
            Proof,
            Vault,
        ) {
            todo!()
        }
    }
}
