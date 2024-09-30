use scrypto::prelude::*;

#[blueprint]
mod identity_test {
    struct IdentityTest {}

    impl IdentityTest {
        pub fn accept_address(address: ComponentAddress) {}
    }
}
