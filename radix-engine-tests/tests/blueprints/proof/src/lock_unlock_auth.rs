use scrypto::api::*;
use scrypto::engine::scrypto_env::*;
use scrypto::prelude::*;

#[blueprint]
mod lock_unlock_auth {
    struct LockUnlockAuth {
        vault: Vault,
    }

    impl LockUnlockAuth {
        pub fn new_fungible() -> ComponentAddress {
            let bucket = ResourceBuilder::new_fungible().mint_initial_supply(100);

            Self {
                vault: Vault::with_bucket(bucket),
            }
            .instantiate()
            .globalize()
        }

        pub fn call_lock_fungible_amount_directly(&self) {
            ScryptoEnv
                .call_method(self.vault.0.as_node_id(), "lock_fungible_amount", scrypto_args!(
                    Decimal::from(1)
                ))
                .unwrap();
        }

        pub fn call_unlock_fungible_amount_directly(&self) {
            let _proof = self.vault.create_proof_by_amount(dec!(1));

            ScryptoEnv
                .call_method(self.vault.0.as_node_id(), "unlock_fungible_amount", scrypto_args!(
                    Decimal::from(1)
                ))
                .unwrap();
        }
    }
}
