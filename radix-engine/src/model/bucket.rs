use sbor::*;
use scrypto::engine::types::*;
use scrypto::rust::collections::BTreeSet;
use scrypto::rust::rc::Rc;
use scrypto::rust::string::ToString;
use scrypto::rust::vec::Vec;

/// Represents an error when accessing a bucket.
#[derive(Debug, Clone, PartialEq)]
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

    NonFungible { ids: BTreeSet<NonFungibleId> },
}

/// A transient resource container.
#[derive(Debug, TypeId, Encode, Decode)]
pub struct Bucket {
    resource_def_id: ResourceDefId,
    resource_type: ResourceType,
    resource: Resource,
}

/// A bucket becomes locked after a borrow operation.
#[derive(Debug, TypeId, Encode, Decode)]
pub struct LockedBucket {
    bucket_id: BucketId,
    bucket: Bucket,
}

/// A bucket proof
pub type Proof = Rc<LockedBucket>;

impl Bucket {
    pub fn new(
        resource_def_id: ResourceDefId,
        resource_type: ResourceType,
        resource: Resource,
    ) -> Self {
        Self {
            resource_def_id,
            resource_type,
            resource,
        }
    }

    pub fn put(&mut self, other: Self) -> Result<(), BucketError> {
        if self.resource_def_id != other.resource_def_id {
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
                Resource::NonFungible { ref mut ids } => {
                    let other_ids = match other.resource() {
                        Resource::Fungible { .. } => {
                            panic!("Illegal state!")
                        }
                        Resource::NonFungible { ids } => ids,
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
            match &mut self.resource {
                Resource::Fungible { amount } => {
                    self.resource = Resource::Fungible {
                        amount: *amount - quantity,
                    };
                    Ok(Self::new(
                        self.resource_def_id,
                        self.resource_type,
                        Resource::Fungible { amount: quantity },
                    ))
                }
                Resource::NonFungible { ref mut ids } => {
                    let n: usize = quantity.to_string().parse().unwrap();
                    let taken: BTreeSet<NonFungibleId> = ids.iter().cloned().take(n).collect();
                    for e in &taken {
                        ids.remove(e);
                    }
                    Ok(Self::new(
                        self.resource_def_id,
                        self.resource_type,
                        Resource::NonFungible { ids: taken },
                    ))
                }
            }
        }
    }

    pub fn take_non_fungible(&mut self, key: &NonFungibleId) -> Result<Self, BucketError> {
        self.take_non_fungibles(&BTreeSet::from([key.clone()]))
    }

    pub fn take_non_fungibles(
        &mut self,
        set: &BTreeSet<NonFungibleId>,
    ) -> Result<Self, BucketError> {
        match &mut self.resource {
            Resource::Fungible { .. } => Err(BucketError::UnsupportedOperation),
            Resource::NonFungible { ref mut ids } => {
                for id in set {
                    if !ids.remove(&id) {
                        return Err(BucketError::NonFungibleNotFound);
                    }
                }
                Ok(Self::new(
                    self.resource_def_id,
                    self.resource_type,
                    Resource::NonFungible { ids: set.clone() },
                ))
            }
        }
    }

    pub fn get_non_fungible_ids(&self) -> Result<Vec<NonFungibleId>, BucketError> {
        match &self.resource {
            Resource::Fungible { .. } => Err(BucketError::UnsupportedOperation),
            Resource::NonFungible { ids } => Ok(ids.iter().cloned().collect()),
        }
    }

    pub fn resource(&self) -> Resource {
        self.resource.clone()
    }

    pub fn amount(&self) -> Decimal {
        match &self.resource {
            Resource::Fungible { amount } => *amount,
            Resource::NonFungible { ids } => ids.len().into(),
        }
    }

    pub fn resource_def_id(&self) -> ResourceDefId {
        self.resource_def_id
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
