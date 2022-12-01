use scrypto::prelude::*;

blueprint! {
    struct StoredResource {
        resource_address: ResourceAddress,
    }

    impl StoredResource {
        pub fn create() -> ComponentAddress {
            let resource_address = ResourceBuilder::new_fungible().no_initial_supply_no_owner();
            Self { resource_address }.instantiate().globalize_no_owner()
        }

        pub fn total_supply(&self) -> Decimal {
            borrow_resource_manager!(self.resource_address).total_supply()
        }
    }
}
