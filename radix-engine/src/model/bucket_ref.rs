use scrypto::types::*;

#[derive(Debug)]
pub struct BucketRef {
    amount: U256,
    resource: Address,
    count: u32,
}

impl BucketRef {
    pub fn new(amount: U256, resource: Address, count: u32) -> Self {
        Self {
            amount,
            resource,
            count,
        }
    }

    pub fn increase_count(&mut self) {
        self.count += 1;
    }

    pub fn decrease_count(&mut self) {
        assert!(self.count() > 0, "Reference count can't go negative");
        self.count -= 1;
    }

    pub fn amount(&self) -> U256 {
        self.amount
    }

    pub fn resource(&self) -> Address {
        self.resource
    }

    pub fn count(&self) -> u32 {
        self.count
    }
}
