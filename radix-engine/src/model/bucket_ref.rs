use scrypto::types::*;

#[derive(Debug)]
pub struct BucketRef {
    amount: U256,
    resource: Address,
    count: usize,
}

impl BucketRef {
    pub fn new(amount: U256, resource: Address, count: usize) -> Self {
        Self {
            amount,
            resource,
            count,
        }
    }

    pub fn increase_count(&mut self) -> usize {
        self.count += 1;
        self.count
    }

    pub fn decrease_count(&mut self) -> usize {
        assert!(self.count() > 0, "Reference count can't go negative");
        self.count -= 1;
        self.count
    }

    pub fn amount(&self) -> U256 {
        self.amount
    }

    pub fn resource(&self) -> Address {
        self.resource
    }

    pub fn count(&self) -> usize {
        self.count
    }
}
