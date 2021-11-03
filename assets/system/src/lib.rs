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

        /// Creates a token resource with mutable supply, and returns the resource definition address.
        pub fn new_token_mutable(metadata: HashMap<String, String>, minter: Address) -> Address {
            let resource_def = ResourceDef::new_mutable(1, metadata, minter);
            resource_def.address()
        }

        /// Creates a token resource with fixed supply, and returns all supply.
        pub fn new_token_fixed(metadata: HashMap<String, String>, supply: Decimal) -> (Address, Bucket) {
            let (resource_def, bucket) = ResourceDef::new_fixed(1, metadata, supply);
            (resource_def.address(), bucket)
        }

        /// Creates a badge resource with mutable supply, and returns the resource definition address.
        pub fn new_badge_mutable(metadata: HashMap<String, String>, minter: Address) -> Address {
            let resource_def = ResourceDef::new_mutable(18, metadata, minter);
            resource_def.address()
        }

        /// Creates a badge resource with fixed supply, and returns all supply.
        pub fn new_badge_fixed(metadata: HashMap<String, String>, supply: Decimal) -> (Address, Bucket) {
            let (resource_def, bucket) = ResourceDef::new_fixed(18, metadata, supply);
            (resource_def.address(), bucket)
        }

        /// Mints resource.
        pub fn mint_resource(amount: Decimal, resource_def: Address, auth: BucketRef) -> Bucket {
            ResourceDef::from(resource_def).mint(amount, auth)
        }

        /// Gives away XRD tokens for testing.
        pub fn free_xrd(&self, amount: Decimal) -> Bucket {
            self.xrd.take(amount)
        }
    }
}
