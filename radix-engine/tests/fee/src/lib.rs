use scrypto::prelude::*;

blueprint! {
    struct Fee {
        xrd: Vault,
        xrd_empty: Vault,
        doge: Vault,
        garbage_vaults: Vec<Vault>,
    }

    impl Fee {
        pub fn new(xrd: Bucket) -> ComponentAddress {
            let doge_tokens = ResourceBuilder::new_fungible()
                .metadata("name", "DogeCoin")
                .initial_supply(100);

            Self {
                xrd: Vault::with_bucket(xrd),
                xrd_empty: Vault::new(RADIX_TOKEN),
                doge: Vault::with_bucket(doge_tokens),
                garbage_vaults: Vec::new(),
            }
            .instantiate()
            .globalize()
        }

        pub fn pay_fee(&mut self, amount: Decimal) {
            self.xrd.pay_fee(amount);
        }

        pub fn pay_fee_with_empty_vault(&mut self, amount: Decimal) {
            self.xrd_empty.pay_fee(amount);
        }

        pub fn pay_fee_with_doge(&mut self, amount: Decimal) {
            self.doge.pay_fee(amount);
        }

        pub fn pay_fee_with_temp_vault(&mut self, amount: Decimal) {
            let mut vault = Vault::with_bucket(self.xrd.take(amount));
            vault.pay_fee(amount);
            self.garbage_vaults.push(vault);
        }

        pub fn query_vault_and_pay_fee(&mut self, amount: Decimal) {
            info!("Balance: {}", self.xrd.amount());
            self.xrd.pay_fee(amount);
        }

        pub fn pay_fee_and_query_vault(&mut self, amount: Decimal) {
            self.xrd.pay_fee(amount);
            info!("Balance: {}", self.xrd.amount());
        }
    }
}
