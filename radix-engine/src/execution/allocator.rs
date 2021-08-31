use scrypto::rust::vec::Vec;
use scrypto::types::*;
use scrypto::utils::*;

pub struct AddressAllocator {
    count: u32,
}

impl AddressAllocator {
    pub fn new() -> Self {
        Self { count: 0 }
    }

    pub fn count(&self) -> u32 {
        self.count
    }

    pub fn new_package_address(&mut self, tx_hash: H256) -> Address {
        let mut data = tx_hash.as_ref().to_vec();
        data.extend(self.count.to_le_bytes());

        let hash = sha256_twice(data);
        Address::Package(hash.lower_26_bytes())
    }

    pub fn new_component_address(&mut self, tx_hash: H256) -> Address {
        let mut data = tx_hash.as_ref().to_vec();
        data.extend(self.count.to_le_bytes());

        let hash = sha256_twice(data);
        Address::Component(hash.lower_26_bytes())
    }

    pub fn new_resource_address(&self, creator: Address, symbol: &str) -> Address {
        let mut data: Vec<u8> = creator.to_vec();
        data.extend(symbol.as_bytes());

        let hash = sha256_twice(data);
        Address::Resource(hash.lower_26_bytes())
    }

    pub fn new_transient_bid(&mut self) -> BID {
        self.count += 1;
        BID::Transient(self.count - 1)
    }

    pub fn new_persisted_bid(&mut self, tx_hash: H256) -> BID {
        self.count += 1;
        BID::Persisted(tx_hash, self.count - 1)
    }

    pub fn new_fixed_rid(&mut self) -> RID {
        self.count += 1;
        RID::Immutable(self.count)
    }

    pub fn new_mid(&mut self, tx_hash: H256) -> MID {
        self.count += 1;
        MID(tx_hash, self.count - 1)
    }
}
