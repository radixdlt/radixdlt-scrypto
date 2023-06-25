use scrypto::prelude::*;

#[blueprint]
mod balance_changes_test {
    enable_package_royalties! {
        instantiate => Free;
        put => Xrd(2.into());
        boom => Xrd(2.into());
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
            .enable_component_royalties(component_royalties! {
                roles {
                    royalty_admin => rule!(allow_all), locked;
                    royalty_admin_updater => rule!(deny_all), locked;
                },
                init {
                    put => Xrd(1.into()), locked;
                    boom => Xrd(1.into()), locked;
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
