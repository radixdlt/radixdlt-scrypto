use scrypto::prelude::*;

blueprint! {
    // nobody can instantiate a system component except the bootstrap process
    struct System {
        xrd: Vault,
    }

    impl System {
        /// Publishes a package.
        pub fn publish_package(code: Vec<u8>) -> PackageAddress {
            component_system().publish_package(&code)
        }

        /// Creates a resource.
        pub fn new_resource(
            resource_type: ResourceType,
            metadata: HashMap<String, String>,
            authorization: ComponentAuthorization,
            initial_supply: Option<MintParams>,
        ) -> (ResourceAddress, Option<Bucket>) {
            resource_system().new_resource(
                resource_type,
                metadata,
                authorization,
                initial_supply,
            )
        }

        /// Mints fungible resource. TODO: Remove
        pub fn mint(amount: Decimal, resource_address: ResourceAddress, auth: Bucket) -> (Bucket, Bucket) {
            let minted = auth.authorize(|| resource_manager!(resource_address).mint(amount));
            (minted, auth)
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
