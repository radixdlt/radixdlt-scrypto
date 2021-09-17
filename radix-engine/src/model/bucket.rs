use sbor::*;
use scrypto::rust::rc::Rc;
use scrypto::types::*;

/// Represents an error when accessing a bucket.
#[derive(Debug, Clone)]
pub enum BucketError {
    MismatchingResourceType,
    InsufficientBalance,
    ReferenceCountUnderflow,
    UnauthorizedAccess,
}

/// A bucket is a container that holds resources.
#[derive(Debug, Clone, Encode, Decode)]
pub struct Bucket {
    amount: U256,
    resource: Address,
}

/// When a bucket gets borrowed, it becomes unlocked immediately
/// until all references have been dropped.
#[derive(Debug, Clone, Encode, Decode)]
pub struct LockedBucket {
    bucket_id: BID,
    bucket: Bucket,
}

/// A reference to a bucket.
pub type BucketRef = Rc<LockedBucket>;

/// A persisted bucket is stored permanently on ledger state.
#[derive(Debug, Clone, Encode, Decode)]
pub struct PersistentBucket {
    bucket: Bucket,
    owner: Address,
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

impl LockedBucket {
    pub fn new(bucket_id: BID, bucket: Bucket) -> Self {
        Self { bucket_id, bucket }
    }

    pub fn bucket_id(&self) -> BID {
        self.bucket_id
    }

    pub fn bucket(&self) -> &Bucket {
        &self.bucket
    }
}

impl From<LockedBucket> for Bucket {
    fn from(b: LockedBucket) -> Self {
        b.bucket
    }
}

impl PersistentBucket {
    pub fn new(bucket: Bucket, owner: Address) -> Self {
        Self { bucket, owner }
    }

    pub fn put(&mut self, other: Bucket, requester: Address) -> Result<(), BucketError> {
        if requester == self.owner {
            self.bucket.put(other)
        } else {
            Err(BucketError::UnauthorizedAccess)
        }
    }

    pub fn take(&mut self, amount: U256, requester: Address) -> Result<Bucket, BucketError> {
        if requester == self.owner {
            self.bucket.take(amount)
        } else {
            Err(BucketError::UnauthorizedAccess)
        }
    }

    pub fn amount(&self) -> U256 {
        self.bucket.amount()
    }

    pub fn resource(&self) -> Address {
        self.bucket.resource()
    }
}
