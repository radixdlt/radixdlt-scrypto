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
                vault: Vault::new(XRD),
            }
            .instantiate()
            .prepare_to_globalize(OwnerRole::Fixed(rule!(allow_all)))
            .enable_component_royalties(component_royalties! {
                roles {
                    royalty_setter => rule!(allow_all);
                    royalty_setter_updater => rule!(deny_all);
                    royalty_locker => rule!(allow_all);
                    royalty_locker_updater => rule!(deny_all);
                    royalty_claimer => rule!(allow_all);
                    royalty_claimer_updater => rule!(deny_all);
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
