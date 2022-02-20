use scrypto::prelude::scrypto_decode;
use scrypto::prelude::scrypto_encode;
use scrypto::rust::collections::HashMap;
use scrypto::rust::vec::Vec;
use scrypto::types::*;

use crate::ledger::*;
use crate::model::*;

/// An in-memory ledger stores all substates in host memory.
#[derive(Debug, Clone)]
pub struct InMemorySubstateStore {
    substates: HashMap<Address, Vec<u8>>,
    child_substates: HashMap<(Address, Vec<u8>), Vec<u8>>,
    vaults: HashMap<(Address, Vid), Vec<u8>>,
    non_fungibles: HashMap<(Address, NonFungibleKey), NonFungible>,
    current_epoch: u64,
    nonce: u64,
}

impl InMemorySubstateStore {
    pub fn new() -> Self {
        Self {
            substates: HashMap::new(),
            child_substates: HashMap::new(),
            vaults: HashMap::new(),
            non_fungibles: HashMap::new(),
            current_epoch: 0,
            nonce: 0,
        }
    }

    pub fn with_bootstrap() -> Self {
        let mut ledger = Self::new();
        ledger.bootstrap();
        ledger
    }
}

impl Default for InMemorySubstateStore {
    fn default() -> Self {
        Self::new()
    }
}

impl SubstateStore for InMemorySubstateStore {
    fn get_substate(&self, address: &Address) -> Option<Vec<u8>> {
        self.substates.get(address).cloned()
    }

    fn put_substate(&mut self, address: &Address, substate: &[u8]) {
        self.substates.insert(*address, substate.to_vec());
    }

    fn get_child_substate(&self, address: &Address, key: &[u8]) -> Option<Vec<u8>> {
        let mut id = scrypto_encode(address);
        id.extend(key.to_vec());
        self.child_substates.get(&(*address, id)).cloned()
    }

    fn put_child_substate(&mut self, address: &Address, key: &[u8], substate: &[u8]) {
        let mut id = scrypto_encode(address);
        id.extend(key.to_vec());
        self.child_substates.insert((*address, id), substate.to_vec());
    }

    fn get_vault(&self, component_address: &Address, vid: &Vid) -> Vault {
        self.vaults
            .get(&(component_address.clone(), vid.clone()))
            .map(|data| scrypto_decode(data).unwrap())
            .unwrap()
    }

    fn put_vault(&mut self, component_address: &Address, vid: &Vid, vault: Vault) {
        let data = scrypto_encode(&vault);
        self.vaults
            .insert((component_address.clone(), vid.clone()), data);
    }

    fn get_non_fungible(
        &self,
        resource_address: &Address,
        key: &NonFungibleKey,
    ) -> Option<NonFungible> {
        self.non_fungibles
            .get(&(resource_address.clone(), key.clone()))
            .cloned()
    }

    fn put_non_fungible(
        &mut self,
        resource_address: &Address,
        key: &NonFungibleKey,
        non_fungible: NonFungible,
    ) {
        self.non_fungibles
            .insert((resource_address.clone(), key.clone()), non_fungible);
    }

    fn get_epoch(&self) -> u64 {
        self.current_epoch
    }

    fn set_epoch(&mut self, epoch: u64) {
        self.current_epoch = epoch;
    }

    fn get_nonce(&self) -> u64 {
        self.nonce
    }

    fn increase_nonce(&mut self) {
        self.nonce += 1;
    }
}
