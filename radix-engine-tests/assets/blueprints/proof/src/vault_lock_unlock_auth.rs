use scrypto::prelude::*;

#[derive(Debug, PartialEq, Eq, ScryptoSbor, NonFungibleData)]
pub struct Example {
    pub name: String,
    #[mutable]
    pub available: bool,
}

#[blueprint]
mod vault_lock_unlock_auth {
    struct VaultLockUnlockAuth {
        vault: Vault,
    }

    impl VaultLockUnlockAuth {
        pub fn new_fungible() -> Global<VaultLockUnlockAuth> {
            let bucket = ResourceBuilder::new_fungible(OwnerRole::None).mint_initial_supply(100);

            Self {
                vault: Vault::with_bucket(bucket.into()),
            }
            .instantiate()
            .prepare_to_globalize(OwnerRole::None)
            .globalize()
        }

        pub fn call_lock_fungible_amount_directly(&self) {
            ScryptoVmV1Api::object_call(
                self.vault.0.as_node_id(),
                FUNGIBLE_VAULT_LOCK_FUNGIBLE_AMOUNT_IDENT,
                scrypto_args!(Decimal::from(1)),
            );
        }

        pub fn call_unlock_fungible_amount_directly(&self) {
            let _proof = self.vault.as_fungible().create_proof_of_amount(dec!(1));

            ScryptoVmV1Api::object_call(
                self.vault.0.as_node_id(),
                FUNGIBLE_VAULT_UNLOCK_FUNGIBLE_AMOUNT_IDENT,
                scrypto_args!(Decimal::from(1)),
            );
        }

        pub fn new_non_fungible() -> Global<VaultLockUnlockAuth> {
            let bucket = ResourceBuilder::new_integer_non_fungible(OwnerRole::None)
                .mint_initial_supply([(
                    1u64.into(),
                    Example {
                        name: "One".to_owned(),
                        available: true,
                    },
                )]);

            Self {
                vault: Vault::with_bucket(bucket.into()),
            }
            .instantiate()
            .prepare_to_globalize(OwnerRole::None)
            .globalize()
        }

        pub fn call_lock_non_fungibles_directly(&self) {
            ScryptoVmV1Api::object_call(
                self.vault.0.as_node_id(),
                NON_FUNGIBLE_VAULT_LOCK_NON_FUNGIBLES_IDENT,
                scrypto_args!([NonFungibleLocalId::integer(1)]),
            );
        }

        pub fn call_unlock_non_fungibles_directly(&self) {
            let _proof = self.vault.as_fungible().create_proof_of_amount(dec!(1));

            ScryptoVmV1Api::object_call(
                self.vault.0.as_node_id(),
                NON_FUNGIBLE_VAULT_UNLOCK_NON_FUNGIBLES_IDENT,
                scrypto_args!([NonFungibleLocalId::integer(1)]),
            );
        }
    }
}
