use hashbrown::HashMap;
use scrypto::kernel::*;
use scrypto::types::*;
use scrypto::utils::*;

use crate::execution::*;
use crate::ledger::*;
use crate::model::*;

/// Represents the transaction execution runtime, one per transaction.
/// A runtime is shared by a chain of processes, created during the execution of the transaction.
pub struct Runtime<T> {
    tx_hash: Hash,
    ledger: T,
    logger: Logger,
    component_counter: u32,
    bucket_counter: u32,
    blueprints: HashMap<Address, Vec<u8>>,
    components: HashMap<Address, Component>,
    accounts: HashMap<Address, Account>,
    resources: HashMap<Address, ResourceInfo>,
    // TODO track updates
}

impl<T: Ledger> Runtime<T> {
    pub fn new(tx_hash: Hash, ledger: T, logger: Logger) -> Self {
        Self {
            tx_hash,
            ledger,
            logger,
            component_counter: 0,
            bucket_counter: 0,
            blueprints: HashMap::new(),
            components: HashMap::new(),
            accounts: HashMap::new(),
            resources: HashMap::new(),
        }
    }

    pub fn logger(&self) -> &Logger {
        &self.logger
    }

    pub fn get_blueprint(&mut self, address: Address) -> Option<&Vec<u8>> {
        if self.blueprints.contains_key(&address) {
            return self.blueprints.get(&address);
        }

        match self.ledger.get_blueprint(address) {
            Some(blueprint) => {
                self.blueprints.insert(address, blueprint);
                self.blueprints.get(&address)
            }
            None => None,
        }
    }

    pub fn put_blueprint(&mut self, address: Address, blueprint: Vec<u8>) {
        self.blueprints.insert(address, blueprint);
    }

    pub fn get_component(&mut self, address: Address) -> Option<&mut Component> {
        if self.components.contains_key(&address) {
            return self.components.get_mut(&address);
        }

        match self.ledger.get_component(address) {
            Some(component) => {
                self.components.insert(address, component);
                self.components.get_mut(&address)
            }
            None => None,
        }
    }

    pub fn put_component(&mut self, address: Address, component: Component) {
        self.components.insert(address, component);
    }

    pub fn get_account(&mut self, address: Address) -> Option<&mut Account> {
        if self.accounts.contains_key(&address) {
            return self.accounts.get_mut(&address);
        }

        match self.ledger.get_account(address) {
            Some(account) => {
                self.accounts.insert(address, account);
                self.accounts.get_mut(&address)
            }
            None => None,
        }
    }

    pub fn put_account(&mut self, address: Address, account: Account) {
        self.accounts.insert(address, account);
    }

    pub fn get_resource(&mut self, address: Address) -> Option<&ResourceInfo> {
        if self.resources.contains_key(&address) {
            return self.resources.get(&address);
        }

        match self.ledger.get_resource(address) {
            Some(resource) => {
                self.resources.insert(address, resource);
                self.resources.get(&address)
            }
            None => None,
        }
    }

    pub fn put_resource(&mut self, address: Address, resource: ResourceInfo) {
        self.resources.insert(address, resource);
    }

    pub fn new_blueprint_address(&mut self, code: &[u8]) -> Address {
        Address::Blueprint(sha256_twice(code).lower_26_bytes())
    }

    pub fn new_component_address(&mut self) -> Address {
        let mut data = self.tx_hash.as_ref().to_vec();
        data.extend(self.component_counter.to_le_bytes());

        let hash = sha256_twice(data);
        Address::Component(hash.lower_26_bytes())
    }

    pub fn new_resource_address(&self, owner: Address, symbol: &str) -> Address {
        let mut data: Vec<u8> = owner.into();
        data.extend(symbol.as_bytes());

        let hash = sha256_twice(data);
        Address::Resource(hash.lower_26_bytes())
    }

    pub fn new_bid(&mut self) -> BID {
        self.bucket_counter += 1;
        BID::Transient(self.bucket_counter - 1)
    }
}
