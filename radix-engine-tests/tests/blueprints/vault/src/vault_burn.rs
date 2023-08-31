use scrypto::prelude::*;

#[blueprint]
mod vault_burn {
    struct VaultBurn {
        vault: Vault,
    }

    impl VaultBurn {
        pub fn new(bucket: Bucket) -> Global<VaultBurn> {
            Self {
                vault: Vault::with_bucket(bucket),
            }
            .instantiate()
            .prepare_to_globalize(OwnerRole::None)
            .globalize()
        }

        pub fn vault_id(&self) -> NodeId {
            self.vault.0 .0
        }

        pub fn burn_amount(&mut self, amount: Decimal) {
            self.vault.burn(amount)
        }

        pub fn burn_ids(&mut self, ids: IndexSet<NonFungibleLocalId>) {
            self.vault.as_non_fungible().burn_non_fungibles(&ids)
        }

        pub fn take_amount(&mut self, amount: Decimal) -> Bucket {
            self.vault.as_fungible().take(amount).0
        }

        pub fn take_ids(&mut self, ids: IndexSet<NonFungibleLocalId>) -> Bucket {
            self.vault.as_non_fungible().take_non_fungibles(&ids).0
        }
    }
}
