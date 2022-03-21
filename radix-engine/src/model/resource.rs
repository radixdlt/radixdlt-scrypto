use sbor::*;
use scrypto::engine::types::*;
use scrypto::rust::collections::BTreeMap;
use scrypto::rust::collections::BTreeSet;
use scrypto::rust::collections::HashMap;
use scrypto::rust::string::ToString;

/// Represents an error when manipulating resources in a container.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResourceContainerError {
    /// Resource addresses do not match
    ResourceAddressNotMatching,
    /// The amount is invalid, according to the resource divisibility
    InvalidAmount(Decimal, u8),
    /// The balance is not enough
    InsufficientBalance,
    /// Fungible operation on non-fungible resource is not allowed
    FungibleOperationNotAllowed,
    /// Non-fungible operation on fungible resource is not allowed
    NonFungibleOperationNotAllowed,
}

#[derive(Debug, TypeId, Encode, Decode)]
pub enum ResourceContainer {
    Fungible {
        /// The resource definition id
        resource_def_id: ResourceDefId,
        /// The resource divisibility
        divisibility: u8,
        /// The locked amounts and the corresponding times of being locked.
        locked_amounts: BTreeMap<Decimal, usize>,
        /// The liquid amount.
        liquid_amount: Decimal,
    },
    NonFungible {
        /// The resource definition id
        resource_def_id: ResourceDefId,
        /// The locked non-fungible ids and the corresponding times of being locked.
        locked_ids: HashMap<NonFungibleId, usize>,
        /// The liquid non-fungible ids.
        liquid_ids: BTreeSet<NonFungibleId>,
    },
}

impl ResourceContainer {
    pub fn new_fungible(resource_def_id: ResourceDefId, divisibility: u8, amount: Decimal) -> Self {
        Self::Fungible {
            resource_def_id,
            divisibility,
            locked_amounts: BTreeMap::new(),
            liquid_amount: amount,
        }
    }

    pub fn new_non_fungible(resource_def_id: ResourceDefId, ids: BTreeSet<NonFungibleId>) -> Self {
        Self::NonFungible {
            resource_def_id,
            locked_ids: HashMap::new(),
            liquid_ids: ids.clone(),
        }
    }

    pub fn new_empty(resource_def_id: ResourceDefId, resource_type: ResourceType) -> Self {
        match resource_type {
            ResourceType::Fungible { divisibility } => {
                Self::new_fungible(resource_def_id, divisibility, Decimal::zero())
            }
            ResourceType::NonFungible => Self::new_non_fungible(resource_def_id, BTreeSet::new()),
        }
    }

    pub fn put(&mut self, other: Self) -> Result<(), ResourceContainerError> {
        // check resource address
        if self.resource_def_id() != other.resource_def_id() {
            return Err(ResourceContainerError::ResourceAddressNotMatching);
        }

        // Invariant: owned container should always be free
        assert!(!other.is_locked());

        // update liquidity
        match self {
            Self::Fungible { liquid_amount, .. } => {
                *liquid_amount = other.liquid_amount();
            }
            Self::NonFungible { liquid_ids, .. } => {
                liquid_ids.extend(other.liquid_ids()?);
            }
        }
        Ok(())
    }

    pub fn take(&mut self, amount: Decimal) -> Result<Self, ResourceContainerError> {
        // check amount granularity
        let divisibility = self.resource_type().divisibility();
        Self::check_amount(amount, divisibility)?;

        // deduct from liquidity pool
        match self {
            Self::Fungible { liquid_amount, .. } => {
                if *liquid_amount < amount {
                    return Err(ResourceContainerError::InsufficientBalance);
                }
                *liquid_amount = *liquid_amount - amount;
                Ok(Self::new_fungible(
                    self.resource_def_id(),
                    divisibility,
                    amount,
                ))
            }
            Self::NonFungible { liquid_ids, .. } => {
                let n: usize = amount.to_string().parse().unwrap();
                let taken: BTreeSet<NonFungibleId> = liquid_ids.iter().cloned().take(n).collect();
                taken.iter().for_each(|key| {
                    liquid_ids.remove(key);
                });
                Ok(Self::new_non_fungible(self.resource_def_id(), taken))
            }
        }
    }

    pub fn take_non_fungibles(
        &mut self,
        ids: &BTreeSet<NonFungibleId>,
    ) -> Result<Self, ResourceContainerError> {
        match self {
            Self::Fungible { .. } => Err(ResourceContainerError::NonFungibleOperationNotAllowed),
            Self::NonFungible { liquid_ids, .. } => {
                for id in ids {
                    if !liquid_ids.remove(&id) {
                        return Err(ResourceContainerError::InsufficientBalance);
                    }
                }
                Ok(Self::new_non_fungible(self.resource_def_id(), ids.clone()))
            }
        }
    }

