use scrypto::rust::collections::*;
use scrypto::rust::string::String;
use scrypto::rust::vec::Vec;
use scrypto::types::*;
use scrypto::utils::*;
use wasmi::*;

use crate::execution::*;
use crate::ledger::*;
use crate::model::*;

/// Runtime is an abstraction layer over the execution state for a transaction.
///
/// It serves as a proxy to the ledger state and keeps track of all state updates.
/// The `flush` method should be called to write all updates into ledger.
///
/// A runtime is shared by a chain of processes, created during the life time
/// of a transaction.
///
pub struct Runtime<'le, T: Ledger> {
    tx_hash: H256,
    ledger: &'le mut T,
    counter: u32,
    logs: Vec<(Level, String)>,
    packages: HashMap<Address, Package>,
    components: HashMap<Address, Component>,
    accounts: HashMap<Address, Account>,
    resources: HashMap<Address, Resource>,
    buckets: HashMap<BID, Bucket>,
    updated_packages: HashSet<Address>,
    updated_components: HashSet<Address>,
    updated_accounts: HashSet<Address>,
    updated_resources: HashSet<Address>,
    updated_buckets: HashSet<BID>,
}

impl<'le, T: Ledger> Runtime<'le, T> {
    pub fn new(tx_hash: H256, ledger: &'le mut T) -> Self {
        Self {
            tx_hash,
            ledger,
            counter: 0,
            logs: Vec::new(),
            packages: HashMap::new(),
            components: HashMap::new(),
            accounts: HashMap::new(),
            resources: HashMap::new(),
            buckets: HashMap::new(),
            updated_packages: HashSet::new(),
            updated_components: HashSet::new(),
            updated_accounts: HashSet::new(),
            updated_resources: HashSet::new(),
            updated_buckets: HashSet::new(),
        }
    }

    /// Returns the transaction hash.
    pub fn tx_hash(&self) -> H256 {
        self.tx_hash
    }

    /// Returns the logs collected so far.
    pub fn logs(&self) -> &Vec<(Level, String)> {
        &self.logs
    }

    /// Adds a log message.
    pub fn add_log(&mut self, level: Level, message: String) {
        self.logs.push((level, message));
    }

    /// Loads a module.
    pub fn load_module(&mut self, address: Address) -> Option<(ModuleRef, MemoryRef)> {
        self.get_package(address)
            .map(|package| load_module(package.code()).expect("All package should be loadable"))
    }

    /// Returns an immutable reference to a package, if exists.
    pub fn get_package(&mut self, address: Address) -> Option<&Package> {
        if self.packages.contains_key(&address) {
            return self.packages.get(&address);
        }

        if let Some(package) = self.ledger.get_package(address) {
            self.packages.insert(address, package);
            self.packages.get(&address)
        } else {
            None
        }
    }

    /// Returns a mutable reference to a package, if exists.
    #[allow(dead_code)]
    pub fn get_package_mut(&mut self, address: Address) -> Option<&mut Package> {
        self.updated_packages.insert(address);

        if self.packages.contains_key(&address) {
            return self.packages.get_mut(&address);
        }

        if let Some(package) = self.ledger.get_package(address) {
            self.packages.insert(address, package);
            self.packages.get_mut(&address)
        } else {
            None
        }
    }

    /// Inserts a new package.
    pub fn put_package(&mut self, address: Address, package: Package) {
        self.updated_packages.insert(address);

        self.packages.insert(address, package);
    }

    /// Returns an immutable reference to a component, if exists.
    pub fn get_component(&mut self, address: Address) -> Option<&Component> {
        if self.components.contains_key(&address) {
            return self.components.get(&address);
        }

        if let Some(component) = self.ledger.get_component(address) {
            self.components.insert(address, component);
            self.components.get(&address)
        } else {
            None
        }
    }
    /// Returns a mutable reference to a component, if exists.
    pub fn get_component_mut(&mut self, address: Address) -> Option<&mut Component> {
        self.updated_components.insert(address);

        if self.components.contains_key(&address) {
            return self.components.get_mut(&address);
        }

        if let Some(component) = self.ledger.get_component(address) {
            self.components.insert(address, component);
            self.components.get_mut(&address)
        } else {
            None
        }
    }

    /// Inserts a new component.
    pub fn put_component(&mut self, address: Address, component: Component) {
        self.updated_components.insert(address);

        self.components.insert(address, component);
    }

    /// Returns an immutable reference to an account, if exists.
    #[allow(dead_code)]
    pub fn get_account(&mut self, address: Address) -> Option<&Account> {
        if self.accounts.contains_key(&address) {
            return self.accounts.get(&address);
        }

        if let Some(account) = self.ledger.get_account(address) {
            self.accounts.insert(address, account);
            self.accounts.get(&address)
        } else {
            None
        }
    }

