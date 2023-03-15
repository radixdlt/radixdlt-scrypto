use scrypto::prelude::*;

#[blueprint]
mod reference_test {
    struct ReferenceTest {
        reference: InternalRef,
    }

    impl ReferenceTest {
        pub fn new() -> ComponentAddress {
            Self {
                reference: InternalRef([0u8; 31]),
            }
            .instantiate()
            .globalize()
        }
    }
}
