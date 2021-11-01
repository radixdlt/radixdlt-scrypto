use scrypto::prelude::*;

blueprint! {
    // nobody can instantiate a system component except the bootstrap process
    struct System {
        xrd: Vault
    }

    impl System {
        /// Publishes a package.
        pub fn publish_package(code: Vec<u8>) -> Address {
            let package = Package::new(&code);
            package.into()
        }

        /// Creates a resource with mutable supply, and returns the resource definition address.
        pub fn new_resource_mutable(metadata: HashMap<String, String>, mint_burn_auth: Address) -> Address {
            let resource_def = ResourceDef::new_mutable(metadata, mint_burn_auth);
            resource_def.address()
        }

        /// Creates a resource with fixed supply, and returns all supply.
        pub fn new_resource_fixed(metadata: HashMap<String, String>, supply: Amount) -> (Address, Bucket) {
            let (resource_def, bucket) = ResourceDef::new_fixed(metadata, supply);
            (resource_def.address(), bucket)
        }

        /// Mints resource.
        pub fn mint_resource(amount: Amount, resource_def: Address, auth: BucketRef) -> Bucket {
            ResourceDef::from(resource_def).mint(amount, auth)
        }

        /// Gives away XRD tokens for testing.
        pub fn free_xrd(&self, amount: Amount) -> Bucket {
            self.xrd.take(amount)
        }
    }
}
