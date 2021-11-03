use scrypto::prelude::*;

use crate::callee::Airdrop;

blueprint! {
    struct Proxy2 {
        airdrop: Airdrop
    }

    impl Proxy2 {
        pub fn new() -> Component {
            Self {
                airdrop: Airdrop::new().into()
            }
            .instantiate()
        }

        pub fn free_token(&self) -> Bucket {
            self.airdrop.free_token()
        }
    }
}
