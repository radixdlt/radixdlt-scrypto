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
    NonFungibleNotFound,
}

/// Represents the supply of resource.
#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub enum Supply {
    Fungible { amount: Decimal },

    NonFungible { keys: BTreeSet<NonFungibleKey> },
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
                Supply::NonFungible { ref mut keys } => {
                    let other_keys = match other.supply() {
                        Supply::Fungible { .. } => {
                            panic!("Illegal state!")
                        }
                        Supply::NonFungible { keys } => keys,
                    };
                    keys.extend(other_keys);
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
                Supply::NonFungible { ref mut keys } => {
                    let n: usize = quantity.to_string().parse().unwrap();
                    let taken: BTreeSet<NonFungibleKey> = keys.iter().cloned().take(n).collect();
                    for e in &taken {
                        keys.remove(e);
                    }
                    Ok(Self::new(
                        self.resource_address,
                        self.resource_type,
                        Supply::NonFungible { keys: taken },
                    ))
                }
            }
        }
    }

    pub fn take_non_fungible(&mut self, key: &NonFungibleKey) -> Result<Self, BucketError> {
        match &mut self.supply {
            Supply::Fungible { .. } => Err(BucketError::UnsupportedOperation),
            Supply::NonFungible { ref mut keys } => {
                if !keys.contains(&key) {
                    return Err(BucketError::NonFungibleNotFound);
                }
                keys.remove(&key);
                Ok(Self::new(
                    self.resource_address,
                    self.resource_type,
                    Supply::NonFungible {
                        keys: BTreeSet::from([key.clone()]),
                    },
                ))
            }
        }
    }

    pub fn get_non_fungible_keys(&self) -> Result<Vec<NonFungibleKey>, BucketError> {
        match &self.supply {
            Supply::Fungible { .. } => Err(BucketError::UnsupportedOperation),
            Supply::NonFungible { keys } => Ok(keys.iter().cloned().collect()),
        }
    }

    pub fn supply(&self) -> Supply {
        self.supply.clone()
    }

    pub fn amount(&self) -> Decimal {
        match &self.supply {
            Supply::Fungible { amount } => *amount,
            Supply::NonFungible { keys } => keys.len().into(),
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