    pub fn lock_amount(&mut self, amount: Decimal) -> Result<(), ResourceContainerError> {
        // TODO do we allow locking non-fungibles in a fungible way?

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
                        return Err(ResourceContainerError::InsufficientBalance);
                    }
                }

                locked_amounts.insert(
                    amount,
                    locked_amounts.get(&amount).cloned().unwrap_or(0) + 1,
                );

                Ok(())
            }
            Self::NonFungible { .. } => Err(ResourceContainerError::FungibleOperationNotAllowed),
        }
    }

    pub fn lock_ids(
        &mut self,
        ids: &BTreeSet<NonFungibleId>,
    ) -> Result<(), ResourceContainerError> {
        match self {
            Self::NonFungible {
                locked_ids,
                liquid_ids,
                ..
            } => {
                for id in ids {
                    if liquid_ids.remove(id) {
                        // if the non-fungible is liquid, move it to locked.
                        locked_ids.insert(id.clone(), 1);
                    } else if let Some(cnt) = locked_ids.get_mut(id) {
                        // if the non-fungible is locked, increase the ref count.
                        *cnt += 1;
                    } else {
                        return Err(ResourceContainerError::InsufficientBalance);
                    }
                }

                Ok(())
            }
            Self::Fungible { .. } => Err(ResourceContainerError::NonFungibleOperationNotAllowed),
        }
    }

    fn largest_key(map: &BTreeMap<Decimal, usize>) -> Decimal {
        // TODO: remove loop once `last_key_value` is stable.
        map.keys().cloned().max().unwrap_or(Decimal::zero())
    }

    pub fn unlock_amount(&mut self, amount: Decimal) -> Result<(), ResourceContainerError> {
        // TODO do we allow locking non-fungibles in a fungible way?

        match self {
            Self::Fungible {
                locked_amounts,
                liquid_amount,
                ..
            } => {
                let max_locked = Self::largest_key(locked_amounts);
                let count = locked_amounts
                    .remove(&amount)
                    .expect("Amount not locked in the container");
                if count > 1 {
                    locked_amounts.insert(amount, count - 1);
                } else {
                    let new_max_locked = Self::largest_key(locked_amounts);
                    *liquid_amount += max_locked - new_max_locked;
                }
                Ok(())
            }
            Self::NonFungible { .. } => Err(ResourceContainerError::FungibleOperationNotAllowed),
        }
    }

    pub fn unlock_ids(
        &mut self,
        ids: &BTreeSet<NonFungibleId>,
    ) -> Result<(), ResourceContainerError> {
        match self {
            Self::NonFungible {
                locked_ids,
                liquid_ids,
                ..
            } => {
                for id in ids {
                    if let Some(cnt) = locked_ids.remove(id) {
                        if cnt > 1 {
                            locked_ids.insert(id.clone(), cnt - 1);
                        } else {
                            liquid_ids.insert(id.clone());
                        }
                    } else {
                        panic!("Non-fungible not locked in the container: id = {}", id);
                    }
                }
                Ok(())
            }
            Self::Fungible { .. } => Err(ResourceContainerError::NonFungibleOperationNotAllowed),
        }
    }

    pub fn max_locked_amount(&self) -> Decimal {
        match self {
            ResourceContainer::Fungible { locked_amounts, .. } => Self::largest_key(locked_amounts),
            ResourceContainer::NonFungible { locked_ids, .. } => locked_ids.len().into(),
        }
    }

    pub fn max_locked_ids(&self) -> Result<BTreeSet<NonFungibleId>, ResourceContainerError> {
        match self {
            ResourceContainer::Fungible { .. } => {
                Err(ResourceContainerError::NonFungibleOperationNotAllowed)
            }
            ResourceContainer::NonFungible { locked_ids, .. } => {
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

    pub fn liquid_ids(&self) -> Result<BTreeSet<NonFungibleId>, ResourceContainerError> {
        match self {
            Self::Fungible { .. } => Err(ResourceContainerError::NonFungibleOperationNotAllowed),
            Self::NonFungible { liquid_ids, .. } => Ok(liquid_ids.clone()),
        }
    }

    pub fn total_amount(&self) -> Decimal {
        self.max_locked_amount() + self.liquid_amount()
    }

    pub fn total_ids(&self) -> Result<BTreeSet<NonFungibleId>, ResourceContainerError> {
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

    pub fn resource_def_id(&self) -> ResourceDefId {
        match self {
            Self::Fungible {
                resource_def_id, ..
            }
            | Self::NonFungible {
                resource_def_id, ..
            } => *resource_def_id,
        }
    }

    pub fn resource_type(&self) -> ResourceType {
        match self {
            Self::Fungible { divisibility, .. } => ResourceType::Fungible {
                divisibility: *divisibility,
            },
            Self::NonFungible { .. } => ResourceType::NonFungible,
        }
    }

    fn check_amount(amount: Decimal, divisibility: u8) -> Result<(), ResourceContainerError> {
        if !amount.is_negative() && amount.0 % 10i128.pow((18 - divisibility).into()) != 0.into() {
            Err(ResourceContainerError::InvalidAmount(amount, divisibility))
        } else {
            Ok(())
        }
    }
}
