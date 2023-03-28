#![cfg_attr(not(feature = "std"), no_std)]
#![allow(unused_imports)]

use scrypto::prelude::*;

#[blueprint]
mod empty {
    struct Empty {}

    impl Empty {}
}

#[blueprint]
mod simple {
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

#[blueprint]
mod empty_with_use_statements {
    use radix_engine_common::types::ComponentAddress;

    struct EmptyWithUse {}

    impl EmptyWithUse {}
}

#[blueprint]
mod empty_with_use_super {
    use super::*;

    struct EmptyWithUseSuper {}

    impl EmptyWithUseSuper {}
}
