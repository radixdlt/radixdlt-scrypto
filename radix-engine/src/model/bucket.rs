use sbor::*;
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
    granularity: u8,
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
    pub fn new(amount: Decimal, resource_def: Address, granularity: u8) -> Self {
        Self {
            amount,
            resource_def,
            granularity,
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
        Self::check_amount(&amount, self.granularity)?;

        if self.amount < amount {
            Err(BucketError::InsufficientBalance)
        } else {
            self.amount -= amount;

            Ok(Self::new(amount, self.resource_def, self.granularity))
        }
    }

    pub fn amount(&self) -> Decimal {
        self.amount
    }

    pub fn resource_def(&self) -> Address {
        self.resource_def
    }

    fn check_amount(amount: &Decimal, granularity: u8) -> Result<(), BucketError> {
        if amount.is_negative() {
            return Err(BucketError::NegativeAmount);
        }

        match granularity {
            1 => Ok(()),
            18 => {
                if amount.0 % 10i128.pow(18) != 0.into() {
                    Err(BucketError::GranularityCheckFailed)
                } else {
                    Ok(())
                }
            }
            _ => Err(BucketError::InvalidGranularity),
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
