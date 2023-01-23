use crate::types::*;
use radix_engine_interface::api::types::{BucketId, VaultId};
use radix_engine_interface::model::*;

#[derive(Debug, Clone, PartialEq, Eq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub enum ResourceOperationError {
    /// Resource addresses do not match.
    ResourceAddressNotMatching,
    /// The amount is invalid, according to the resource divisibility.
    InvalidAmount(Decimal, u8),
    /// The balance is not enough.
    InsufficientBalance,
    /// Fungible operation on non-fungible resource is not allowed.
    FungibleOperationNotAllowed,
    /// Non-fungible operation on fungible resource is not allowed.
    NonFungibleOperationNotAllowed,
    /// Resource is locked because of proofs
    ResourceLocked,
    /// Non-fungible resource id type is not matching this resource id type.
    NonFungibleIdTypeNotMatching,
}

/// A raw record of resource persisted in the substate store
#[derive(Debug, Clone, PartialEq, Eq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub enum Resource {
    Fungible {
        /// The resource address.
        resource_address: ResourceAddress,
        /// The resource divisibility.
        divisibility: u8,
        /// The total amount.
        amount: Decimal,
    },
    NonFungible {
        /// The resource address.
        resource_address: ResourceAddress,
        /// The total non-fungible ids.
        ids: BTreeSet<NonFungibleLocalId>,
        /// NonFungible Id type
        id_type: NonFungibleIdType,
    },
}

impl Resource {
    pub fn new_fungible(
        resource_address: ResourceAddress,
        divisibility: u8,
        amount: Decimal,
    ) -> Self {
        Self::Fungible {
            resource_address,
            divisibility,
            amount,
        }
    }

    pub fn new_non_fungible(
        resource_address: ResourceAddress,
        ids: BTreeSet<NonFungibleLocalId>,
        id_type: NonFungibleIdType,
    ) -> Self {
        Self::NonFungible {
            resource_address,
            ids,
            id_type,
        }
    }
    pub fn new_empty(resource_address: ResourceAddress, resource_type: ResourceType) -> Self {
        match resource_type {
            ResourceType::Fungible { divisibility } => {
                Self::new_fungible(resource_address, divisibility, Decimal::zero())
            }
            ResourceType::NonFungible { id_type } => {
                Self::new_non_fungible(resource_address, BTreeSet::new(), id_type)
            }
        }
    }

    pub fn ids(&self) -> &BTreeSet<NonFungibleLocalId> {
        match self {
            Resource::Fungible { .. } => {
                panic!("Attempted to list non-fungible IDs on fungible resource")
            }
            Resource::NonFungible { ids, .. } => &ids,
        }
    }

