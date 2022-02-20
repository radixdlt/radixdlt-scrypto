use sbor::*;
use scrypto::buffer::*;
use scrypto::engine::*;
use scrypto::rust::borrow::ToOwned;
use scrypto::rust::collections::*;
use scrypto::rust::vec::Vec;
use scrypto::types::*;

use crate::model::*;

const XRD_SYMBOL: &str = "XRD";
const XRD_NAME: &str = "Radix";
const XRD_DESCRIPTION: &str = "The Radix Public Network's native token, used to pay the network's required transaction fees and to secure the network through staking to its validator nodes.";
const XRD_URL: &str = "https://tokens.radixdlt.com";
const XRD_MAX_SUPPLY: i128 = 24_000_000_000_000i128;
const XRD_VAULT_ID: Vid = Vid(H256([0u8; 32]), 0);

const SYSTEM_COMPONENT_NAME: &str = "System";

#[derive(TypeId, Encode, Decode)]
struct SystemComponentState {
    xrd: Vid,
}

pub trait QueryableSubstateStore {
    fn get_lazy_map_entries(
        &self,
        component_address: &Address,
        mid: &Mid,
    ) -> HashMap<Vec<u8>, Vec<u8>>;
}

/// A ledger stores all transactions and substates.
pub trait SubstateStore {
    fn get_substate(&self, address: &Address) -> Option<Vec<u8>>;
    fn put_substate(&mut self, address: &Address, substate: &[u8]);

    fn get_child_substate(&self, address: &Address, key: &[u8]) -> Option<Vec<u8>>;
    fn put_child_substate(&mut self, address: &Address, key: &[u8], substate: &[u8]);

    /// Child Objects
    fn get_non_fungible(
        &self,
        resource_address: &Address,
        id: &NonFungibleKey,
    ) -> Option<NonFungible>;
    fn put_non_fungible(
        &mut self,
        resource_address: &Address,
        id: &NonFungibleKey,
        non_fungible: NonFungible,
    );

    fn bootstrap(&mut self) {
        let package: Option<Package> = self
            .get_substate(&SYSTEM_PACKAGE)
            .map(|v| scrypto_decode(&v).unwrap());
        if package.is_none() {
            // System package
            let system_package =
                Package::new(include_bytes!("../../../assets/system.wasm").to_vec());
            self.put_substate(&SYSTEM_PACKAGE, &scrypto_encode(&system_package));

            // Account package
            let account_package =
                Package::new(include_bytes!("../../../assets/account.wasm").to_vec());
            self.put_substate(&ACCOUNT_PACKAGE, &scrypto_encode(&account_package));

            // Radix token resource definition
            let mut metadata = HashMap::new();
            metadata.insert("symbol".to_owned(), XRD_SYMBOL.to_owned());
            metadata.insert("name".to_owned(), XRD_NAME.to_owned());
            metadata.insert("description".to_owned(), XRD_DESCRIPTION.to_owned());
            metadata.insert("url".to_owned(), XRD_URL.to_owned());

            let xrd = ResourceDef::new(
                ResourceType::Fungible { divisibility: 18 },
                metadata,
                0,
                0,
                HashMap::new(),
                &Some(NewSupply::Fungible {
                    amount: XRD_MAX_SUPPLY.into(),
                }),
            )
            .unwrap();
            self.put_substate(&RADIX_TOKEN, &scrypto_encode(&xrd));
            let ecdsa_token = ResourceDef::new(
                ResourceType::NonFungible,
                HashMap::new(),
                0,
                0,
                HashMap::new(),
                &None,
            )
            .unwrap();

            self.put_substate(&ECDSA_TOKEN, &scrypto_encode(&ecdsa_token));

            // Instantiate system component
            let system_vault = Vault::new(Bucket::new(
                RADIX_TOKEN,
                ResourceType::Fungible { divisibility: 18 },
                Supply::Fungible {
                    amount: XRD_MAX_SUPPLY.into(),
                }));
            self.put_child_substate(
                &SYSTEM_COMPONENT,
                &scrypto_encode(&XRD_VAULT_ID),
                &scrypto_encode(&system_vault),
            );

            let system_component = Component::new(
                SYSTEM_PACKAGE,
                SYSTEM_COMPONENT_NAME.to_owned(),
                scrypto_encode(&SystemComponentState { xrd: XRD_VAULT_ID }),
            );
            self.put_substate(&SYSTEM_COMPONENT, &scrypto_encode(&system_component));
        }
    }

    fn get_epoch(&self) -> u64;

    fn set_epoch(&mut self, epoch: u64);

    // Before transaction hash is defined, we use the following TEMPORARY interfaces
    // to introduce entropy for address derivation.

    fn get_nonce(&self) -> u64;

    fn increase_nonce(&mut self);
}
