use scrypto::prelude::*;

blueprint! {
    // nobody can instantiate a system component except the bootstrap process
    struct System {
        xrd: Vault,
    }

    impl System {
        /// Publishes a package.
        pub fn publish_package(code: Vec<u8>) -> PackageId {
            component_system().publish_package(&code)
        }

        /// Creates a resource.
        pub fn new_resource(
            resource_type: ResourceType,
            metadata: HashMap<String, String>,
            authorization: ComponentAuthorization,
            initial_supply: Option<MintParams>,
        ) -> (ResourceDefId, Option<Bucket>) {
            resource_system().instantiate_resource_definition(
                resource_type,
                metadata,
                authorization,
                initial_supply,
            )
        }

        /// Mints fungible resource. TODO: Remove
        pub fn mint(amount: Decimal, resource_def_id: ResourceDefId) -> Bucket {
            resource_def!(resource_def_id).mint(amount)
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