    pub fn amount(&self) -> Decimal {
        match self {
            Resource::Fungible { amount, .. } => amount.clone(),
            Resource::NonFungible { ids, .. } => ids.len().into(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.amount().is_zero()
    }

    pub fn id_type(&self) -> NonFungibleIdType {
        match self {
            Resource::Fungible { .. } => panic!("id_type() called on fungible resource"),
            Resource::NonFungible { id_type, .. } => id_type.clone(),
        }
    }

    pub fn resource_address(&self) -> ResourceAddress {
        match self {
            Self::Fungible {
                resource_address, ..
            }
            | Self::NonFungible {
                resource_address, ..
            } => *resource_address,
        }
    }

    pub fn resource_type(&self) -> ResourceType {
        match self {
            Self::Fungible { divisibility, .. } => ResourceType::Fungible {
                divisibility: *divisibility,
            },
            Self::NonFungible { id_type, .. } => ResourceType::NonFungible { id_type: *id_type },
        }
    }

    pub fn put(&mut self, other: Resource) -> Result<(), ResourceOperationError> {
        // check resource address
        if self.resource_address() != other.resource_address() {
            return Err(ResourceOperationError::ResourceAddressNotMatching);
        }

        // update liquidity
        match self {
            Self::Fungible { amount, .. } => {
                *amount += other.amount();
            }
            Self::NonFungible { ids, id_type, .. } => {
                if *id_type != other.id_type() {
                    return Err(ResourceOperationError::NonFungibleIdTypeNotMatching);
                }
                ids.extend(other.ids().clone());
            }
        }
        Ok(())
    }

    pub fn take_by_amount(
        &mut self,
        amount_to_take: Decimal,
    ) -> Result<Resource, ResourceOperationError> {
        // check amount granularity
        let divisibility = self.resource_type().divisibility();
        Self::check_amount(amount_to_take, divisibility)?;

        // deduct from liquidity pool
        match self {
            Self::Fungible { amount, .. } => {
                if *amount < amount_to_take {
                    return Err(ResourceOperationError::InsufficientBalance);
                }
                *amount = *amount - amount_to_take;
                Ok(Resource::new_fungible(
                    self.resource_address(),
                    divisibility,
                    amount_to_take,
                ))
            }
            Self::NonFungible { ids, .. } => {
                if Decimal::from(ids.len()) < amount_to_take {
                    return Err(ResourceOperationError::InsufficientBalance);
                }
                let n: usize = amount_to_take
                    .to_string()
                    .parse()
                    .expect("Failed to convert amount to usize");
                let ids: BTreeSet<NonFungibleLocalId> = ids.iter().take(n).cloned().collect();
                self.take_by_ids(&ids)
            }
        }
    }

    pub fn take_by_ids(
        &mut self,
        ids_to_take: &BTreeSet<NonFungibleLocalId>,
    ) -> Result<Resource, ResourceOperationError> {
        let resource_address = self.resource_address();
        match self {
            Self::Fungible { .. } => Err(ResourceOperationError::NonFungibleOperationNotAllowed),
            Self::NonFungible { ids, id_type, .. } => {
                for id in ids_to_take {
                    if !ids.remove(&id) {
                        return Err(ResourceOperationError::InsufficientBalance);
                    }
                }
                Ok(Resource::new_non_fungible(
                    resource_address,
                    ids.clone(),
                    *id_type,
                ))
            }
        }
    }

    pub fn take_all(&mut self) -> Resource {
        self.take_by_amount(self.amount())
            .expect("Take all from `Resource` should not fail")
    }

    pub fn check_amount(amount: Decimal, divisibility: u8) -> Result<(), ResourceOperationError> {
        if amount.is_negative()
            || amount.0 % BnumI256::from(10i128.pow((18 - divisibility).into()))
                != BnumI256::from(0)
        {
            Err(ResourceOperationError::InvalidAmount(amount, divisibility))
        } else {
            Ok(())
        }
    }
}

impl Into<LockableResource> for Resource {
    fn into(self) -> LockableResource {
        match self {
            Resource::Fungible {
                resource_address,
                divisibility,
                amount,
            } => LockableResource::Fungible {
                resource_address,
                divisibility,
                locked_amounts: BTreeMap::default(),
                liquid_amount: amount,
            },
            Resource::NonFungible {
                resource_address,
                ids,
                id_type,
            } => LockableResource::NonFungible {
                resource_address,
                locked_ids: BTreeMap::new(),
                liquid_ids: ids,
                id_type,
            },
        }
    }
}

/// Resource that can be partially or completely locked for proofs.
#[derive(Debug, PartialEq, Eq)]
pub enum LockableResource {
    Fungible {
        /// The resource address.
        resource_address: ResourceAddress,
        /// The resource divisibility.
        divisibility: u8,
        /// The locked amounts and the corresponding times of being locked.
        locked_amounts: BTreeMap<Decimal, usize>,
        /// The liquid amount.
        liquid_amount: Decimal,
    },
    NonFungible {
        /// The resource address.
        resource_address: ResourceAddress,
        /// The locked non-fungible ids and the corresponding times of being locked.
        locked_ids: BTreeMap<NonFungibleLocalId, usize>,
        /// The liquid non-fungible ids.
        liquid_ids: BTreeSet<NonFungibleLocalId>,
        /// The non-fungible ID type.
        id_type: NonFungibleIdType,
    },
}

/// The locked amount or non-fungible IDs.
///
/// Invariant: always consistent with resource fungibility.
#[derive(Debug, Clone, PartialEq, Eq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub enum LockedAmountOrIds {
    Amount(Decimal),
    Ids(BTreeSet<NonFungibleLocalId>),
}

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub enum ResourceContainerId {
    Bucket(BucketId),
    Vault(VaultId),
    Worktop(u32, ResourceAddress),
}

impl LockedAmountOrIds {
    pub fn is_empty(&self) -> bool {
        self.amount().is_zero()
    }

