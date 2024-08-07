use scrypto::prelude::*;

#[blueprint]
mod stored_resource {
    struct StoredResource {
        resource_manager: ResourceManager,
    }

    impl StoredResource {
        pub fn create() -> Global<StoredResource> {
            let resource_manager = ResourceBuilder::new_fungible(OwnerRole::None)
                .create_with_no_initial_supply()
                .into();
            Self { resource_manager }
                .instantiate()
                .prepare_to_globalize(OwnerRole::None)
                .globalize()
        }

        pub fn total_supply(&self) -> Decimal {
            self.resource_manager.total_supply().unwrap()
        }
    }
}
