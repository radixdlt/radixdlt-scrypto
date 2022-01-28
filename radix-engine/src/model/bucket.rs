use sbor::*;
use scrypto::kernel::*;
use scrypto::rust::collections::BTreeSet;
use scrypto::rust::rc::Rc;
use scrypto::rust::string::ToString;
use scrypto::rust::vec::Vec;
use scrypto::types::*;

/// Represents an error when accessing a bucket.
#[derive(Debug, Clone)]
pub enum BucketError {
    ResourceNotMatching,
    InsufficientBalance,
    InvalidAmount(Decimal),
    UnsupportedOperation,
    NftNotFound,
}

/// Represents the supply of resource.
#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub enum Supply {
    Fungible { amount: Decimal },

    NonFungible { ids: BTreeSet<NftKey> },
}

/// A transient resource container.
#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct Bucket {
    resource_address: Address,
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
    pub fn new(resource_address: Address, resource_type: ResourceType, supply: Supply) -> Self {
        Self {
            resource_address,
            resource_type,
            supply,
        }
    }

    pub fn put(&mut self, other: Self) -> Result<(), BucketError> {
        if self.resource_address != other.resource_address {
            Err(BucketError::ResourceNotMatching)
        } else {
            match &mut self.supply {
                Supply::Fungible { ref mut amount } => {
                    let other_amount = match other.supply() {
                        Supply::Fungible { amount } => amount,
                        Supply::NonFungible { .. } => {
                            panic!("Illegal state!")
                        }
                    };
                    *amount = *amount + other_amount;
                }
                Supply::NonFungible { ref mut ids } => {
                    let other_ids = match other.supply() {
                        Supply::Fungible { .. } => {
                            panic!("Illegal state!")
                        }
                        Supply::NonFungible { ids } => ids,
                    };
                    ids.extend(other_ids);
                }
            }
            Ok(())
        }
    }

    pub fn take(&mut self, quantity: Decimal) -> Result<Self, BucketError> {
        Self::check_amount(quantity, self.resource_type.divisibility())?;

        if self.amount() < quantity {
            Err(BucketError::InsufficientBalance)
        } else {
            match &mut self.supply {
                Supply::Fungible { amount } => {
                    self.supply = Supply::Fungible {
                        amount: *amount - quantity,
                    };
                    Ok(Self::new(
                        self.resource_address,
                        self.resource_type,
                        Supply::Fungible { amount: quantity },
                    ))
                }
                Supply::NonFungible { ref mut ids } => {
                    let n: usize = quantity.to_string().parse().unwrap();
                    let taken: BTreeSet<NftKey> = ids.iter().cloned().take(n).collect();
                    for e in &taken {
                        ids.remove(e);
                    }
                    Ok(Self::new(
                        self.resource_address,
                        self.resource_type,
                        Supply::NonFungible { ids: taken },
                    ))
                }
            }
        }
    }

    pub fn take_nft(&mut self, id: NftKey) -> Result<Self, BucketError> {
        match &mut self.supply {
            Supply::Fungible { .. } => Err(BucketError::UnsupportedOperation),
            Supply::NonFungible { ref mut ids } => {
                if !ids.contains(&id) {
                    return Err(BucketError::NftNotFound);
                }
                ids.remove(&id);
                Ok(Self::new(
                    self.resource_address,
                    self.resource_type,
                    Supply::NonFungible {
                        ids: BTreeSet::from([id]),
                    },
                ))
            }
        }
    }

    pub fn get_nft_ids(&self) -> Result<Vec<NftKey>, BucketError> {
        match &self.supply {
            Supply::Fungible { .. } => Err(BucketError::UnsupportedOperation),
            Supply::NonFungible { ids } => Ok(ids.iter().cloned().collect()),
        }
    }

    pub fn supply(&self) -> Supply {
        self.supply.clone()
    }

    pub fn amount(&self) -> Decimal {
        match &self.supply {
            Supply::Fungible { amount } => *amount,
            Supply::NonFungible { ids } => ids.len().into(),
        }
    }

    pub fn resource_address(&self) -> Address {
        self.resource_address
    }

    fn check_amount(amount: Decimal, divisibility: u8) -> Result<(), BucketError> {
        if !amount.is_negative() && amount.0 % 10i128.pow((18 - divisibility).into()) != 0.into() {
            Err(BucketError::InvalidAmount(amount))
        } else {
            Ok(())
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
