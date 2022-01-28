use lru::LruCache;
use scrypto::kernel::*;
use scrypto::rust::collections::*;
use scrypto::rust::string::String;
use scrypto::rust::vec::Vec;
use scrypto::types::*;
use wasmi::*;

use crate::engine::*;
use crate::ledger::*;
use crate::model::*;

/// An abstraction of transaction execution state.
///
/// It acts as the facade of ledger state and keeps track of all temporary state updates,
/// until the `commit()` method is called.
///
/// Typically, a track is shared by all the processes created within a transaction.
///
pub struct Track<'l, L: Ledger> {
    ledger: &'l mut L,
    current_epoch: u64,
    transaction_hash: H256,
    transaction_signers: Vec<Address>,
    id_allocator: IdAllocator,
    logs: Vec<(LogLevel, String)>,
    packages: HashMap<Address, Package>,
    components: HashMap<Address, Component>,
    resource_defs: HashMap<Address, ResourceDef>,
    lazy_maps: HashMap<Mid, LazyMap>,
    vaults: HashMap<Vid, Vault>,
    nfts: HashMap<(Address, u128), Nft>,
    updated_packages: HashSet<Address>,
    updated_components: HashSet<Address>,
    updated_lazy_maps: HashSet<Mid>,
    updated_resource_defs: HashSet<Address>,
    updated_vaults: HashSet<Vid>,
    updated_nfts: HashSet<(Address, u128)>,
    new_entities: Vec<Address>,
    code_cache: LruCache<Address, Module>, // TODO: move to ledger level
}

impl<'l, L: Ledger> Track<'l, L> {
    pub fn new(
        ledger: &'l mut L,
        current_epoch: u64,
        transaction_hash: H256,
        transaction_signers: Vec<Address>,
    ) -> Self {
        Self {
            ledger,
            current_epoch,
            transaction_hash,
            transaction_signers,
            id_allocator: IdAllocator::new(USER_OBJECT_ID_RANGE),
            logs: Vec::new(),
            packages: HashMap::new(),
            components: HashMap::new(),
            resource_defs: HashMap::new(),
            lazy_maps: HashMap::new(),
            vaults: HashMap::new(),
            nfts: HashMap::new(),
            updated_packages: HashSet::new(),
            updated_components: HashSet::new(),
            updated_lazy_maps: HashSet::new(),
            updated_resource_defs: HashSet::new(),
            updated_vaults: HashSet::new(),
            updated_nfts: HashSet::new(),
            new_entities: Vec::new(),
            code_cache: LruCache::new(1024),
        }
    }

