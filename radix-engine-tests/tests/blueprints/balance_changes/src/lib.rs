use scrypto::prelude::*;

#[blueprint]
mod balance_changes_test {
    struct BalanceChangesTest {
        vault: Vault,
    }

    impl BalanceChangesTest {
        pub fn instantiate() -> Global<BalanceChangesTest> {
            Self {
                vault: Vault::new(RADIX_TOKEN),
            }
            .instantiate()
            .prepare_to_globalize(OwnerRole::Fixed(rule!(allow_all)))
            .royalties(royalties! {
                init {
                    put => Xrd(1.into());
                    boom => Xrd(1.into());
                },
                permissions {
                    claim_royalty => [OWNER_ROLE];
                    set_royalty_config => [];
                }
            })
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
