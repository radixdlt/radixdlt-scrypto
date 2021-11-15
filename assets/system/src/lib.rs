use scrypto::prelude::*;

blueprint! {
    // nobody can instantiate a system component except the bootstrap process
    struct System {
        xrd: Vault,
    }

    impl System {
        /// Publishes a package.
        pub fn publish_package(code: Vec<u8>) -> Address {
            let package = Package::new(&code);
            package.into()
        }

        /// Creates a resource with mutable supply, and returns the resource definition address.
        pub fn new_resource_mutable(
            resource_type: ResourceType,
            metadata: HashMap<String, String>,
            minter: Address,
        ) -> Address {
            let resource_def = ResourceDef::new_mutable(resource_type, metadata, minter);
            resource_def.address()
        }

        /// Creates a resource with fixed supply, and returns all supply.
        pub fn new_resource_fixed(
            resource_type: ResourceType,
            metadata: HashMap<String, String>,
            supply: ResourceSupply,
        ) -> (Address, Bucket) {
            let (resource_def, bucket) = ResourceDef::new_fixed(resource_type, metadata, supply);
            (resource_def.address(), bucket)
        }

        /// Mints resource.
        pub fn mint_resource(
            resource_def: Address,
            supply: ResourceSupply,
            auth: BucketRef,
        ) -> Bucket {
            ResourceDef::from(resource_def).mint(supply, auth)
        }

        /// Gives away XRD tokens for testing.
        pub fn free_xrd(&self, amount: Decimal) -> Bucket {
            self.xrd.take(amount)
        }
    }
}
