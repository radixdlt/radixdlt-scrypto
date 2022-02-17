use scrypto::prelude::*;

blueprint! {
    // nobody can instantiate a system component except the bootstrap process
    struct System {
        xrd: Vault,
    }

    impl System {
        /// Publishes a package.
        pub fn publish_package(code: Vec<u8>) -> PackageId {
            Context::publish_package(&code)
        }

        /// Creates a resource.
        pub fn new_resource(
            resource_type: ResourceType,
            metadata: HashMap<String, String>,
            flags: u64,
            mutable_flags: u64,
            authorities: HashMap<ResourceDefId, u64>,
            initial_supply: Option<Supply>,
        ) -> (ResourceDefId, Option<Bucket>) {
            Context::create_resource(
                resource_type,
                metadata,
                flags,
                mutable_flags,
                authorities,
                initial_supply,
            )
        }

        /// Mints fungible resource.
        pub fn mint(amount: Decimal, mut resource_def_id: ResourceDefId, auth: Proof) -> Bucket {
            resource_def_id.mint(amount, auth)
        }

        /// Gives away XRD tokens for testing.
        pub fn free_xrd(&mut self) -> Bucket {
            self.xrd.take(1_000_000)
        }
    }
}
