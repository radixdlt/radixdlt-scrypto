use scrypto::prelude::*;

#[blueprint]
mod balance_changes_test {
    define_permissions! {
        main {
            put => PUBLIC;
            boom => PUBLIC;
        },
        royalties {
            claim_royalty => OWNER;
            set_royalty_config => OWNER;
        }
    }

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
                    put => 1u32;
                    boom => 1u32;
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
