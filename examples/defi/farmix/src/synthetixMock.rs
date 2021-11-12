use scrypto::prelude::*;

blueprint! {
    struct Synthetix {
        synthetic_asset_vault: Vault,
    }

    impl Synthetix {
        pub fn new() -> Component {
            Self {
                synthetic_asset_vault: Vault::with_bucket(
                  ResourceBuilder::new()
                    .metadata("symbol", "TSLA")
                    .create_fixed(10000000)
                ),
            }
            .instantiate()
        }

        pub fn mint(&mut self, collateral: Bucket) -> Bucket {
            return self.synthetic_asset_vault.take(10);
        }
    }
}