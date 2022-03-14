use sbor::*;
use scrypto::engine::types::*;
use scrypto::rust::collections::BTreeMap;
use scrypto::rust::collections::BTreeSet;
use scrypto::rust::collections::HashMap;
use scrypto::rust::rc::Rc;
use scrypto::rust::string::ToString;

/// Represents an error when accessing a bucket.
#[derive(Debug, Clone)]
pub enum BucketError {
    ResourceNotMatching,
    InsufficientBalance,
    InvalidAmount(Decimal),
    UnsupportedOperation,
    NonFungibleNotFound,
    ResourceLocked,
    NotNonFungible,
}

/// Represents the contained resource.
#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub enum Resource {
    Fungible {
        /// The locked amounts and the corresponding times of being locked.
        locked_amounts: BTreeMap<Decimal, usize>,
        /// The liquid amount.
        liquid_amount: Decimal,
    },
    NonFungible {
        /// The locked non-fungible ids and the corresponding times of being locked.
        locked_ids: HashMap<NonFungibleId, usize>,
        /// The liquid non-fungible ids.
        liquid_ids: BTreeSet<NonFungibleId>,
    },
}

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub enum ResourceAmount {
    /// Fungible amount
    Fungible { amount: Decimal },
    /// Non-fungible amount
    NonFungible { ids: BTreeSet<NonFungibleId> },
}

/// A transient resource container.
#[derive(Debug, TypeId, Encode, Decode)]
pub struct Bucket {
    /// The resource definition id
    resource_def_id: ResourceDefId,
    /// The resource type
    resource_type: ResourceType,
    /// The contained resource
    resource: Resource,
}

#[derive(Debug, Clone)]
pub struct Proof {
    /// The resource definition id
    resource_def_id: ResourceDefId,
    /// The resource type
    resource_type: ResourceType,
    /// Restricted proof can't be moved down along the call stack (growing down).
    restricted: bool,
    /// The total amount locked
    total_amount: ResourceAmount,
    /// The sub-amounts (to be extended)
    #[allow(dead_code)]
    amounts: (Rc<Bucket>, ResourceAmount),
}

impl Resource {
    pub fn fungible(amount: Decimal) -> Self {
        Resource::Fungible {
            locked_amounts: BTreeMap::new(),
            liquid_amount: amount,
        }
    }

    pub fn non_fungible(ids: BTreeSet<NonFungibleId>) -> Self {
        Resource::NonFungible {
            locked_ids: HashMap::new(),
            liquid_ids: ids.clone(),
        }
    }
}

impl ResourceAmount {
    pub fn quantity(&self) -> Decimal {
        match self {
            ResourceAmount::Fungible { amount } => *amount,
            ResourceAmount::NonFungible { ids } => ids.len().into(),
        }
    }

