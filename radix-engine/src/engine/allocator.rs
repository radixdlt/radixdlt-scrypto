use scrypto::types::*;
use scrypto::utils::*;

/// An address allocator generates new addresses and identities.
pub struct IdAllocator {
    count: u32,
}

impl IdAllocator {
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

    /// Creates a new resource definition address.
    pub fn new_resource_def_address(&mut self, tx_hash: H256) -> Address {
        let mut data = tx_hash.as_ref().to_vec();
        data.extend(self.count.to_le_bytes());
        self.count += 1;

        let hash = sha256_twice(data);
        Address::ResourceDef(hash.lower_26_bytes())
    }

    /// Creates a new bucket ID.
    pub fn new_bid(&mut self) -> Bid {
        self.count += 1;
        Bid(self.count - 1)
    }

    /// Creates a new vault ID.
    pub fn new_vid(&mut self, tx_hash: H256) -> Vid {
        self.count += 1;
        Vid(tx_hash, self.count - 1)
    }

    /// Creates a new bucket ref ID.
    pub fn new_rid(&mut self) -> Rid {
        self.count += 1;
        Rid(self.count - 1)
    }

    /// Creates a new lazy map ID.
    pub fn new_mid(&mut self, tx_hash: H256) -> Mid {
        self.count += 1;
        Mid(tx_hash, self.count - 1)
    }
}

impl Default for IdAllocator {
    fn default() -> Self {
        Self::new()
    }
}
