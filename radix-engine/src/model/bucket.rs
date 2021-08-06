use sbor::*;
use scrypto::types::*;

#[derive(Debug, Clone, Encode, Decode)]
pub struct Bucket {
    amount: U256,
    resource: Address,
}

impl Bucket {
    pub fn new(amount: U256, resource: Address) -> Self {
        Self { amount, resource }
    }

    pub fn put(&mut self, other: Self) {
        assert_eq!(self.resource, other.resource, "Mismatching resource types");

        self.amount += other.amount;
    }

    pub fn take(&mut self, amount: U256) -> Self {
        assert!(self.amount >= amount, "Insufficient balance");

        self.amount -= amount;

        Self::new(amount, self.resource)
    }

    pub fn amount(&self) -> U256 {
        self.amount
    }

    pub fn resource(&self) -> Address {
        self.resource
    }
}
