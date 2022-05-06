use scrypto::prelude::*;

use crate::component::ComponentTest;

blueprint! {
    struct TypedCrossComponentCall {
        other: ComponentTest,
    }

    impl TypedCrossComponentCall {
        pub fn new(address: ComponentAddress) -> ComponentAddress {
            Self {
                other: address.into(),
            }
            .instantiate()
            .globalize()
        }
    }
}
