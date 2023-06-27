use scrypto::api::*;
use scrypto::engine::scrypto_env::*;
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
                vault: Vault::with_bucket(bucket),
            }
            .instantiate()
            .prepare_to_globalize(OwnerRole::None)
            .globalize()
        }

        pub fn call_lock_fungible_amount_directly(&self) {
            ScryptoEnv
                .call_method(
                    self.vault.0.as_node_id(),
                    FUNGIBLE_VAULT_LOCK_FUNGIBLE_AMOUNT_IDENT,
                    scrypto_args!(Decimal::from(1)),
                )
                .unwrap();
        }

        pub fn call_unlock_fungible_amount_directly(&self) {
            let _proof = self.vault.create_proof_of_amount(dec!(1));

            ScryptoEnv
                .call_method(
                    self.vault.0.as_node_id(),
                    FUNGIBLE_VAULT_UNLOCK_FUNGIBLE_AMOUNT_IDENT,
                    scrypto_args!(Decimal::from(1)),
                )
                .unwrap();
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
                vault: Vault::with_bucket(bucket),
            }
            .instantiate()
            .prepare_to_globalize(OwnerRole::None)
            .globalize()
        }

        pub fn call_lock_non_fungibles_directly(&self) {
            ScryptoEnv
                .call_method(
                    self.vault.0.as_node_id(),
                    NON_FUNGIBLE_VAULT_LOCK_NON_FUNGIBLES_IDENT,
                    scrypto_args!([NonFungibleLocalId::integer(1)]),
                )
                .unwrap();
        }

        pub fn call_unlock_non_fungibles_directly(&self) {
            let _proof = self.vault.create_proof_of_amount(dec!(1));

            ScryptoEnv
                .call_method(
                    self.vault.0.as_node_id(),
                    NON_FUNGIBLE_VAULT_UNLOCK_NON_FUNGIBLES_IDENT,
                    scrypto_args!([NonFungibleLocalId::integer(1)]),
                )
                .unwrap();
        }
    }
}
