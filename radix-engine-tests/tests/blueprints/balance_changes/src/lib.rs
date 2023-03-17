use scrypto::prelude::*;

#[blueprint]
mod balance_changes_test {
    struct BalanceChangesTest {
        vault: Vault,
    }

    impl BalanceChangesTest {
        pub fn instantiate() -> ComponentAddress {
            let local_component = Self {
                vault: Vault::new(RADIX_TOKEN),
            }
            .instantiate();

            let config = RoyaltyConfigBuilder::new()
                .add_rule("put", 1)
                .add_rule("boom", 1)
                .default(0);

            local_component.globalize_with_royalty_config(config)
        }

        pub fn put(&mut self, bucket: Bucket) {
            self.vault.put(bucket);
        }

        pub fn boom(&mut self, bucket: Bucket) {
            self.vault.put(bucket);
            panic!("Boom!")
        }
    }
}
