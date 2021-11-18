use sbor::*;
use scrypto::kernel::*;
use scrypto::rust::collections::BTreeMap;
use scrypto::rust::collections::BTreeSet;
use scrypto::rust::rc::Rc;
use scrypto::rust::string::ToString;
use scrypto::rust::vec::Vec;
use scrypto::types::*;

/// Represents an error when accessing a bucket.
#[derive(Debug, Clone)]
pub enum BucketError {
    MismatchingResourceDef,
    InsufficientBalance,
    InvalidGranularity,
    GranularityCheckFailed,
    NegativeAmount,
    UnsupportedOperation,
    NftNotFound,
}

/// A transient resource container.
#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct Bucket {
    resource_def: Address,
    resource_type: ResourceType,
    supply: ResourceSupply,
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
    pub fn new(resource_def: Address, resource_type: ResourceType, supply: ResourceSupply) -> Self {
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
                ResourceSupply::Fungible { amount } => {
                    self.supply = ResourceSupply::Fungible {
                        amount: *amount + other.amount(),
                    };
                }
                ResourceSupply::NonFungible { ref mut entries } => {
                    entries.extend(other.entries()?);
                }
            }
            Ok(())
        }
    }

    pub fn take(&mut self, amount_to_withdraw: Decimal) -> Result<Self, BucketError> {
        Self::check_amount(amount_to_withdraw, &self.resource_type)?;

        if self.amount() < amount_to_withdraw {
            Err(BucketError::InsufficientBalance)
        } else {
            match &mut self.supply {
                ResourceSupply::Fungible { amount } => {
                    self.supply = ResourceSupply::Fungible {
                        amount: *amount - amount_to_withdraw,
                    };
                    Ok(Self::new(
                        self.resource_def,
                        self.resource_type,
                        ResourceSupply::Fungible {
                            amount: amount_to_withdraw,
                        },
                    ))
                }
                ResourceSupply::NonFungible { ref mut entries } => {
                    let mut to_return = BTreeMap::new();
                    let n: usize = amount_to_withdraw.to_string().parse().unwrap();
                    for _ in 0..n {
                        // pop_first() once it's stable
                        let first_key = entries.keys().next().unwrap().clone();
                        let first_value = entries.remove(&first_key).unwrap();
                        to_return.insert(first_key, first_value);
                    }
                    Ok(Self::new(
                        self.resource_def,
                        self.resource_type,
                        ResourceSupply::NonFungible { entries: to_return },
                    ))
                }
            }
        }
    }

    pub fn take_nft(&mut self, id: u64) -> Result<Self, BucketError> {
        match &mut self.supply {
            ResourceSupply::Fungible { .. } => Err(BucketError::UnsupportedOperation),
            ResourceSupply::NonFungible { ref mut entries } => {
                let nft = entries.remove(&id).ok_or(BucketError::NftNotFound)?;
                Ok(Self::new(
                    self.resource_def,
                    self.resource_type,
                    ResourceSupply::NonFungible {
                        entries: BTreeMap::from([(id, nft)]),
                    },
                ))
            }
        }
    }

    pub fn get_nft(&self, id: u64) -> Result<Vec<u8>, BucketError> {
        match &self.supply {
            ResourceSupply::Fungible { .. } => Err(BucketError::UnsupportedOperation),
            ResourceSupply::NonFungible { entries } => {
                let nft = entries.get(&id).ok_or(BucketError::NftNotFound)?;
                Ok(nft.clone())
            }
        }
    }

    pub fn get_nft_ids(&self) -> Result<BTreeSet<u64>, BucketError> {
        match &self.supply {
            ResourceSupply::Fungible { .. } => Err(BucketError::UnsupportedOperation),
            ResourceSupply::NonFungible { entries } => {
                Ok(entries.keys().into_iter().map(Clone::clone).collect())
            }
        }
    }

    pub fn supply(&self) -> ResourceSupply {
        self.supply.clone()
    }

    pub fn amount(&self) -> Decimal {
        match &self.supply {
            ResourceSupply::Fungible { amount } => *amount,
            ResourceSupply::NonFungible { entries } => entries.len().into(),
        }
    }

    pub fn entries(&self) -> Result<BTreeMap<u64, Vec<u8>>, BucketError> {
        match &self.supply {
            ResourceSupply::Fungible { .. } => Err(BucketError::UnsupportedOperation),
            ResourceSupply::NonFungible { entries } => Ok(entries.clone()),
        }
    }

    pub fn resource_def(&self) -> Address {
        self.resource_def
    }

    fn check_amount(amount: Decimal, resource_type: &ResourceType) -> Result<(), BucketError> {
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
