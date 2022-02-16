use sbor::*;
use scrypto::engine::types::*;
use scrypto::rust::collections::BTreeSet;
use scrypto::rust::rc::Rc;
use scrypto::rust::string::ToString;
use scrypto::rust::vec::Vec;

/// Represents an error when accessing a bucket.
#[derive(Debug, Clone)]
pub enum BucketError {
    ResourceNotMatching,
    InsufficientBalance,
    InvalidAmount(Decimal),
    UnsupportedOperation,
    NonFungibleNotFound,
}

/// Represents the contained resource.
#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub enum Resource {
    Fungible { amount: Decimal },

    NonFungible { keys: BTreeSet<NonFungibleKey> },
}

/// A transient resource container.
#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct Bucket {
    resource_def_ref: ResourceDefRef,
    resource_type: ResourceType,
    resource: Resource,
}

/// A bucket becomes locked after a borrow operation.
#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct LockedBucket {
    bucket_id: BucketId,
    bucket: Bucket,
}

/// A reference to a bucket.
pub type Proof = Rc<LockedBucket>;

impl Bucket {
    pub fn new(
        resource_def_ref: ResourceDefRef,
        resource_type: ResourceType,
        resource: Resource,
    ) -> Self {
        Self {
            resource_def_ref,
            resource_type,
            resource,
        }
    }

    pub fn put(&mut self, other: Self) -> Result<(), BucketError> {
        if self.resource_def_ref != other.resource_def_ref {
            Err(BucketError::ResourceNotMatching)
        } else {
            match &mut self.resource {
                Resource::Fungible { ref mut amount } => {
                    let other_amount = match other.resource() {
                        Resource::Fungible { amount } => amount,
                        Resource::NonFungible { .. } => {
                            panic!("Illegal state!")
                        }
                    };
                    *amount = *amount + other_amount;
                }
                Resource::NonFungible { ref mut keys } => {
                    let other_keys = match other.resource() {
                        Resource::Fungible { .. } => {
                            panic!("Illegal state!")
                        }
                        Resource::NonFungible { keys } => keys,
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
            match &mut self.resource {
                Resource::Fungible { amount } => {
                    self.resource = Resource::Fungible {
                        amount: *amount - quantity,
                    };
                    Ok(Self::new(
                        self.resource_def_ref,
                        self.resource_type,
                        Resource::Fungible { amount: quantity },
                    ))
                }
                Resource::NonFungible { ref mut keys } => {
                    let n: usize = quantity.to_string().parse().unwrap();
                    let taken: BTreeSet<NonFungibleKey> = keys.iter().cloned().take(n).collect();
                    for e in &taken {
                        keys.remove(e);
                    }
                    Ok(Self::new(
                        self.resource_def_ref,
                        self.resource_type,
                        Resource::NonFungible { keys: taken },
                    ))
                }
            }
        }
    }

    pub fn take_non_fungible(&mut self, key: &NonFungibleKey) -> Result<Self, BucketError> {
        self.take_non_fungibles(&BTreeSet::from([key.clone()]))
    }

    pub fn take_non_fungibles(
        &mut self,
        set: &BTreeSet<NonFungibleKey>,
    ) -> Result<Self, BucketError> {
        match &mut self.resource {
            Resource::Fungible { .. } => Err(BucketError::UnsupportedOperation),
            Resource::NonFungible { ref mut keys } => {
                for key in set {
                    if !keys.remove(&key) {
                        return Err(BucketError::NonFungibleNotFound);
                    }
                }
                Ok(Self::new(
                    self.resource_def_ref,
                    self.resource_type,
                    Resource::NonFungible { keys: set.clone() },
                ))
            }
        }
    }

    pub fn get_non_fungible_keys(&self) -> Result<Vec<NonFungibleKey>, BucketError> {
        match &self.resource {
            Resource::Fungible { .. } => Err(BucketError::UnsupportedOperation),
            Resource::NonFungible { keys } => Ok(keys.iter().cloned().collect()),
        }
    }

    pub fn resource(&self) -> Resource {
        self.resource.clone()
    }

    pub fn amount(&self) -> Decimal {
        match &self.resource {
            Resource::Fungible { amount } => *amount,
            Resource::NonFungible { keys } => keys.len().into(),
        }
    }

    pub fn resource_def_ref(&self) -> ResourceDefRef {
        self.resource_def_ref
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
    pub fn new(bucket_id: BucketId, bucket: Bucket) -> Self {
        Self { bucket_id, bucket }
    }

    pub fn bucket_id(&self) -> BucketId {
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
