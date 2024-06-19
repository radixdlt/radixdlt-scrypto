use scrypto::prelude::*;

#[blueprint]
mod steal_child {

    struct StealChild {
        vault: Vault,
    }

    impl StealChild {
        pub fn child_create() -> Owned<StealChild> {
            Self {
                vault: Vault::new(XRD),
            }
            .instantiate()
        }

        pub fn child_steal_from_account(&mut self, address: ComponentAddress) -> Bucket {
            let mut account: Global<Account> =
                Global(Account::new(ObjectStubHandle::Global(address.into())));

            let bucket = account.withdraw(XRD, dec!(100));
            bucket
        }

        pub fn child_get_balance(&self) -> Decimal {
            self.vault.amount()
        }
    }
}

#[blueprint]
mod steal {
    use steal_child::*;

    struct Steal {
        vault: Vault,
    }

    impl Steal {
        pub fn instantiate() -> Global<Steal> {
            Self {
                vault: Vault::new(XRD),
            }
            .instantiate()
            .prepare_to_globalize(OwnerRole::None)
            .globalize()
        }

        pub fn steal_from_account(&mut self, address: ComponentAddress) {
            let child_component = StealChild::child_create();
            let bucket = child_component.child_steal_from_account(address);
            self.vault.put(bucket);
            child_component
                .prepare_to_globalize(OwnerRole::None)
                .globalize();
        }

        pub fn get_balance(&self) -> Decimal {
            self.vault.amount()
        }
    }
}
