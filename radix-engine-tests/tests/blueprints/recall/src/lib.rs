use scrypto::api::*;
use scrypto::engine::scrypto_env::ScryptoEnv;
use scrypto::prelude::*;

#[blueprint]
mod recall {
    struct RecallTest {
        vault: Vault,
    }

    impl RecallTest {
        pub fn new() -> ComponentAddress {
            let bucket = ResourceBuilder::new_fungible()
                .mintable(rule!(allow_all), rule!(deny_all))
                .burnable(rule!(allow_all), rule!(deny_all))
                .recallable(rule!(allow_all), rule!(deny_all))
                .metadata("name", "TestToken")
                .mint_initial_supply(500);

            Self {
                vault: Vault::with_bucket(bucket),
            }
            .instantiate()
            .globalize()
        }

        pub fn recall_on_self_vault(&self) -> Bucket {
            scrypto_decode(
                &ScryptoEnv
                    .call_method(
                        self.vault.0.as_node_id(),
                        VAULT_RECALL_IDENT,
                        scrypto_args!(Decimal::ONE),
                    )
                    .unwrap(),
            )
            .unwrap()
        }
    }
}
