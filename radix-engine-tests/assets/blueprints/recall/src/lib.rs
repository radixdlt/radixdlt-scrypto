use scrypto::prelude::*;

#[blueprint]
mod recall {
    struct RecallTest {
        vault: Vault,
    }

    impl RecallTest {
        pub fn new() -> (Global<RecallTest>, ResourceAddress) {
            let bucket = ResourceBuilder::new_fungible(OwnerRole::None)
                .mint_roles(mint_roles! {
                    minter => rule!(allow_all);
                    minter_updater => rule!(deny_all);
                })
                .burn_roles(burn_roles! {
                    burner => rule!(allow_all);
                    burner_updater => rule!(deny_all);
                })
                .recall_roles(recall_roles! {
                    recaller => rule!(allow_all);
                    recaller_updater => rule!(deny_all);
                })
                .metadata(metadata! {
                    init {
                        "name" => "TestToken".to_owned(), locked;
                    }
                })
                .mint_initial_supply(500);

            let address = bucket.resource_address();

            let global = Self {
                vault: Vault::with_bucket(bucket.into()),
            }
            .instantiate()
            .prepare_to_globalize(OwnerRole::None)
            .globalize();

            (global, address)
        }

        pub fn recall_on_internal_vault(&self) -> Bucket {
            scrypto_decode(&ScryptoVmV1Api::object_call_direct(
                self.vault.0.as_node_id(),
                VAULT_RECALL_IDENT,
                scrypto_args!(Decimal::ONE),
            ))
            .unwrap()
        }

        pub fn recall_on_direct_access_ref(reference: InternalAddress) -> Bucket {
            scrypto_decode(&ScryptoVmV1Api::object_call_direct(
                reference.as_node_id(),
                VAULT_RECALL_IDENT,
                scrypto_args!(Decimal::ONE),
            ))
            .unwrap()
        }

        pub fn recall_on_direct_access_ref_method(&self, reference: InternalAddress) -> Bucket {
            scrypto_decode(&ScryptoVmV1Api::object_call_direct(
                reference.as_node_id(),
                VAULT_RECALL_IDENT,
                scrypto_args!(Decimal::ONE),
            ))
            .unwrap()
        }
    }
}
