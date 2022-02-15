use scrypto::prelude::*;

use crate::airdrop::Airdrop;

blueprint! {
    struct Proxy2 {
        airdrop: Airdrop,
    }

    impl Proxy2 {
        pub fn new() -> ComponentRef {
            Self {
                // The new() function returns a generic Component. We use `.into()` to convert it into an `Airdrop`.
                airdrop: Airdrop::new().into(),
            }
            .instantiate()
        }

        pub fn free_token(&self) -> Bucket {
            // Calling a method on a component using `.method_name()`.
            self.airdrop.free_token()
        }
    }
}
