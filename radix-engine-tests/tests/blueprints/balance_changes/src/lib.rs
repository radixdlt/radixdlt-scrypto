use scrypto::prelude::*;

#[blueprint]
mod balance_changes_test {
    struct BalanceChangesTest {
        vault: Vault,
    }

    impl BalanceChangesTest {
        pub fn instantiate() -> Global<BalanceChangesTest> {
            let mut local_component = Self {
                vault: Vault::new(RADIX_TOKEN),
            }
            .instantiate();

            local_component
                .set_royalty("put", 1)
                .set_royalty("boom", 1)
                .set_royalty_default(0)
                .set_authority_rule("owner", rule!(allow_all), rule!(allow_all))
                .globalize()
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
