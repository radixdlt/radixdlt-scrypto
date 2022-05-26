use scrypto::prelude::*;

blueprint! {
    // nobody can instantiate a system component except the bootstrap process
    struct System {
        xrd: Vault,
    }

    impl System {
        /// Publishes a package.
        pub fn publish_package(code: Vec<u8>, blueprints: Vec<String>) -> PackageAddress {
            let package = Package::new(code, blueprints);
            component_system().publish_package(package)
        }

        /// Creates a resource.
        pub fn new_resource(
            resource_type: ResourceType,
            metadata: HashMap<String, String>,
            access_rules: HashMap<ResourceMethodAuthKey, (AccessRule, Mutability)>,
            initial_supply: Option<MintParams>,
        ) -> (ResourceAddress, Option<Bucket>) {
            resource_system().new_resource(resource_type, metadata, access_rules, initial_supply)
        }

        /// Mints fungible resource. TODO: Remove
        pub fn mint(amount: Decimal, resource_address: ResourceAddress) -> Bucket {
            borrow_resource_manager!(resource_address).mint(amount)
        }

        /// Burns bucket. TODO: Remove
        pub fn burn(bucket: Bucket) {
            bucket.burn()
        }

        /// Gives away XRD tokens for testing. TODO: Remove
        pub fn free_xrd(&mut self) -> Bucket {
            self.xrd.take(1_000_000)
        }
    }
}
