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

        // Try to withdraw from the account.
        // It is assumed that the same account signed the transaction.
        pub fn steal_from_account(&mut self, address: ComponentAddress) {
            // Instantiate owned component and call it's methods while they are still owned.
            // NOTE: This attack concept doesn't work if the child component was loaded from the substate
            // store because it gets a `ReferenceOrigin::Global` instead of `ReferenceOrigin::FrameOwned`
            let child_component = steal_child::StealChild::child_create();

            let bucket = child_component.child_steal_from_account(address);
            self.vault.put(bucket);
            // Globalize to avoid error
            child_component
                .prepare_to_globalize(OwnerRole::None)
                .globalize();
        }

        pub fn get_balance(&self) -> Decimal {
            self.vault.amount()
        }
    }
}