    pub fn non_fungible_ids(&self) -> Result<BTreeSet<NonFungibleId>, ()> {
        match self {
            ResourceAmount::Fungible { .. } => Err(()),
            ResourceAmount::NonFungible { ids } => Ok(ids.clone()),
        }
    }
}

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
        // check resource address
        if self.resource_def_id != other.resource_def_id {
            return Err(BucketError::ResourceNotMatching);
        }

        // check locking status
        if other.is_locked() {
            return Err(BucketError::ResourceLocked);
        }

        // add the other bucket into liquid pool
        match (&mut self.resource, other.liquid_amount()) {
            (Resource::Fungible { liquid_amount, .. }, ResourceAmount::Fungible { amount }) => {
                *liquid_amount = *liquid_amount + amount;
            }
            (Resource::NonFungible { liquid_ids, .. }, ResourceAmount::NonFungible { ids }) => {
                liquid_ids.extend(ids);
            }
            _ => panic!("Resource type should match!"),
        }
        Ok(())
    }

    pub fn take(&mut self, quantity: Decimal) -> Result<Self, BucketError> {
        // check amount granularity
        Self::check_amount(quantity, self.resource_type.divisibility())?;

        // check balance
        if self.liquid_amount().quantity() < quantity {
            return Err(BucketError::InsufficientBalance);
        }

        // deduct from liquidity pool
        match &mut self.resource {
            Resource::Fungible { liquid_amount, .. } => {
                *liquid_amount = *liquid_amount - quantity;
                Ok(Self::new(
                    self.resource_def_id,
                    self.resource_type,
                    Resource::fungible(quantity),
                ))
            }
            Resource::NonFungible { liquid_ids, .. } => {
                let n: usize = quantity.to_string().parse().unwrap();
                let taken: BTreeSet<NonFungibleId> = liquid_ids.iter().cloned().take(n).collect();
                taken.iter().for_each(|key| {
                    liquid_ids.remove(key);
                });
                Ok(Self::new(
                    self.resource_def_id,
                    self.resource_type,
                    Resource::non_fungible(taken),
                ))
            }
        }
    }

    pub fn take_non_fungible(&mut self, key: &NonFungibleId) -> Result<Self, BucketError> {
        self.take_non_fungibles(&BTreeSet::from([key.clone()]))
    }

    pub fn take_non_fungibles(
        &mut self,
        ids: &BTreeSet<NonFungibleId>,
    ) -> Result<Self, BucketError> {
        match &mut self.resource {
            Resource::Fungible { .. } => Err(BucketError::UnsupportedOperation),
            Resource::NonFungible { liquid_ids, .. } => {
                for key in ids {
                    if !liquid_ids.remove(&key) {
                        return Err(BucketError::NonFungibleNotFound);
                    }
                }
                Ok(Self::new(
                    self.resource_def_id,
                    self.resource_type,
                    Resource::non_fungible(ids.clone()),
                ))
            }
        }
    }

    pub fn liquid_amount(&self) -> ResourceAmount {
        match &self.resource {
            Resource::Fungible { liquid_amount, .. } => ResourceAmount::Fungible {
                amount: liquid_amount.clone(),
            },
            Resource::NonFungible { liquid_ids, .. } => ResourceAmount::NonFungible {
                ids: liquid_ids.clone(),
            },
        }
    }

    pub fn is_locked(&self) -> bool {
        match &self.resource {
            Resource::Fungible { locked_amounts, .. } => !locked_amounts.is_empty(),
            Resource::NonFungible { locked_ids, .. } => !locked_ids.is_empty(),
        }
    }

    pub fn resource_def_id(&self) -> ResourceDefId {
        self.resource_def_id
    }

    pub fn resource_type(&self) -> ResourceType {
        self.resource_type
    }

    fn check_amount(amount: Decimal, divisibility: u8) -> Result<(), BucketError> {
        if !amount.is_negative() && amount.0 % 10i128.pow((18 - divisibility).into()) != 0.into() {
            Err(BucketError::InvalidAmount(amount))
        } else {
            Ok(())
        }
    }
}

impl Proof {
    pub fn from_bucket(bucket: Rc<Bucket>) -> Self {
        let amount = bucket.liquid_amount();
        Self {
            resource_def_id: bucket.resource_def_id(),
            resource_type: bucket.resource_type(),
            restricted: false,
            total_amount: amount.clone(),
            amounts: (bucket, amount),
        }
    }

    pub fn resource_def_id(&self) -> ResourceDefId {
        self.resource_def_id
    }
    pub fn resource_type(&self) -> ResourceType {
        self.resource_type
    }
    pub fn total_amount(&self) -> ResourceAmount {
        self.total_amount.clone()
    }

    pub fn is_restricted(&self) -> bool {
        self.restricted
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_take_fungible() {}

    #[test]
    fn test_take_non_fungible() {}

    #[test]
    fn test_put() {}

    #[test]
    fn test_put_wrong_resource() {}

    #[test]
    fn test_put_locked_bucket() {}

    #[test]
    fn test_generate_proof() {}
}
