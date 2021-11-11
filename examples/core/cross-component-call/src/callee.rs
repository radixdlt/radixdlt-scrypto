use scrypto::prelude::*;

blueprint! {
    struct Airdrop {
        tokens: Vault,
    }

    impl Airdrop {
        pub fn new() -> Component {
            Self {
                tokens: Vault::with_bucket(ResourceBuilder::new().new_token_fixed(1000)),
            }
            .instantiate()
        }

        pub fn free_token(&self) -> Bucket {
            self.tokens.take(1)
        }
    }
}
