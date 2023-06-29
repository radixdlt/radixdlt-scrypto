use scrypto::api::*;
use scrypto::engine::scrypto_env::ScryptoEnv;
use scrypto::prelude::*;

#[blueprint]
mod recall {
    struct RecallTest {
        vault: Vault,
    }

    impl RecallTest {
        pub fn new() -> Global<RecallTest> {
            let bucket = ResourceBuilder::new_fungible(OwnerRole::None)
                .mintable(mintable! {
                    minter => rule!(allow_all), locked;
                    minter_updater => rule!(deny_all), locked;
                })
                .burnable(burnable! {
                    burner => rule!(allow_all), locked;
                    burner_updater => rule!(deny_all), locked;
                })
                .recallable(recallable! {
                    recaller => rule!(allow_all), locked;
                    recaller_updater => rule!(deny_all), locked;
                })
                .metadata(metadata! {
                    init {
                        "name" => "TestToken".to_owned(), locked;
                    }
                })
                .mint_initial_supply(500);

            Self {
                vault: Vault::with_bucket(bucket),
            }
            .instantiate()
            .prepare_to_globalize(OwnerRole::None)
            .globalize()
        }

        pub fn recall_on_internal_vault(&self) -> Bucket {
            scrypto_decode(
                &ScryptoEnv
                    .call_method_advanced(
                        self.vault.0.as_node_id(),
                        true,
                        ObjectModuleId::Main,
                        VAULT_RECALL_IDENT,
                        scrypto_args!(Decimal::ONE),
                    )
                    .unwrap(),
            )
            .unwrap()
        }

        pub fn recall_on_direct_access_ref(reference: InternalAddress) -> Bucket {
            scrypto_decode(
                &ScryptoEnv
                    .call_method_advanced(
                        reference.as_node_id(),
                        true,
                        ObjectModuleId::Main,
                        VAULT_RECALL_IDENT,
                        scrypto_args!(Decimal::ONE),
                    )
                    .unwrap(),
            )
            .unwrap()
        }
    }
}
