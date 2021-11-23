use sbor::*;
use scrypto::kernel::*;
use scrypto::rust::collections::BTreeSet;
use scrypto::rust::rc::Rc;
use scrypto::rust::string::ToString;
use scrypto::types::*;

/// Represents an error when accessing a bucket.
#[derive(Debug, Clone)]
pub enum BucketError {
    MismatchingResourceDef,
    InsufficientBalance,
    InvalidGranularity,
    GranularityCheckFailed,
    UnsupportedOperation,
    NftNotFound,
    InvalidAmount(Decimal),
}

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub enum Supply {
    Fungible { amount: Decimal },

    NonFungible { entries: BTreeSet<u128> },
}

/// A transient resource container.
#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct Bucket {
    resource_def: Address,
    resource_type: ResourceType,
    supply: Supply,
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
    pub fn new(resource_def: Address, resource_type: ResourceType, supply: Supply) -> Self {
        Self {
            resource_def,
            resource_type,
            supply,
        }
    }

    pub fn put(&mut self, other: Self) -> Result<(), BucketError> {
        if self.resource_def != other.resource_def {
            Err(BucketError::MismatchingResourceDef)
        } else {
            match &mut self.supply {
                Supply::Fungible { ref mut amount } => {
                    let other_amount = match other.supply() {
                        Supply::Fungible { amount } => amount,
                        Supply::NonFungible { .. } => {
                            return Err(BucketError::UnsupportedOperation);
                        }
                    };
                    *amount = *amount + other_amount;
                }
                Supply::NonFungible { ref mut entries } => {
                    let other_entries = match other.supply() {
                        Supply::Fungible { .. } => {
                            return Err(BucketError::UnsupportedOperation);
                        }
                        Supply::NonFungible { entries } => entries,
                    };
                    entries.extend(other_entries);
                }
            }
            Ok(())
        }
    }

    pub fn take(&mut self, quantity: Decimal) -> Result<Self, BucketError> {
        Self::check_amount(quantity, &self.resource_type)?;

        if self.amount() < quantity {
            Err(BucketError::InsufficientBalance)
        } else {
            match &mut self.supply {
                Supply::Fungible { amount } => {
                    self.supply = Supply::Fungible {
                        amount: *amount - quantity,
                    };
                    Ok(Self::new(
                        self.resource_def,
                        self.resource_type,
                        Supply::Fungible { amount: quantity },
                    ))
                }
                Supply::NonFungible { ref mut entries } => {
                    let n: usize = quantity.to_string().parse().unwrap();
                    let taken: BTreeSet<u128> = entries.iter().cloned().take(n).collect();
                    for e in &taken {
                        entries.remove(e);
                    }
                    Ok(Self::new(
                        self.resource_def,
                        self.resource_type,
                        Supply::NonFungible { entries: taken },
                    ))
                }
            }
        }
    }

    pub fn take_nft(&mut self, id: u128) -> Result<Self, BucketError> {
        match &mut self.supply {
            Supply::Fungible { .. } => Err(BucketError::UnsupportedOperation),
            Supply::NonFungible { ref mut entries } => {
                if !entries.contains(&id) {
                    return Err(BucketError::NftNotFound);
                }
                entries.remove(&id);
                Ok(Self::new(
                    self.resource_def,
                    self.resource_type,
                    Supply::NonFungible {
                        entries: BTreeSet::from([id]),
                    },
                ))
            }
        }
    }

    pub fn get_nft_ids(&self) -> Result<BTreeSet<u128>, BucketError> {
        match &self.supply {
            Supply::Fungible { .. } => Err(BucketError::UnsupportedOperation),
            Supply::NonFungible { entries } => Ok(entries.iter().cloned().collect()),
        }
    }

    pub fn supply(&self) -> Supply {
        self.supply.clone()
    }

    pub fn amount(&self) -> Decimal {
        match &self.supply {
            Supply::Fungible { amount } => *amount,
            Supply::NonFungible { entries } => entries.len().into(),
        }
    }

    pub fn resource_def(&self) -> Address {
        self.resource_def
    }

    fn check_amount(amount: Decimal, resource_type: &ResourceType) -> Result<(), BucketError> {
        if amount.is_negative() {
            return Err(BucketError::InvalidAmount(amount));
        }

        let granularity = match resource_type {
            ResourceType::Fungible { granularity } => *granularity,
            ResourceType::NonFungible => 19,
        };

        if granularity >= 1 && granularity <= 36 {
            if amount.0 % 10i128.pow((granularity - 1).into()) != 0.into() {
                Err(BucketError::InvalidAmount(amount))
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
