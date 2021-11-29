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

        /// Creates a resource.
        pub fn new_resource(
            resource_type: ResourceType,
            metadata: HashMap<String, String>,
            flags: u16,
            mutable_flags: u16,
            authorities: HashMap<Address, u16>,
            initial_supply: Option<NewSupply>,
        ) -> (ResourceDef, Option<Bucket>) {
            ResourceDef::new(
                resource_type,
                metadata,
                flags,
                mutable_flags,
                authorities,
                initial_supply,
            )
        }

        /// Mints fungible resource.
        pub fn mint(amount: Decimal, resource_address: Address, auth: BucketRef) -> Bucket {
            ResourceDef::from(resource_address).mint(amount, auth)
        }

        /// Gives away XRD tokens for testing.
        pub fn free_xrd(&self, amount: Decimal) -> Bucket {
            self.xrd.take(amount, None)
        }
    }
}
