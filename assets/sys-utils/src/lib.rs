use scrypto::prelude::*;

blueprint! {
    struct SysUtils {}

    impl SysUtils {
        /// Creates a resource.
        ///
        /// TODO: Remove if manifest natively supports this
        pub fn new_resource(
            resource_type: ResourceType,
            metadata: HashMap<String, String>,
            access_rules: HashMap<ResourceMethodAuthKey, (AccessRule, Mutability)>,
            initial_supply: Option<MintParams>,
        ) -> (ResourceAddress, Option<Bucket>) {
            resource_system().new_resource(resource_type, metadata, access_rules, initial_supply)
        }

        /// Mints fungible resource.
        ///
        /// TODO: Remove if manifest natively supports this
        pub fn mint(amount: Decimal, resource_address: ResourceAddress) -> Bucket {
            borrow_resource_manager!(resource_address).mint(amount)
        }

        /// Burns bucket.
        ///
        /// TODO: Remove if manifest natively supports this
        pub fn burn(bucket: Bucket) {
            bucket.burn()
        }
    }
}