    pub fn amount(&self) -> Decimal {
        match self {
            Self::Amount(amount) => amount.clone(),
            Self::Ids(ids) => ids.len().into(),
        }
    }

    pub fn ids(&self) -> Result<BTreeSet<NonFungibleLocalId>, ()> {
        match self {
            Self::Amount(_) => Err(()),
            Self::Ids(ids) => Ok(ids.clone()),
        }
    }
}

impl LockableResource {
    pub fn put(&mut self, other: Resource) -> Result<(), ResourceOperationError> {
        // check resource address
        if self.resource_address() != other.resource_address() {
            return Err(ResourceOperationError::ResourceAddressNotMatching);
        }

        // update liquidity
        match self {
            Self::Fungible { liquid_amount, .. } => {
                *liquid_amount += other.amount();
            }
            Self::NonFungible { liquid_ids, .. } => {
                liquid_ids.extend(other.ids().clone());
            }
        }
        Ok(())
    }

    pub fn take_by_amount(&mut self, amount: Decimal) -> Result<Resource, ResourceOperationError> {
        // check amount granularity
        let divisibility = self.resource_type().divisibility();
        Self::check_amount(amount, divisibility)?;

        // deduct from liquidity pool
        match self {
            Self::Fungible { liquid_amount, .. } => {
                if *liquid_amount < amount {
                    return Err(ResourceOperationError::InsufficientBalance);
                }
                *liquid_amount = *liquid_amount - amount;
                Ok(Resource::new_fungible(
                    self.resource_address(),
                    divisibility,
                    amount,
                ))
            }
            Self::NonFungible { liquid_ids, .. } => {
                if Decimal::from(liquid_ids.len()) < amount {
                    return Err(ResourceOperationError::InsufficientBalance);
                }
                let n: usize = amount
                    .to_string()
                    .parse()
                    .expect("Failed to convert amount to usize");
                let ids: BTreeSet<NonFungibleLocalId> =
                    liquid_ids.iter().take(n).cloned().collect();
                self.take_by_ids(&ids)
            }
        }
    }

    pub fn take_by_ids(
        &mut self,
        ids: &BTreeSet<NonFungibleLocalId>,
    ) -> Result<Resource, ResourceOperationError> {
        match self {
            Self::Fungible { .. } => Err(ResourceOperationError::NonFungibleOperationNotAllowed),
            Self::NonFungible {
                liquid_ids,
                id_type,
                ..
            } => {
                let id_type = id_type.clone();
                for id in ids {
                    if !liquid_ids.remove(&id) {
                        return Err(ResourceOperationError::InsufficientBalance);
                    }
                }
                Ok(Resource::new_non_fungible(
                    self.resource_address(),
                    ids.clone(),
                    id_type,
                ))
            }
        }
    }

    pub fn take_all_liquid(&mut self) -> Result<Resource, ResourceOperationError> {
        self.take_by_amount(self.liquid_amount())
    }

