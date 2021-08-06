use hashbrown::HashMap;
use scrypto::types::*;

use crate::ledger::*;
use crate::model::*;

pub struct InMemoryLedger {
    blueprints: HashMap<Address, Vec<u8>>,
    components: HashMap<Address, Component>,
    accounts: HashMap<Address, Account>,
    resources: HashMap<Address, ResourceInfo>,
}

impl Ledger for InMemoryLedger {
    fn get_blueprint(&self, address: Address) -> Option<Vec<u8>> {
        self.blueprints.get(&address).map(Clone::clone)
    }

    fn put_blueprint(&mut self, address: Address, blueprint: Vec<u8>) {
        self.blueprints.insert(address, blueprint);
    }

    fn get_resource(&self, address: Address) -> Option<ResourceInfo> {
        self.resources.get(&address).map(Clone::clone)
    }

    fn put_resource(&mut self, address: Address, info: ResourceInfo) {
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
}
