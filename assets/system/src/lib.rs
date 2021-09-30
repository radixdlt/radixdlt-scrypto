use scrypto::prelude::*;

blueprint! {
    struct System;

    impl System {
        /// Publishes a package.
        pub fn publish_package(code: Vec<u8>) -> Address {
            let package = Package::new(&code);
            package.into()
        }

        /// Creates a resource with mutable supply, and returns the resource definition address.
        pub fn new_resource_mutable(metadata: HashMap<String, String>, minter: Address) -> Address {
            let resource_def = ResourceDef::new_mutable(metadata, minter);
            resource_def.address()
        }

        /// Creates a resource with fixed supply, and returns all supply.
        pub fn new_resource_fixed(metadata: HashMap<String, String>, supply: Amount) -> Bucket {
            ResourceDef::new_fixed(metadata, supply).1
        }

        /// Mints resource
        pub fn mint(amount: Amount, resource_address: Address) -> Bucket {
            ResourceDef::from(resource_address).mint(amount)
        }
    }
}
