use hashbrown::HashMap;
use scrypto::types::*;

use crate::ledger::*;
use crate::model::*;

/// An in-memory ledger stores all substates in host memory.
pub struct InMemoryLedger {
    blueprints: HashMap<Address, Blueprint>,
    components: HashMap<Address, Component>,
    accounts: HashMap<Address, Account>,
    resources: HashMap<Address, Resource>,
    buckets: HashMap<BID, Bucket>,
}

impl InMemoryLedger {
    pub fn new() -> Self {
        Self {
            blueprints: HashMap::new(),
            components: HashMap::new(),
            accounts: HashMap::new(),
            resources: HashMap::new(),
            buckets: HashMap::new(),
        }
    }
}

impl Ledger for InMemoryLedger {
    fn get_blueprint(&self, address: Address) -> Option<Blueprint> {
        self.blueprints.get(&address).map(Clone::clone)
    }

    fn put_blueprint(&mut self, address: Address, blueprint: Blueprint) {
        self.blueprints.insert(address, blueprint);
    }

    fn get_resource(&self, address: Address) -> Option<Resource> {
        self.resources.get(&address).map(Clone::clone)
    }

    fn put_resource(&mut self, address: Address, info: Resource) {
        self.resources.insert(address, info);
    }

    fn get_component(&self, address: Address) -> Option<Component> {
        self.components.get(&address).map(Clone::clone)
    }

    fn put_component(&mut self, address: Address, component: Component) {
        self.components.insert(address, component);
    }

    fn get_account(&self, address: Address) -> Option<Account> {
        self.accounts.get(&address).map(Clone::clone)
    }

    fn put_account(&mut self, address: Address, account: Account) {
        self.accounts.insert(address, account);
    }

    fn get_bucket(&self, bid: BID) -> Option<Bucket> {
        self.buckets.get(&bid).map(Clone::clone)
    }

    fn put_bucket(&mut self, bid: BID, bucket: Bucket) {
        self.buckets.insert(bid, bucket);
    }
}
