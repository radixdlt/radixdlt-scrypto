use sbor::*;
use scrypto::kernel::*;
use scrypto::rust::rc::Rc;
use scrypto::types::*;

/// Represents an error when accessing a bucket.
#[derive(Debug, Clone)]
pub enum BucketError {
    MismatchingResourceDef,
    InsufficientBalance,
    InvalidGranularity,
    GranularityCheckFailed,
    NegativeAmount,
}

/// A transient resource container.
#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct Bucket {
    amount: Decimal,
    resource_def: Address,
    resource_type: ResourceType,
}

/// A bucket becomes locked after a borrow operation.
#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct LockedBucket {
    bucket_id: Bid,
    bucket: Bucket,
}

/// A reference to a bucket.
pub type BucketRef = Rc<LockedBucket>;

impl Bucket {
    pub fn new(amount: Decimal, resource_def: Address, resource_type: ResourceType) -> Self {
        Self {
            amount,
            resource_def,
            resource_type,
        }
    }

    pub fn put(&mut self, other: Self) -> Result<(), BucketError> {
        if self.resource_def != other.resource_def {
            Err(BucketError::MismatchingResourceDef)
        } else {
            self.amount += other.amount;
            Ok(())
        }
    }

    pub fn take(&mut self, amount: Decimal) -> Result<Self, BucketError> {
        Self::check_amount(&amount, &self.resource_type)?;

        if self.amount < amount {
            Err(BucketError::InsufficientBalance)
        } else {
            self.amount -= amount;

            Ok(Self::new(amount, self.resource_def, self.resource_type))
        }
    }

    pub fn amount(&self) -> Decimal {
        self.amount
    }

    pub fn resource_def(&self) -> Address {
        self.resource_def
    }

    fn check_amount(amount: &Decimal, resource_type: &ResourceType) -> Result<(), BucketError> {
        if amount.is_negative() {
            return Err(BucketError::NegativeAmount);
        }

        let granularity = match resource_type {
            ResourceType::Fungible { granularity } => *granularity,
            ResourceType::NonFungible => 19,
        };

        if granularity >= 1 && granularity <= 36 {
            if amount.0 % 10i128.pow((granularity - 1).into()) != 0.into() {
                Err(BucketError::GranularityCheckFailed)
            } else {
                Ok(())
            }
        } else {
            Err(BucketError::InvalidGranularity)
        }
    }
}

impl LockedBucket {
    pub fn new(bucket_id: Bid, bucket: Bucket) -> Self {
        Self { bucket_id, bucket }
    }

    pub fn bucket_id(&self) -> Bid {
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
