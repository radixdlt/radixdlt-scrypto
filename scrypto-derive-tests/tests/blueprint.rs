#![cfg_attr(not(feature = "std"), no_std)]
#![allow(unused_imports)]

use scrypto::prelude::*;

#[blueprint]
mod empty {
    struct Empty {}

    impl Empty {}
}

#[derive(ScryptoSbor)]
struct Struct1 {
    a: String,
}

#[derive(ScryptoSbor)]
struct Struct2 {
    a: String,
}

pub type Array = [u8; 22];
pub type Tuple = (String, String);

#[blueprint]
#[types(Struct1, Struct2 as Hi, u32, NonFungibleGlobalId, Vec<Hash>, Vec<Bucket> as GenericAlias, scrypto::prelude::NonFungibleLocalId, Array, Tuple)]
mod simple {
    use scrypto::prelude::OwnerRole;

    struct Simple {
        state: u32,
    }

    impl Simple {
        pub fn new() -> Global<Simple> {
            Self { state: 0 }
                .instantiate()
                .prepare_to_globalize(OwnerRole::None)
                .globalize()
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
            unreachable!()
        }
    }
}

#[blueprint]
mod empty_with_use_statements {
    use radix_common::types::ComponentAddress;

    struct EmptyWithUse {}

    impl EmptyWithUse {}
}

#[blueprint]
mod empty_with_use_super {
    use super::*;

    struct EmptyWithUseSuper {}

    impl EmptyWithUseSuper {}
}

#[blueprint]
mod kv_entry_clone {
    use super::*;

    struct KVEntryClone {
        store: KeyValueStore<String, String>,
    }

    impl KVEntryClone {
        pub fn get_and_clone(&self, key: String) -> Option<String> {
            self.store.get(&key).cloned()
        }

        pub fn get_mut_and_clone(&mut self, key: String) -> Option<String> {
            self.store.get_mut(&key).cloned()
        }
    }
}
