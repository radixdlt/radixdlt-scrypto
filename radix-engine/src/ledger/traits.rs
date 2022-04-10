use sbor::*;
use scrypto::auth;
use scrypto::buffer::*;
use scrypto::constants::*;
use scrypto::crypto::*;
use scrypto::engine::types::*;
use scrypto::resource::ComponentAuthorization;
use scrypto::resource::ResourceMethod::TakeFromVault;
use scrypto::rust::borrow::ToOwned;
use scrypto::rust::collections::*;
use scrypto::rust::vec::Vec;

use crate::model::*;

const XRD_SYMBOL: &str = "XRD";
const XRD_NAME: &str = "Radix";
const XRD_DESCRIPTION: &str = "The Radix Public Network's native token, used to pay the network's required transaction fees and to secure the network through staking to its validator nodes.";
const XRD_URL: &str = "https://tokens.radixdlt.com";
const XRD_MAX_SUPPLY: i128 = 24_000_000_000i128;
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
        component_address: ComponentAddress,
        lazy_map_id: &LazyMapId,
    ) -> HashMap<Vec<u8>, Vec<u8>>;
}

#[derive(Clone, Debug, Encode, Decode, TypeId)]
pub struct Substate {
    pub value: Vec<u8>,
    pub phys_id: (Hash, u32),
}

#[derive(Debug)]
pub struct SubstateIdGenerator {
    tx_hash: Hash,
    count: u32,
}

impl SubstateIdGenerator {
    pub fn new(tx_hash: Hash) -> Self {
        Self { tx_hash, count: 0 }
    }

    pub fn next(&mut self) -> (Hash, u32) {
        let value = self.count;
        self.count = self.count + 1;
        (self.tx_hash.clone(), value)
    }
}

/// A ledger stores all transactions and substates.
pub trait SubstateStore {
    fn get_substate<T: Encode>(&self, address: &T) -> Option<Substate>;
    fn put_substate<T: Encode>(&mut self, address: &T, substate: Substate);

    fn get_child_substate<T: Encode>(&self, address: &T, key: &[u8]) -> Option<Substate>;
    fn put_child_substate<T: Encode>(&mut self, address: &T, key: &[u8], substate: Substate);

    // Temporary Encoded/Decoded interface
    fn get_decoded_substate<A: Encode, T: Decode>(&self, address: &A) -> Option<(T, (Hash, u32))> {
        self.get_substate(address)
            .map(|s| (scrypto_decode(&s.value).unwrap(), s.phys_id))
    }
    fn put_encoded_substate<A: Encode, V: Encode>(
        &mut self,
        address: &A,
        value: &V,
        phys_id: (Hash, u32),
    ) {
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
    ) -> Option<(T, (Hash, u32))> {
        let child_key = &scrypto_encode(key);
        self.get_child_substate(address, child_key)
            .map(|s| (scrypto_decode(&s.value).unwrap(), s.phys_id))
    }
    fn put_encoded_child_substate<A: Encode, K: Encode, V: Encode>(
        &mut self,
        address: &A,
        key: &K,
        value: &V,
        phys_id: (Hash, u32),
    ) {
        let child_key = &scrypto_encode(key);
        self.put_child_substate(
            address,
            child_key,
            Substate {
                value: scrypto_encode(value),
                phys_id,
            },
        );
    }
    fn get_decoded_grand_child_substate<A: Encode, C: Encode>(
        &self,
        address: &A,
        child_key: &C,
        grand_child_key: &[u8],
    ) -> Option<(Vec<u8>, (Hash, u32))> {
        let mut key = scrypto_encode(child_key);
        key.extend(grand_child_key.to_vec());
        self.get_child_substate(address, &key)
            .map(|s| (s.value, s.phys_id))
    }
    fn put_encoded_grand_child_substate<A: Encode, C: Encode>(
        &mut self,
        address: &A,
        child_key: &C,
        grand_child_key: &[u8],
        value: &[u8],
        phys_id: (Hash, u32),
    ) {
        let mut key = scrypto_encode(child_key);
        key.extend(grand_child_key.to_vec());
        self.put_child_substate(
            address,
            &key,
            Substate {
                value: value.to_vec(),
                phys_id,
            },
        );
    }

    fn bootstrap(&mut self) {
        let package: Option<Package> = self
            .get_decoded_substate(&SYSTEM_PACKAGE)
            .map(|(package, _)| package);
        if package.is_none() {
            let tx_hash = hash(self.get_and_increase_nonce().to_le_bytes());
            let mut id_gen = SubstateIdGenerator::new(tx_hash);

            // System package
            let system_package =
                Package::new(include_bytes!("../../../assets/system.wasm").to_vec()).unwrap();
            self.put_encoded_substate(&SYSTEM_PACKAGE, &system_package, id_gen.next());

            // Account package
            let account_package =
                Package::new(include_bytes!("../../../assets/account.wasm").to_vec()).unwrap();
            self.put_encoded_substate(&ACCOUNT_PACKAGE, &account_package, id_gen.next());

            // Radix token resource address
            let mut metadata = HashMap::new();
            metadata.insert("symbol".to_owned(), XRD_SYMBOL.to_owned());
            metadata.insert("name".to_owned(), XRD_NAME.to_owned());
            metadata.insert("description".to_owned(), XRD_DESCRIPTION.to_owned());
            metadata.insert("url".to_owned(), XRD_URL.to_owned());

            let mut resource_auth = HashMap::new();
            resource_auth.insert(TakeFromVault, auth!(allow_all));

            let mut xrd = ResourceManager::new(
                ResourceType::Fungible { divisibility: 18 },
                metadata,
                resource_auth,
            )
            .unwrap();
            self.put_encoded_substate(&RADIX_TOKEN, &xrd, id_gen.next());
            let minted_xrd = xrd
                .mint_fungible(XRD_MAX_SUPPLY.into(), RADIX_TOKEN.clone())
                .unwrap();

            let mut ecdsa_resource_auth = HashMap::new();
            ecdsa_resource_auth.insert(TakeFromVault, auth!(allow_all));
            let ecdsa_token = ResourceManager::new(
                ResourceType::NonFungible,
                HashMap::new(),
                ecdsa_resource_auth,
            )
            .unwrap();
            self.put_encoded_substate(&ECDSA_TOKEN, &ecdsa_token, id_gen.next());

            // Instantiate system component
            let system_vault = Vault::new(minted_xrd);
            self.put_encoded_child_substate(
                &SYSTEM_COMPONENT,
                &XRD_VAULT_ID,
                &system_vault,
                id_gen.next(),
            );

            let mut authorization = ComponentAuthorization::new();
            authorization.insert("free_xrd", auth!(allow_all));

            let system_component = Component::new(
                SYSTEM_PACKAGE,
                SYSTEM_COMPONENT_NAME.to_owned(),
                authorization,
                scrypto_encode(&SystemComponentState { xrd: XRD_VAULT }),
            );
            self.put_encoded_substate(&SYSTEM_COMPONENT, &system_component, id_gen.next());
        }
    }

    fn get_epoch(&self) -> u64;

    fn set_epoch(&mut self, epoch: u64);

    // TODO: redefine what nonce is and how it's updated
    // For now, we bump nonce only when a transaction has been committed
    // or when an account is created (for testing).

    fn get_nonce(&self) -> u64;

    fn increase_nonce(&mut self);

    fn get_and_increase_nonce(&mut self) -> u64 {
        let nonce = self.get_nonce();
        self.increase_nonce();
        nonce
    }
}
