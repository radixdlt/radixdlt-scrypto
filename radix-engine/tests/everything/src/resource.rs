use crate::utils::*;
use scrypto::constructs::*;
use scrypto::resource::*;
use scrypto::rust::collections::*;
use scrypto::types::*;
use scrypto::*;

blueprint! {
    struct ResourceTest {
        resource: Address,
        bucket: Bucket,
        secret: String,
    }

    impl ResourceTest {
        pub fn create_mutable() -> Bucket {
           let resource = create_mutable("r1", Context::package_address());
           mint_resource(resource, 100)
        }

        pub fn create_fixed() -> Bucket {
           create_fixed("r2", 100.into())
        }

        pub fn query() -> (HashMap<String, String>, Option<Address>, Amount) {
            let resource = create_mutable("r3", Context::package_address());
            let def = ResourceDef::from(resource);
            (def.metadata(), def.minter(), def.supply())
        }

        pub fn burn() {
           ResourceDef::burn(create_fixed("r4", 100.into()));
        }
    }
}
