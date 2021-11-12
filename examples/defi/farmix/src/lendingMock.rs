use scrypto::prelude::*;

blueprint! {
    struct Lending {
        usdc_vault: Vault,
    }

    impl Lending {
        pub fn new() -> Component {
            Self {
                usdc_vault: Vault::with_bucket(
                  ResourceBuilder::new()
                    .metadata("symbol", "usdc")
                    .create_fixed(10000000)
                ),
            }
            .instantiate()
        }

        pub fn get_collateralization_ratio(&mut self) -> u8 {
          return 2
        }

        pub fn borrow(&mut self, amount: u8, collateral: Bucket) -> Bucket {
            return self.usdc_vault.take(amount)
        }
    }
}