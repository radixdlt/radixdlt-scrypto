use scrypto::blueprint;
use scrypto::core::Context;
use scrypto::resource::Bucket;
use scrypto::rust::collections::*;
use scrypto::types::{Address, Amount};

use crate::utils::*;

blueprint! {
    struct ResourceTest;

    impl ResourceTest {
        pub fn create_mutable() -> Bucket {
           let resource_def = create_mutable("r1", Context::package_address());
           resource_def.mint(100)
        }

        pub fn create_fixed() -> Bucket {
           create_fixed("r2", 100.into())
        }

        pub fn query() -> (HashMap<String, String>, Option<Address>, Amount) {
            let resource_def = create_mutable("r3", Context::package_address());
            (resource_def.metadata(), resource_def.minter(), resource_def.supply())
        }

        pub fn burn() {
           create_fixed("r4", 100.into()).burn();
        }
    }
}
