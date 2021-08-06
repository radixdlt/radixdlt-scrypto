use hashbrown::HashMap;
use scrypto::kernel::*;
use scrypto::types::*;
use scrypto::utils::*;

use crate::execution::*;
use crate::ledger::*;
use crate::model::*;

/// Transaction execution context.
pub struct TransactionContext<T> {
    tx_hash: Hash,
    ledger: T,
    logger: Logger,
    component_counter: u32,
    bucket_counter: u32,
    blueprints: HashMap<Address, Vec<u8>>,
    components: HashMap<Address, Component>,
    accounts: HashMap<Address, Account>,
    resources: HashMap<Address, ResourceInfo>,
}

impl<T: Ledger> TransactionContext<T> {
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

    pub fn tx_hash(&self) -> Hash {
        self.tx_hash
    }

    pub fn ledger(&self) -> &T {
        &self.ledger
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
                self.blueprints().get(&address)
            }
            None => None,
        }
    }

    pub fn blueprints(&self) -> &HashMap<Address, Vec<u8>> {
        &self.blueprints
    }

    pub fn blueprints_mut(&mut self) -> &mut HashMap<Address, Vec<u8>> {
        &mut self.blueprints
    }

    pub fn components(&self) -> &HashMap<Address, Component> {
        &self.components
    }
    pub fn components_mut(&mut self) -> &mut HashMap<Address, Component> {
        &mut self.components
    }

    pub fn accounts(&self) -> &HashMap<Address, Account> {
        &self.accounts
    }

    pub fn accounts_mut(&mut self) -> &mut HashMap<Address, Account> {
        &mut self.accounts
    }

    pub fn resources(&self) -> &HashMap<Address, ResourceInfo> {
        &self.resources
    }

    pub fn resources_mut(&mut self) -> &mut HashMap<Address, ResourceInfo> {
        &mut self.resources
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
