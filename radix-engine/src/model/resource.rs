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
    /// The other container is locked, thus can't be put into this container
    ContainerLocked,
    /// Generating zero-amount proof is not allowed
    ZeroAmountProofNotAllowed,
}

#[derive(Debug, TypeId, Encode, Decode)]
pub enum ResourceContainer {
    // TODO: update state based on proofs.
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

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum ResourceContainerId {
    /// For a vault on ledger state
    Vault(VaultId),
    /// For a bucket on the n-th worktop
    Bucket(usize, BucketId),
    /// For a resource container on the n-th worktop
    Worktop(usize, ResourceDefId),
}

#[derive(Debug)]
pub struct Proof {
    /// The resource definition id
    resource_def_id: ResourceDefId,
    /// The resource type
    resource_type: ResourceType,
    /// Restricted proof can't be moved down along the call stack (growing down).
    restricted: bool,
    /// The total amount for optimization purpose
    total_amount: Amount,
    /// The sub-amounts (to be extended)
    amounts: HashMap<ResourceContainerId, Amount>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProofError {
    SupportContainerError(ResourceContainerError),
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

        // check container lock status
        if other.is_locked() {
            return Err(ResourceContainerError::ContainerLocked);
        }

        // add the other bucket into liquid pool
        match (self, other.liquid_amount()) {
            (Self::Fungible { liquid_amount, .. }, Amount::Fungible { amount }) => {
                *liquid_amount = *liquid_amount + amount;
            }
            (Self::NonFungible { liquid_ids, .. }, Amount::NonFungible { ids }) => {
                liquid_ids.extend(ids);
            }
            _ => panic!("Resource type should match!"),
        }
        Ok(())
    }

    pub fn take(&mut self, quantity: Decimal) -> Result<Self, ResourceContainerError> {
        // check amount granularity
        let divisibility = self.resource_type().divisibility();
        Self::check_amount(quantity, divisibility)?;

        // deduct from liquidity pool
        match self {
            Self::Fungible { liquid_amount, .. } => {
                if *liquid_amount < quantity {
                    return Err(ResourceContainerError::InsufficientBalance);
                }
                *liquid_amount = *liquid_amount - quantity;
                Ok(Self::new_fungible(
                    self.resource_def_id(),
                    divisibility,
                    quantity,
                ))
            }
            Self::NonFungible { liquid_ids, .. } => {
                let n: usize = quantity.to_string().parse().unwrap();
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

    pub fn lock(&mut self, amount: &Amount) -> Result<(), ResourceContainerError> {
        match &amount {
            Amount::Fungible { amount } => self.lock_quantity(*amount),
            Amount::NonFungible { ids } => self.lock_non_fungibles(ids),
        }
    }

    pub fn lock_quantity(&mut self, quantity: Decimal) -> Result<(), ResourceContainerError> {
        // TODO do we allow locking non-fungibles in a fungible way?

        let max_locked = self.max_locked_amount().as_quantity();

        match self {
            Self::Fungible {
                locked_amounts,
                liquid_amount,
                ..
            } => {
                if quantity > max_locked {
                    let delta = quantity - max_locked;
                    if *liquid_amount >= delta {
                        *liquid_amount -= delta;
                    } else {
                        return Err(ResourceContainerError::InsufficientBalance);
                    }
                }

                locked_amounts.insert(
                    quantity,
                    locked_amounts.get(&quantity).cloned().unwrap_or(0) + 1,
                );

                Ok(())
            }
            Self::NonFungible { .. } => Err(ResourceContainerError::FungibleOperationNotAllowed),
        }
    }

    pub fn lock_non_fungibles(
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

    pub fn unlock(&mut self, amount: &Amount) -> Result<(), ResourceContainerError> {
        match &amount {
            Amount::Fungible { amount } => self.unlock_quantity(*amount),
            Amount::NonFungible { ids } => self.unlock_non_fungibles(ids),
        }
    }

    fn largest_key(map: &BTreeMap<Decimal, usize>) -> Decimal {
        // TODO: remove loop once `last_key_value` is stable.
        map.keys().cloned().max().unwrap_or(Decimal::zero())
    }

    pub fn unlock_quantity(&mut self, quantity: Decimal) -> Result<(), ResourceContainerError> {
        // TODO do we allow locking non-fungibles in a fungible way?

        match self {
            Self::Fungible {
                locked_amounts,
                liquid_amount,
                ..
            } => {
                let max_locked = Self::largest_key(locked_amounts);
                let count = locked_amounts
                    .remove(&quantity)
                    .expect("Amount not locked in the container");
                if count > 1 {
                    locked_amounts.insert(quantity, count - 1);
                } else {
                    let new_max_locked = Self::largest_key(locked_amounts);
                    *liquid_amount += max_locked - new_max_locked;
                }
                Ok(())
            }
            Self::NonFungible { .. } => Err(ResourceContainerError::FungibleOperationNotAllowed),
        }
    }

    pub fn unlock_non_fungibles(
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

    pub fn max_locked_amount(&self) -> Amount {
        match self {
            ResourceContainer::Fungible { locked_amounts, .. } => Amount::Fungible {
                amount: Self::largest_key(locked_amounts),
            },
            ResourceContainer::NonFungible { locked_ids, .. } => Amount::NonFungible {
                ids: locked_ids.keys().cloned().collect(),
            },
        }
    }

    pub fn liquid_amount(&self) -> Amount {
        match self {
            Self::Fungible { liquid_amount, .. } => Amount::Fungible {
                amount: liquid_amount.clone(),
            },
            Self::NonFungible { liquid_ids, .. } => Amount::NonFungible {
                ids: liquid_ids.clone(),
            },
        }
    }

    pub fn total_amount(&self) -> Amount {
        let mut total = self.liquid_amount();
        total.add(&self.max_locked_amount()).unwrap();
        total
    }

    pub fn is_locked(&self) -> bool {
        match self {
            Self::Fungible { locked_amounts, .. } => !locked_amounts.is_empty(),
            Self::NonFungible { locked_ids, .. } => !locked_ids.is_empty(),
        }
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

impl Proof {
    // TODO: partial proof
    // TODO: multiple containers
    // TODO: mixed types of container

    pub fn new(
        resource_container_id: ResourceContainerId,
        resource_container: &mut ResourceContainer,
    ) -> Result<Self, ProofError> {
        let resource_def_id = resource_container.resource_def_id();
        let resource_type = resource_container.resource_type();

        // lock the full amount
        let total_amount = resource_container.total_amount();
        resource_container
            .lock(&total_amount)
            .map_err(ProofError::SupportContainerError)?;

        // record the supporting container
        let mut amounts = HashMap::new();
        amounts.insert(resource_container_id, total_amount.clone());

        // generate proof
        Ok(Self {
            resource_def_id,
            resource_type,
            restricted: false,
            total_amount,
            amounts,
        })
    }

    pub fn resource_def_id(&self) -> ResourceDefId {
        self.resource_def_id
    }

    pub fn resource_type(&self) -> ResourceType {
        self.resource_type
    }

    pub fn total_amount(&self) -> Amount {
        self.total_amount.clone()
    }

    pub fn is_restricted(&self) -> bool {
        self.restricted
    }

    pub fn amounts(&mut self) -> &mut HashMap<ResourceContainerId, Amount> {
        &mut self.amounts
    }
}
