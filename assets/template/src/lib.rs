use scrypto::prelude::*;

blueprint! {
    struct Hello {
        vault: Vault
    }

    impl Hello {
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

        pub fn airdrop(&mut self) -> Bucket {
            let bucket: Bucket = self.vault.take(1);
            info!("Balance: {} HT", self.vault.amount());
            bucket
        }
    }
}
