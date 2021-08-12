use sbor::*;
use scrypto::types::*;

/// A bucket represents a certain amount of resource.
#[derive(Debug, Clone, Encode, Decode)]
pub struct Bucket {
    amount: U256,
    resource: Address,
}

/// Represents an error when accessing a bucket.
#[derive(Debug, Clone)]
pub enum BucketError {
    MismatchingResourceType,
    InsufficientBalance,
    ReferenceCountUnderflow,
}

impl Bucket {
    pub fn new(amount: U256, resource: Address) -> Self {
        Self { amount, resource }
    }

    pub fn put(&mut self, other: Self) -> Result<(), BucketError> {
        if self.resource != other.resource {
            Err(BucketError::MismatchingResourceType)
        } else {
            self.amount += other.amount;
            Ok(())
        }
    }

    pub fn take(&mut self, amount: U256) -> Result<Self, BucketError> {
        if self.amount < amount {
            Err(BucketError::InsufficientBalance)
        } else {
            self.amount -= amount;

            Ok(Self::new(amount, self.resource))
        }
    }

    pub fn amount(&self) -> U256 {
        self.amount
    }

    pub fn resource(&self) -> Address {
        self.resource
    }
}

/// Contain a borrowed bucket and record the number of references out.
#[derive(Debug)]
pub struct BucketBorrowed {
    bid: BID,
    bucket: Bucket,
    count: usize,
}

impl BucketBorrowed {
    pub fn new(bid: BID, bucket: Bucket, count: usize) -> Self {
        Self { bid, bucket, count }
    }

    pub fn brw(&mut self) {
        self.count += 1;
    }

    pub fn rtn(&mut self) -> Result<usize, BucketError> {
        if self.ref_count() <= 0 {
            Err(BucketError::ReferenceCountUnderflow)
        } else {
            self.count -= 1;
            Ok(self.count)
        }
    }

    pub fn bid(&self) -> BID {
        self.bid
    }

    pub fn bucket(&self) -> &Bucket {
        &self.bucket
    }

    pub fn ref_count(&self) -> usize {
        self.count
    }
}

impl Into<Bucket> for BucketBorrowed {
    fn into(self) -> Bucket {
        self.bucket
    }
}