    /// Start a process.
    pub fn start_process<'r>(&'r mut self, verbose: bool) -> Process<'r, 'l, L> {
        // FIXME: This is a temp solution
        let signers: BTreeSet<u128> = self
            .transaction_signers
            .clone()
            .into_iter()
            .map(|address| {
                let mut bytes: [u8; 16] = [0; 16];
                match address {
                    Address::Package(d) => bytes[..].copy_from_slice(&d[..16]),
                    Address::Component(d) => bytes[..].copy_from_slice(&d[..16]),
                    Address::ResourceDef(d) => bytes[..].copy_from_slice(&d[..16]),
                    Address::PublicKey(d) => bytes[..].copy_from_slice(&d[..16]),
                }
                u128::from_be_bytes(bytes)
            })
            .collect();
        let mut process = Process::new(0, verbose, self);

        let ecdsa_bucket = Bucket::new(
            ECDSA_TOKEN,
            ResourceType::NonFungible,
            Supply::NonFungible { ids: signers },
        );
        process.create_virtual_bucket_ref(ECDSA_TOKEN_BID, ECDSA_TOKEN_RID, ecdsa_bucket);

        process
    }

    /// Returns the transaction hash.
    pub fn transaction_hash(&self) -> H256 {
        self.transaction_hash
    }

    /// Returns the current epoch.
    pub fn current_epoch(&self) -> u64 {
        self.current_epoch
    }

    /// Returns the logs collected so far.
    pub fn logs(&self) -> &Vec<(LogLevel, String)> {
        &self.logs
    }

    /// Returns new entities created so far.
    pub fn new_entities(&self) -> &[Address] {
        &self.new_entities
    }

    /// Adds a log message.
    pub fn add_log(&mut self, level: LogLevel, message: String) {
        self.logs.push((level, message));
    }

    /// Loads a module.
    pub fn load_module(&mut self, address: Address) -> Option<(ModuleRef, MemoryRef)> {
        match self.get_package(address).map(Clone::clone) {
            Some(p) => {
                if let Some(m) = self.code_cache.get(&address) {
                    Some(instantiate_module(m).unwrap())
                } else {
                    let module = parse_module(p.code()).unwrap();
                    let inst = instantiate_module(&module).unwrap();
                    self.code_cache.put(address, module);
                    Some(inst)
                }
            }
            None => None,
        }
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

    /// Returns an immutable reference to a nft, if exists.
    pub fn get_nft(&mut self, resource_address: Address, id: u128) -> Option<&Nft> {
        if self.nfts.contains_key(&(resource_address, id)) {
            return self.nfts.get(&(resource_address, id));
        }

        if let Some(nft) = self.ledger.get_nft(resource_address, id) {
            self.nfts.insert((resource_address, id), nft);
            self.nfts.get(&(resource_address, id))
        } else {
            None
        }
    }

    /// Returns a mutable reference to a nft, if exists.
    pub fn get_nft_mut(&mut self, resource_address: Address, id: u128) -> Option<&mut Nft> {
        self.updated_nfts.insert((resource_address, id));

        if self.nfts.contains_key(&(resource_address, id)) {
            return self.nfts.get_mut(&(resource_address, id));
        }

        if let Some(nft) = self.ledger.get_nft(resource_address, id) {
            self.nfts.insert((resource_address, id), nft);
            self.nfts.get_mut(&(resource_address, id))
        } else {
            None
        }
    }

    /// Inserts a new nft.
    pub fn put_nft(&mut self, resource_address: Address, id: u128, nft: Nft) {
        self.updated_nfts.insert((resource_address, id));

        self.nfts.insert((resource_address, id), nft);
    }

    /// Returns an immutable reference to a lazy map, if exists.
    pub fn get_lazy_map(&mut self, mid: Mid) -> Option<&LazyMap> {
        if self.lazy_maps.contains_key(&mid) {
            return self.lazy_maps.get(&mid);
        }

        if let Some(lazy_map) = self.ledger.get_lazy_map(mid) {
            self.lazy_maps.insert(mid, lazy_map);
            self.lazy_maps.get(&mid)
        } else {
            None
        }
    }

    /// Returns a mutable reference to a lazy map, if exists.
    pub fn get_lazy_map_mut(&mut self, mid: Mid) -> Option<&mut LazyMap> {
        self.updated_lazy_maps.insert(mid);

        if self.lazy_maps.contains_key(&mid) {
            return self.lazy_maps.get_mut(&mid);
        }

        if let Some(lazy_map) = self.ledger.get_lazy_map(mid) {
            self.lazy_maps.insert(mid, lazy_map);
            self.lazy_maps.get_mut(&mid)
        } else {
            None
        }
    }

    /// Inserts a new lazy map.
    pub fn put_lazy_map(&mut self, mid: Mid, lazy_map: LazyMap) {
        self.updated_lazy_maps.insert(mid);

        self.lazy_maps.insert(mid, lazy_map);
    }

    /// Returns an immutable reference to a resource definition, if exists.
    pub fn get_resource_def(&mut self, address: Address) -> Option<&ResourceDef> {
        if self.resource_defs.contains_key(&address) {
            return self.resource_defs.get(&address);
        }

        if let Some(resource_def) = self.ledger.get_resource_def(address) {
            self.resource_defs.insert(address, resource_def);
            self.resource_defs.get(&address)
        } else {
            None
        }
    }

    /// Returns a mutable reference to a resource definition, if exists.
    #[allow(dead_code)]
    pub fn get_resource_def_mut(&mut self, address: Address) -> Option<&mut ResourceDef> {
        self.updated_resource_defs.insert(address);

        if self.resource_defs.contains_key(&address) {
            return self.resource_defs.get_mut(&address);
        }

        if let Some(resource_def) = self.ledger.get_resource_def(address) {
            self.resource_defs.insert(address, resource_def);
            self.resource_defs.get_mut(&address)
        } else {
            None
        }
    }

    /// Inserts a new resource definition.
    pub fn put_resource_def(&mut self, address: Address, resource_def: ResourceDef) {
        self.updated_resource_defs.insert(address);

        self.resource_defs.insert(address, resource_def);
    }

    /// Returns an immutable reference to a vault, if exists.
    #[allow(dead_code)]
    pub fn get_vault(&mut self, vid: Vid) -> Option<&Vault> {
        if self.vaults.contains_key(&vid) {
            return self.vaults.get(&vid);
        }

        if let Some(vault) = self.ledger.get_vault(vid) {
            self.vaults.insert(vid, vault);
            self.vaults.get(&vid)
        } else {
            None
        }
    }

    /// Returns a mutable reference to a vault, if exists.
    pub fn get_vault_mut(&mut self, vid: Vid) -> Option<&mut Vault> {
        self.updated_vaults.insert(vid);

        if self.vaults.contains_key(&vid) {
            return self.vaults.get_mut(&vid);
        }

        if let Some(vault) = self.ledger.get_vault(vid) {
            self.vaults.insert(vid, vault);
            self.vaults.get_mut(&vid)
        } else {
            None
        }
    }

    /// Inserts a new vault.
    pub fn put_vault(&mut self, vid: Vid, vault: Vault) {
        self.updated_vaults.insert(vid);

        self.vaults.insert(vid, vault);
    }

    /// Creates a new package address.
    pub fn new_package_address(&mut self) -> Address {
        // Security Alert: ensure ID allocating will practically never fail
        let address = self
            .id_allocator
            .new_package_address(self.transaction_hash())
            .unwrap();
        self.new_entities.push(address);
        address
    }

    /// Creates a new component address.
    pub fn new_component_address(&mut self) -> Address {
        let address = self
            .id_allocator
            .new_component_address(self.transaction_hash())
            .unwrap();
        self.new_entities.push(address);
        address
    }

    /// Creates a new resource definition address.
    pub fn new_resource_address(&mut self) -> Address {
        let address = self
            .id_allocator
            .new_resource_address(self.transaction_hash())
            .unwrap();
        self.new_entities.push(address);
        address
    }

    /// Creates a new UUID.
    pub fn new_uuid(&mut self) -> u128 {
        self.id_allocator.new_uuid(self.transaction_hash()).unwrap()
    }

    /// Creates a new bucket ID.
    pub fn new_bid(&mut self) -> Bid {
        self.id_allocator.new_bid().unwrap()
    }

    /// Creates a new vault ID.
    pub fn new_vid(&mut self) -> Vid {
        self.id_allocator.new_vid(self.transaction_hash()).unwrap()
    }

    /// Creates a new reference id.
    pub fn new_rid(&mut self) -> Rid {
        self.id_allocator.new_rid().unwrap()
    }

    /// Creates a new map id.
    pub fn new_mid(&mut self) -> Mid {
        self.id_allocator.new_mid(self.transaction_hash()).unwrap()
    }

    /// Commits changes to the underlying ledger.
    pub fn commit(&mut self) {
        for address in self.updated_packages.clone() {
            self.ledger
                .put_package(address, self.packages.get(&address).unwrap().clone());
        }

        for address in self.updated_components.clone() {
            self.ledger
                .put_component(address, self.components.get(&address).unwrap().clone());
        }

        for address in self.updated_resource_defs.clone() {
            self.ledger
                .put_resource_def(address, self.resource_defs.get(&address).unwrap().clone());
        }

        for mid in self.updated_lazy_maps.clone() {
            self.ledger
                .put_lazy_map(mid, self.lazy_maps.get(&mid).unwrap().clone());
        }

        for vid in self.updated_vaults.clone() {
            self.ledger
                .put_vault(vid, self.vaults.get(&vid).unwrap().clone());
        }

        for (resource_def, id) in self.updated_nfts.clone() {
            self.ledger.put_nft(
                resource_def,
                id,
                self.nfts.get(&(resource_def, id)).unwrap().clone(),
            );
        }
    }
}
