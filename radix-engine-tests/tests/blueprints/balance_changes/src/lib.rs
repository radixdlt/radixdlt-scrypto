use scrypto::prelude::*;

#[blueprint]
mod balance_changes_test {
    enable_package_royalties! {
        instantiate => Free,
        put => Xrd(2.into()),
        boom => Xrd(2.into()),
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
                roles {
                    royalty_admin => rule!(allow_all);
                },
                init {
                    put => Xrd(1.into()),
                    boom => Xrd(1.into()),
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
