use scrypto::prelude::*;

// A blueprint defines the structure and common behaviour of all its instances, called components.
// In this example, we're creating a `Hello` blueprint.  All components instantiated
// from this blueprint will airdrop 1 `HT` token to its caller.

blueprint! {
    /// Every `Hello` component will have a vault, used for storing `HT` tokens.
    struct Hello {
        vault: Vault
    }

    impl Hello {
        /// This function creates 1000 `HT` tokens and a `Hello` component.
        pub fn new() -> Address {
            let bucket: Bucket = ResourceBuilder::new()
                .metadata("name", "HelloToken")
                .metadata("symbol", "HT")
                .create_fixed(1000);

            Self {
                vault: Vault::with_bucket(bucket)
            }
            .instantiate()
        }

        /// This method takes 1 `HT` token from its vault and returns it to the caller.
        pub fn airdrop(&mut self) -> Bucket {
            let bucket: Bucket = self.vault.take(1);
            info!("Balance: {} HT", self.vault.amount());
            bucket
        }
    }
}
