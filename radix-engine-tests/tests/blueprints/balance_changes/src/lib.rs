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
            .prepare_to_globalize()
            .define_roles(roles! {
                "owner" => rule!(allow_all);
            })
            .royalties(royalties! {
                init => {
                    put => 1u32;
                    boom => 1u32;
                },
                permissions => {
                    claim_royalty => ["owner"];
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
