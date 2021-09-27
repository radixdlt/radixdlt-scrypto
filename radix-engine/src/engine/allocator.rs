use scrypto::types::*;
use scrypto::utils::*;

/// An address allocator generates new addresses and identities.
pub struct AddressAllocator {
    count: u32,
}

impl AddressAllocator {
    /// Creates an address allocator.
    pub fn new() -> Self {
        Self { count: 0 }
    }

    /// Returns the number of addresses that have been generated.
    pub fn count(&self) -> u32 {
        self.count
    }

    /// Resets this allocator.
    pub fn reset(&mut self) {
        self.count = 0;
    }

    /// Creates a new package address.
    pub fn new_package_address(&mut self, tx_hash: H256) -> Address {
        let mut data = tx_hash.as_ref().to_vec();
        data.extend(self.count.to_le_bytes());
        self.count += 1;

        let hash = sha256_twice(data);
        Address::Package(hash.lower_26_bytes())
    }

    /// Creates a new component address.
    pub fn new_component_address(&mut self, tx_hash: H256) -> Address {
        let mut data = tx_hash.as_ref().to_vec();
        data.extend(self.count.to_le_bytes());
        self.count += 1;

        let hash = sha256_twice(data);
        Address::Component(hash.lower_26_bytes())
    }

    /// Creates a new resource address.
    pub fn new_resource_address(&mut self, tx_hash: H256) -> Address {
        let mut data = tx_hash.as_ref().to_vec();
        data.extend(self.count.to_le_bytes());
        self.count += 1;

        let hash = sha256_twice(data);
        Address::ResourceDef(hash.lower_26_bytes())
    }

    /// Creates a new bucket ID.
    pub fn new_bucket_id(&mut self) -> BID {
        self.count += 1;
        BID(self.count - 1)
    }

    /// Creates a new vault ID.
    pub fn new_vault_id(&mut self, tx_hash: H256) -> VID {
        self.count += 1;
        VID(tx_hash, self.count - 1)
    }

    /// Creates a new reference ID.
    pub fn new_rid(&mut self) -> RID {
        self.count += 1;
        RID(self.count)
    }

    /// Creates a new lazy map id.
    pub fn new_mid(&mut self, tx_hash: H256) -> MID {
        self.count += 1;
        MID(tx_hash, self.count - 1)
    }
}

impl Default for AddressAllocator {
    fn default() -> Self {
        Self::new()
    }
}