    pub fn lock_by_amount(
        &mut self,
        amount: Decimal,
    ) -> Result<LockedAmountOrIds, ResourceOperationError> {
        // check amount granularity
        let divisibility = self.resource_type().divisibility();
        Self::check_amount(amount, divisibility)?;

        match self {
            Self::Fungible {
                locked_amounts,
                liquid_amount,
                ..
            } => {
                let max_locked = Self::largest_key(locked_amounts);
                if amount > max_locked {
                    let delta = amount - max_locked;
                    if *liquid_amount >= delta {
                        *liquid_amount -= delta;
                    } else {
                        return Err(ResourceOperationError::InsufficientBalance);
                    }
                }

                locked_amounts.insert(
                    amount,
                    locked_amounts.get(&amount).cloned().unwrap_or(0) + 1,
                );

                Ok(LockedAmountOrIds::Amount(amount))
            }
            Self::NonFungible {
                locked_ids,
                liquid_ids,
                ..
            } => {
                if Decimal::from(locked_ids.len() + liquid_ids.len()) < amount {
                    return Err(ResourceOperationError::InsufficientBalance);
                }

                let n: usize = amount
                    .to_string()
                    .parse()
                    .expect("Failed to convert amount to usize");
                let mut ids: BTreeSet<NonFungibleLocalId> =
                    locked_ids.keys().take(n).cloned().collect();
                if ids.len() < n {
                    ids.extend(liquid_ids.iter().take(n - ids.len()).cloned());
                }

                self.lock_by_ids(&ids)
            }
        }
    }

    pub fn lock_by_ids(
        &mut self,
        ids: &BTreeSet<NonFungibleLocalId>,
    ) -> Result<LockedAmountOrIds, ResourceOperationError> {
        match self {
            Self::NonFungible {
                locked_ids,
                liquid_ids,
                id_type,
                ..
            } => {
                for id in ids {
                    if id.id_type() != *id_type {
                        return Err(ResourceOperationError::NonFungibleIdTypeNotMatching);
                    } else if liquid_ids.remove(id) {
                        // if the non-fungible is liquid, move it to locked.
                        locked_ids.insert(id.clone(), 1);
                    } else if let Some(cnt) = locked_ids.get_mut(id) {
                        // if the non-fungible is locked, increase the ref count.
                        *cnt += 1;
                    } else {
                        return Err(ResourceOperationError::InsufficientBalance);
                    }
                }

                Ok(LockedAmountOrIds::Ids(ids.clone()))
            }
            Self::Fungible { .. } => Err(ResourceOperationError::NonFungibleOperationNotAllowed),
        }
    }

    fn largest_key(map: &BTreeMap<Decimal, usize>) -> Decimal {
        // TODO: remove loop once `last_key_value` is stable.
        map.keys().cloned().max().unwrap_or(Decimal::zero())
    }

    pub fn unlock(&mut self, resource: &LockedAmountOrIds) {
        match resource {
            LockedAmountOrIds::Amount(amount) => match self {
                Self::Fungible {
                    locked_amounts,
                    liquid_amount,
                    ..
                } => {
                    let max_locked = Self::largest_key(locked_amounts);
                    let count = locked_amounts
                        .remove(&amount)
                        .expect("Attempted to unlock an amount that is not locked in container");
                    if count > 1 {
                        locked_amounts.insert(*amount, count - 1);
                    } else {
                        let new_max_locked = Self::largest_key(locked_amounts);
                        *liquid_amount += max_locked - new_max_locked;
                    }
                }
                Self::NonFungible { .. } => {
                    panic!("Attempted to unlock amount of non-fungible resource")
                }
            },
            LockedAmountOrIds::Ids(ids) => match self {
                Self::NonFungible {
                    locked_ids,
                    liquid_ids,
                    ..
                } => {
                    for id in ids {
                        if let Some(cnt) = locked_ids.remove(&id) {
                            if cnt > 1 {
                                locked_ids.insert(id.clone(), cnt - 1);
                            } else {
                                liquid_ids.insert(id.clone());
                            }
                        } else {
                            panic!("Attempted to unlock a non-fungible that is not locked in container");
                        }
                    }
                }
                Self::Fungible { .. } => {
                    panic!("Attempted to unlock non-fungibles of fungible resource")
                }
            },
        }
    }

    pub fn max_locked_amount(&self) -> Decimal {
        match self {
            LockableResource::Fungible { locked_amounts, .. } => Self::largest_key(locked_amounts),
            LockableResource::NonFungible { locked_ids, .. } => locked_ids.len().into(),
        }
    }