    /// Returns a mutable reference to an account, if exists.
    pub fn get_account_mut(&mut self, address: Address) -> Option<&mut Account> {
        self.updated_accounts.insert(address);

        if self.accounts.contains_key(&address) {
            return self.accounts.get_mut(&address);
        }

        if let Some(account) = self.ledger.get_account(address) {
            self.accounts.insert(address, account);
            self.accounts.get_mut(&address)
        } else {
            None
        }
    }

    /// Inserts a new account.
    pub fn put_account(&mut self, address: Address, account: Account) {
        self.updated_accounts.insert(address);

        self.accounts.insert(address, account);
    }

    /// Returns an immutable reference to a resource, if exists.
    pub fn get_resource(&mut self, address: Address) -> Option<&Resource> {
        if self.resources.contains_key(&address) {
            return self.resources.get(&address);
        }

        if let Some(resource) = self.ledger.get_resource(address) {
            self.resources.insert(address, resource);
            self.resources.get(&address)
        } else {
            None
        }
    }

    /// Returns a mutable reference to a resource, if exists.
    #[allow(dead_code)]
    pub fn get_resource_mut(&mut self, address: Address) -> Option<&mut Resource> {
        self.updated_resources.insert(address);

        if self.resources.contains_key(&address) {
            return self.resources.get_mut(&address);
        }

        if let Some(resource) = self.ledger.get_resource(address) {
            self.resources.insert(address, resource);
            self.resources.get_mut(&address)
        } else {
            None
        }
    }

    /// Inserts a new resource.
    pub fn put_resource(&mut self, address: Address, resource: Resource) {
        self.updated_resources.insert(address);

        self.resources.insert(address, resource);
    }

    /// Returns an immutable reference to a bucket, if exists.
    #[allow(dead_code)]
    pub fn get_bucket(&mut self, bid: BID) -> Option<&Bucket> {
        if self.buckets.contains_key(&bid) {
            return self.buckets.get(&bid);
        }

        if let Some(bucket) = self.ledger.get_bucket(bid) {
            self.buckets.insert(bid, bucket);
            self.buckets.get(&bid)
        } else {
            None
        }
    }

    /// Returns a mutable reference to a bucket, if exists.
    pub fn get_bucket_mut(&mut self, bid: BID) -> Option<&mut Bucket> {
        self.updated_buckets.insert(bid);

        if self.buckets.contains_key(&bid) {
            return self.buckets.get_mut(&bid);
        }

        if let Some(bucket) = self.ledger.get_bucket(bid) {
            self.buckets.insert(bid, bucket);
            self.buckets.get_mut(&bid)
        } else {
            None
        }
    }

    /// Inserts a new bucket.
    pub fn put_bucket(&mut self, bid: BID, bucket: Bucket) {
        self.updated_buckets.insert(bid);

        self.buckets.insert(bid, bucket);
    }

    /// Creates a new package bid.
    pub fn new_package_address(&mut self) -> Address {
        let mut data = self.tx_hash.as_ref().to_vec();
        data.extend(self.counter.to_le_bytes());

        let hash = sha256_twice(data);
        Address::Package(hash.lower_26_bytes())
    }

    /// Creates a new component address.
    pub fn new_component_address(&mut self) -> Address {
        let mut data = self.tx_hash.as_ref().to_vec();
        data.extend(self.counter.to_le_bytes());

        let hash = sha256_twice(data);
        Address::Component(hash.lower_26_bytes())
    }

    /// Creates a new resource address.
    pub fn new_resource_address(&self, creator: Address, symbol: &str) -> Address {
        let mut data: Vec<u8> = creator.to_vec();
        data.extend(symbol.as_bytes());

        let hash = sha256_twice(data);
        Address::Resource(hash.lower_26_bytes())
    }

    /// Creates a new transient bucket id.
    pub fn new_transient_bid(&mut self) -> BID {
        self.counter += 1;
        BID::Transient(self.counter - 1)
    }

    /// Creates a new persisted bucket id.
    pub fn new_persisted_bid(&mut self) -> BID {
        self.counter += 1;
        BID::Persisted(self.tx_hash, self.counter - 1)
    }

    /// Creates a new persisted bucket id.
    pub fn new_fixed_rid(&mut self) -> RID {
        self.counter += 1;
        RID::Immutable(self.counter)
    }

    /// Flush changes to ledger.
    pub fn flush(&mut self) {
        let mut addresses = self.updated_packages.clone();
        for address in addresses {
            self.ledger
                .put_package(address, self.packages.get(&address).unwrap().clone());
        }

        addresses = self.updated_components.clone();
        for address in addresses {
            self.ledger
                .put_component(address, self.components.get(&address).unwrap().clone());
        }

        addresses = self.updated_accounts.clone();
        for address in addresses {
            self.ledger
                .put_account(address, self.accounts.get(&address).unwrap().clone());
        }

        addresses = self.updated_resources.clone();
        for address in addresses {
            self.ledger
                .put_resource(address, self.resources.get(&address).unwrap().clone());
        }

        let buckets = self.updated_buckets.clone();
        for bucket in buckets {
            self.ledger
                .put_bucket(bucket, self.buckets.get(&bucket).unwrap().clone());
        }
    }
}
