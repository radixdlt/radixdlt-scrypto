use scrypto::prelude::*;

#[blueprint]
mod reference_test {
    struct ReferenceTest {
        reference: Reference,
    }

    impl ReferenceTest {
        pub fn new() -> ComponentAddress {
            Self {
                reference: Reference([0u8; 31]),
            }
            .instantiate()
            .globalize()
        }
    }
}