    pub fn max_locked_ids(&self) -> Result<BTreeSet<NonFungibleLocalId>, ResourceOperationError> {
        match self {
            LockableResource::Fungible { .. } => {
                Err(ResourceOperationError::NonFungibleOperationNotAllowed)
            }
            LockableResource::NonFungible { locked_ids, .. } => {
                Ok(locked_ids.keys().cloned().collect())
            }
        }
    }

    pub fn liquid_amount(&self) -> Decimal {
        match self {
            Self::Fungible { liquid_amount, .. } => *liquid_amount,
            Self::NonFungible { liquid_ids, .. } => liquid_ids.len().into(),
        }
    }

    pub fn liquid_ids(&self) -> Result<BTreeSet<NonFungibleLocalId>, ResourceOperationError> {
        match self {
            Self::Fungible { .. } => Err(ResourceOperationError::NonFungibleOperationNotAllowed),
            Self::NonFungible { liquid_ids, .. } => Ok(liquid_ids.clone()),
        }
    }

    pub fn total_amount(&self) -> Decimal {
        self.max_locked_amount() + self.liquid_amount()
    }

    pub fn total_ids(&self) -> Result<BTreeSet<NonFungibleLocalId>, ResourceOperationError> {
        let mut total = BTreeSet::new();
        total.extend(self.max_locked_ids()?);
        total.extend(self.liquid_ids()?);
        Ok(total)
    }

    pub fn is_locked(&self) -> bool {
        match self {
            Self::Fungible { locked_amounts, .. } => !locked_amounts.is_empty(),
            Self::NonFungible { locked_ids, .. } => !locked_ids.is_empty(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.total_amount().is_zero()
    }

    pub fn resource_address(&self) -> ResourceAddress {
        match self {
            Self::Fungible {
                resource_address, ..
            }
            | Self::NonFungible {
                resource_address, ..
            } => *resource_address,
        }
    }

    pub fn resource_type(&self) -> ResourceType {
        match self {
            Self::Fungible { divisibility, .. } => ResourceType::Fungible {
                divisibility: *divisibility,
            },
            Self::NonFungible { id_type, .. } => ResourceType::NonFungible { id_type: *id_type },
        }
    }

    fn check_amount(amount: Decimal, divisibility: u8) -> Result<(), ResourceOperationError> {
        if amount.is_negative()
            || amount.0 % BnumI256::from(10i128.pow((18 - divisibility).into()))
                != BnumI256::from(0)
        {
            Err(ResourceOperationError::InvalidAmount(amount, divisibility))
        } else {
            Ok(())
        }
    }

    pub fn peek_resource(&self) -> Resource {
        match self {
            LockableResource::Fungible {
                resource_address,
                divisibility,
                liquid_amount,
                ..
            } => Resource::Fungible {
                resource_address: resource_address.clone(),
                divisibility: divisibility.clone(),
                amount: liquid_amount.clone(),
            },
            LockableResource::NonFungible {
                resource_address,
                liquid_ids,
                id_type,
                ..
            } => Resource::NonFungible {
                resource_address: resource_address.clone(),
                ids: liquid_ids.clone(),
                id_type: *id_type,
            },
        }
    }
}

impl Into<Resource> for LockableResource {
    fn into(self) -> Resource {
        if self.is_locked() {
            // We keep resource containers in Rc<RefCell> for all concrete resource containers, like Bucket, Vault and Worktop.
            // When extracting the resource within a container, there should be no locked resource.
            // It should have failed the Rc::try_unwrap() check.
            panic!("Attempted to convert resource container with locked resource");
        }
        match self {
            LockableResource::Fungible {
                resource_address,
                divisibility,
                liquid_amount,
                ..
            } => Resource::Fungible {
                resource_address,
                divisibility,
                amount: liquid_amount,
            },
            LockableResource::NonFungible {
                resource_address,
                liquid_ids,
                id_type,
                ..
            } => Resource::NonFungible {
                resource_address,
                ids: liquid_ids,
                id_type: id_type,
            },
        }
    }
}
