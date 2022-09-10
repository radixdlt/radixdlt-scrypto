use scrypto::prelude::*;

blueprint! {
    struct SysUtils {}

    impl SysUtils {
        /// Mints fungible resource.
        ///
        /// TODO: Remove if manifest natively supports this
        pub fn mint(amount: Decimal, resource_address: ResourceAddress) -> Bucket {
            borrow_resource_manager!(resource_address).mint(amount)
        }
    }
}
