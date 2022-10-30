use scrypto::prelude::*;

blueprint! {
    struct ResourceCreator {}

    impl ResourceCreator {
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

        pub fn lock_mintable(resource_address: ResourceAddress) {
            borrow_resource_manager!(resource_address).lock_mintable();
        }
    }
}
