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

            let access_rules = {
                let mut authority_rules = AuthorityRules::new();
                authority_rules.set_rule("owner", rule!(allow_all), rule!(allow_all));
                AccessRules::new(MethodAuthorities::new(), authority_rules)
            };

            local_component
                .set_royalty("put", 1)
                .set_royalty("boom", 1)
                .set_royalty_default(0)
                .attach_access_rules(access_rules)
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
