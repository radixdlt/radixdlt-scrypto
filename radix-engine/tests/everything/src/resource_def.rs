use scrypto::blueprint;
use scrypto::resource::{Bucket, ResourceBuilder, ResourceDef};
use scrypto::rust::collections::*;
use scrypto::types::{Address, Decimal};

blueprint! {
    struct ResourceTest;

    impl ResourceTest {
        pub fn create_mutable() -> (Bucket, ResourceDef) {
            let auth = ResourceBuilder::new()
                .metadata("name", "Auth")
                .new_token_fixed(1);
            let resource_def = ResourceBuilder::new()
                .metadata("name", "TestToken")
                .new_token_mutable(auth.resource_def());
            (auth, resource_def)
        }

        pub fn create_fixed() -> Bucket {
            let bucket = ResourceBuilder::new()
                .metadata("name", "TestToken")
                .new_token_fixed(100);

            bucket
        }

        pub fn query() -> (Bucket, HashMap<String, String>, Option<Address>, Decimal) {
            let bucket = ResourceBuilder::new()
                .metadata("name", "TestToken")
                .new_token_fixed(100);
            let resource_def = bucket.resource_def();
            (bucket, resource_def.metadata(), resource_def.minter(), resource_def.supply())
        }

        pub fn burn() -> Bucket {
            let (auth, resource_def) = Self::create_mutable();
            let bucket = resource_def.mint(1, auth.borrow());
            resource_def.burn(bucket, auth.borrow());
            auth
        }
    }
}
