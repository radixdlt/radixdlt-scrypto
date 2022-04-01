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
            flags: u64,
            mutable_flags: u64,
            authorities: HashMap<ResourceAddress, u64>,
            initial_supply: Option<MintParams>,
        ) -> (ResourceAddress, Option<Bucket>) {
            resource_system().instantiate_resource_definition(
                resource_type,
                metadata,
                flags,
                mutable_flags,
                authorities,
                initial_supply,
            )
        }

        /// Mints fungible resource. TODO: Remove
        pub fn mint(amount: Decimal, resource_address: ResourceAddress) -> Bucket {
            resource_def!(resource_address).mint(amount)
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

package_init!(blueprint::System::describe());
