use scrypto::prelude::*;

#[blueprint]
mod resource_creator {
    struct ResourceCreator {}

    impl ResourceCreator {
        pub fn set_recallable(resource_address: ResourceAddress, auth_address: ResourceAddress) {
            borrow_resource_manager!(resource_address).set_recallable(rule!(require(auth_address)));
        }

        pub fn set_mintable(resource_address: ResourceAddress, auth_address: ResourceAddress) {
            borrow_resource_manager!(resource_address).set_mintable(rule!(require(auth_address)));
        }

        pub fn set_burnable(resource_address: ResourceAddress, auth_address: ResourceAddress) {
            borrow_resource_manager!(resource_address).set_burnable(rule!(require(auth_address)));
        }

        pub fn set_withdrawable(resource_address: ResourceAddress, auth_address: ResourceAddress) {
            borrow_resource_manager!(resource_address)
                .set_withdrawable(rule!(require(auth_address)));
        }

        pub fn set_depositable(resource_address: ResourceAddress, auth_address: ResourceAddress) {
            borrow_resource_manager!(resource_address)
                .set_depositable(rule!(require(auth_address)));
        }

        pub fn set_updateable_metadata(
            resource_address: ResourceAddress,
            auth_address: ResourceAddress,
        ) {
            borrow_resource_manager!(resource_address)
                .set_updateable_metadata(rule!(require(auth_address)));
        }

        pub fn lock_recallable(resource_address: ResourceAddress) {
            borrow_resource_manager!(resource_address).lock_recallable();
        }

        pub fn lock_mintable(resource_address: ResourceAddress) {
            borrow_resource_manager!(resource_address).lock_mintable();
        }

        pub fn lock_burnable(resource_address: ResourceAddress) {
            borrow_resource_manager!(resource_address).lock_burnable();
        }

        pub fn lock_withdrawable(resource_address: ResourceAddress) {
            borrow_resource_manager!(resource_address).lock_withdrawable();
        }

        pub fn lock_depositable(resource_address: ResourceAddress) {
            borrow_resource_manager!(resource_address).lock_depositable();
        }

        pub fn lock_metadata_updateable(resource_address: ResourceAddress) {
            borrow_resource_manager!(resource_address).lock_updateable_metadata();
        }
    }
}
