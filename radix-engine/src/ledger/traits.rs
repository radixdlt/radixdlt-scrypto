use sbor::*;
use scrypto::buffer::*;
use scrypto::constants::*;
use scrypto::engine::types::*;
use scrypto::rust::borrow::ToOwned;
use scrypto::rust::collections::*;
use scrypto::rust::vec::Vec;

use crate::model::*;

const XRD_SYMBOL: &str = "XRD";
const XRD_NAME: &str = "Radix";
const XRD_DESCRIPTION: &str = "The Radix Public Network's native token, used to pay the network's required transaction fees and to secure the network through staking to its validator nodes.";
const XRD_URL: &str = "https://tokens.radixdlt.com";
const XRD_MAX_SUPPLY: i128 = 24_000_000_000_000i128;
const XRD_VAULT_ID: VaultId = (Hash([0u8; 32]), 0);
const XRD_VAULT: scrypto::resource::Vault = scrypto::resource::Vault(XRD_VAULT_ID);

const SYSTEM_COMPONENT_NAME: &str = "System";

#[derive(TypeId, Encode, Decode)]
struct SystemComponentState {
    xrd: scrypto::resource::Vault,
}

pub trait QueryableSubstateStore {
    fn get_lazy_map_entries(
        &self,
        component_id: ComponentId,
        lazy_map_id: &LazyMapId,
    ) -> HashMap<Vec<u8>, Vec<u8>>;
}

#[derive(Clone, Debug, Encode, Decode, TypeId)]
pub struct Substate {
    pub value: Vec<u8>,
    pub phys_id: u64,
}

/// A ledger stores all transactions and substates.
pub trait SubstateStore {
    fn get_substate<T: Encode>(&self, address: &T) -> Option<Substate>;
    fn put_substate<T: Encode>(&mut self, address: &T, substate: Substate);

    fn get_child_substate<T: Encode>(&self, address: &T, key: &[u8]) -> Option<Vec<u8>>;
    fn put_child_substate<T: Encode>(&mut self, address: &T, key: &[u8], substate: &[u8]);

    // Temporary Encoded/Decoded interface
    fn get_decoded_substate<A: Encode, T: Decode>(&self, address: &A) -> Option<(T, u64)> {
        self.get_substate(address)
            .map(|s| (scrypto_decode(&s.value).unwrap(), s.phys_id))
    }
    fn put_encoded_substate<A: Encode, V: Encode>(&mut self, address: &A, value: &V, phys_id: u64) {
        self.put_substate(
            address,
            Substate {
                value: scrypto_encode(value),
                phys_id,
            },
        );
    }
    fn get_decoded_child_substate<A: Encode, K: Encode, T: Decode>(
        &self,
        address: &A,
        key: &K,
    ) -> Option<T> {
        let child_key = &scrypto_encode(key);
        self.get_child_substate(address, child_key)
            .map(|v| scrypto_decode(&v).unwrap())
    }
    fn put_encoded_child_substate<A: Encode, K: Encode, V: Encode>(
        &mut self,
        address: &A,
        key: &K,
        value: &V,
    ) {
        let child_key = &scrypto_encode(key);
        self.put_child_substate(address, child_key, &scrypto_encode(value));
    }
    fn put_encoded_grand_child_substate<A: Encode, C: Encode>(
        &mut self,
        address: &A,
        child_key: &C,
        grand_child_key: &[u8],
        value: &[u8],
    ) {
        let mut key = scrypto_encode(child_key);
        key.extend(grand_child_key.to_vec());
        self.put_child_substate(address, &key, value);
    }
    fn get_decoded_grand_child_substate<A: Encode, C: Encode>(
        &self,
        address: &A,
        child_key: &C,
        grand_child_key: &[u8],
    ) -> Option<Vec<u8>> {
        let mut key = scrypto_encode(child_key);
        key.extend(grand_child_key.to_vec());
        self.get_child_substate(address, &key)
    }

    fn bootstrap(&mut self) {
        let package: Option<Package> = self
            .get_decoded_substate(&SYSTEM_PACKAGE)
            .map(|(package, _)| package);
        if package.is_none() {
            // System package
            let system_package =
                Package::new(include_bytes!("../../../assets/system.wasm").to_vec());
            self.put_encoded_substate(&SYSTEM_PACKAGE, &system_package, self.get_nonce());

            // Account package
            let account_package =
                Package::new(include_bytes!("../../../assets/account.wasm").to_vec());
            self.put_encoded_substate(&ACCOUNT_PACKAGE, &account_package, self.get_nonce());

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
                &Some(Supply::Fungible {
                    amount: XRD_MAX_SUPPLY.into(),
                }),
            )
            .unwrap();
            self.put_encoded_substate(&RADIX_TOKEN, &xrd, self.get_nonce());

            let ecdsa_token = ResourceDef::new(
                ResourceType::NonFungible,
                HashMap::new(),
                0,
                0,
                HashMap::new(),
                &None,
            )
            .unwrap();
            self.put_encoded_substate(&ECDSA_TOKEN, &ecdsa_token, self.get_nonce());

            // Instantiate system component
            let system_vault = Vault::new(Bucket::new(
                RADIX_TOKEN,
                ResourceType::Fungible { divisibility: 18 },
                Resource::Fungible {
                    amount: XRD_MAX_SUPPLY.into(),
                },
            ));
            self.put_encoded_child_substate(&SYSTEM_COMPONENT, &XRD_VAULT_ID, &system_vault);

            let system_component = Component::new(
                SYSTEM_PACKAGE,
                SYSTEM_COMPONENT_NAME.to_owned(),
                scrypto_encode(&SystemComponentState { xrd: XRD_VAULT }),
            );
            self.put_encoded_substate(&SYSTEM_COMPONENT, &system_component, self.get_nonce());
        }
    }

    fn get_epoch(&self) -> u64;

    fn set_epoch(&mut self, epoch: u64);

    // Before transaction hash is defined, we use the following TEMPORARY interfaces
    // to introduce entropy for id derivation.

    fn get_nonce(&self) -> u64;

    fn increase_nonce(&mut self);
}
