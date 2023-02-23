use crate::math::*;
use crate::*;
use radix_engine_interface::api::types::*;
use radix_engine_interface::blueprints::resource::*;
use sbor::rust::collections::*;

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum ResourceError {
    /// Resource addresses do not match.
    ResourceAddressNotMatching,
    /// The amount is invalid, according to the resource divisibility.
    InvalidAmount(Decimal, u8),
    /// The balance is not enough.
    InsufficientBalance,
    /// Resource is locked because of proofs
    ResourceLocked,
}

// TODO: remove redundant info, such as `resource_address`.

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub struct LiquidFungibleResource {
    /// The resource address.
    resource_address: ResourceAddress,
    /// The resource divisibility.
    divisibility: u8,
    /// The total amount.
    amount: Decimal,
}

impl LiquidFungibleResource {
    pub fn new(resource_address: ResourceAddress, divisibility: u8, amount: Decimal) -> Self {
        Self {
            resource_address,
            divisibility,
            amount,
        }
    }

    pub fn new_empty(resource_address: ResourceAddress, divisibility: u8) -> Self {
        Self::new(resource_address, divisibility, Decimal::zero())
    }

    pub fn amount(&self) -> Decimal {
        self.amount.clone()
    }

    pub fn is_empty(&self) -> bool {
        self.amount.is_zero()
    }

    pub fn resource_address(&self) -> ResourceAddress {
        self.resource_address
    }

    pub fn resource_type(&self) -> ResourceType {
        ResourceType::Fungible {
            divisibility: self.divisibility,
        }
    }

    pub fn divisibility(&self) -> u8 {
        self.divisibility
    }

    pub fn put(&mut self, other: LiquidFungibleResource) -> Result<(), ResourceError> {
        // check resource address
        if self.resource_address() != other.resource_address() {
            return Err(ResourceError::ResourceAddressNotMatching);
        }

        // update liquidity
        self.amount += other.amount();

        Ok(())
    }

    pub fn take_by_amount(
        &mut self,
        amount_to_take: Decimal,
    ) -> Result<LiquidFungibleResource, ResourceError> {
        // check amount granularity
        let divisibility = self.divisibility();
        check_amount(amount_to_take, divisibility)?;

        // deduct from liquidity pool
        if self.amount < amount_to_take {
            return Err(ResourceError::InsufficientBalance);
        }
        self.amount -= amount_to_take;
        Ok(LiquidFungibleResource::new(
            self.resource_address(),
            divisibility,
            amount_to_take,
        ))
    }

    pub fn take_all(&mut self) -> LiquidFungibleResource {
        self.take_by_amount(self.amount())
            .expect("Take all from `Resource` should not fail")
    }
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub struct LiquidNonFungibleResource {
    /// The resource address
    resource_address: ResourceAddress,
    /// NonFungible Id type
    id_type: NonFungibleIdType,
    /// The total non-fungible ids.
    ids: BTreeSet<NonFungibleLocalId>,
}

impl LiquidNonFungibleResource {
    pub fn new(
        resource_address: ResourceAddress,
        id_type: NonFungibleIdType,
        ids: BTreeSet<NonFungibleLocalId>,
    ) -> Self {
        Self {
            resource_address,
            ids,
            id_type,
        }
    }

    pub fn new_empty(resource_address: ResourceAddress, id_type: NonFungibleIdType) -> Self {
        Self::new(resource_address, id_type, BTreeSet::new())
    }

    pub fn ids(&self) -> &BTreeSet<NonFungibleLocalId> {
        &self.ids
    }

    pub fn into_ids(self) -> BTreeSet<NonFungibleLocalId> {
        self.ids
    }

    pub fn amount(&self) -> Decimal {
        self.ids.len().into()
    }

    pub fn is_empty(&self) -> bool {
        self.ids.is_empty()
    }

    pub fn id_type(&self) -> NonFungibleIdType {
        self.id_type.clone()
    }

    pub fn resource_address(&self) -> ResourceAddress {
        self.resource_address.clone()
    }

    pub fn resource_type(&self) -> ResourceType {
        ResourceType::NonFungible {
            id_type: self.id_type,
        }
    }

    pub fn put(&mut self, other: LiquidNonFungibleResource) -> Result<(), ResourceError> {
        // check resource address
        if self.resource_address() != other.resource_address() {
            return Err(ResourceError::ResourceAddressNotMatching);
        }

        // update liquidity
        self.ids.extend(other.ids);
        Ok(())
    }

    pub fn take_by_amount(
        &mut self,
        amount_to_take: Decimal,
    ) -> Result<LiquidNonFungibleResource, ResourceError> {
        // check amount granularity
        check_amount(amount_to_take, 0)?;

        // deduct from liquidity pool
        if Decimal::from(self.ids.len()) < amount_to_take {
            return Err(ResourceError::InsufficientBalance);
        }
        let n: usize = amount_to_take
            .to_string()
            .parse()
            .expect("Failed to convert amount to usize");
        let ids: BTreeSet<NonFungibleLocalId> = self.ids.iter().take(n).cloned().collect();
        self.take_by_ids(&ids)
    }

    pub fn take_by_ids(
        &mut self,
        ids_to_take: &BTreeSet<NonFungibleLocalId>,
    ) -> Result<LiquidNonFungibleResource, ResourceError> {
        let resource_address = self.resource_address();
        for id in ids_to_take {
            if !self.ids.remove(&id) {
                return Err(ResourceError::InsufficientBalance);
            }
        }
        Ok(LiquidNonFungibleResource::new(
            resource_address,
            self.id_type,
            ids_to_take.clone(),
        ))
    }

    pub fn take_all(&mut self) -> LiquidNonFungibleResource {
        self.take_by_amount(self.amount())
            .expect("Take all from `Resource` should not fail")
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum LiquidResource {
    Fungible(LiquidFungibleResource),
    NonFungible(LiquidNonFungibleResource),
}

impl LiquidResource {
    pub fn resource_address(&self) -> ResourceAddress {
        match self {
            LiquidResource::Fungible(f) => f.resource_address(),
            LiquidResource::NonFungible(nf) => nf.resource_address(),
        }
    }
    pub fn resource_type(&self) -> ResourceType {
        match self {
            LiquidResource::Fungible(f) => f.resource_type(),
            LiquidResource::NonFungible(nf) => nf.resource_type(),
        }
    }

    pub fn amount(&self) -> Decimal {
        match self {
            LiquidResource::Fungible(f) => f.amount(),
            LiquidResource::NonFungible(nf) => nf.amount(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.amount().is_zero()
    }

    pub fn non_fungible_ids(&self) -> Option<&BTreeSet<NonFungibleLocalId>> {
        match self {
            LiquidResource::Fungible(_) => None,
            LiquidResource::NonFungible(nf) => Some(nf.ids()),
        }
    }

    pub fn into_fungible(self) -> Option<LiquidFungibleResource> {
        match self {
            LiquidResource::Fungible(f) => Some(f),
            LiquidResource::NonFungible(_) => None,
        }
    }

    pub fn into_non_fungibles(self) -> Option<LiquidNonFungibleResource> {
        match self {
            LiquidResource::Fungible(_) => None,
            LiquidResource::NonFungible(nf) => Some(nf),
        }
    }
}

impl From<LiquidFungibleResource> for LiquidResource {
    fn from(value: LiquidFungibleResource) -> Self {
        Self::Fungible(value)
    }
}

impl From<LiquidNonFungibleResource> for LiquidResource {
    fn from(value: LiquidNonFungibleResource) -> Self {
        Self::NonFungible(value)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct LockedFungibleResource {
    /// The locked amounts and the corresponding times of being locked.
    pub amounts: BTreeMap<Decimal, usize>,
}

impl LockedFungibleResource {
    pub fn is_locked(&self) -> bool {
        !self.amounts.is_empty()
    }

    pub fn amount(&self) -> Decimal {
        self.amounts
            .last_key_value()
            .map(|(k, _)| k)
            .cloned()
            .unwrap_or(Decimal::zero())
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct LockedNonFungibleResource {
    /// The locked non-fungible ids and the corresponding times of being locked.
    pub ids: BTreeMap<NonFungibleLocalId, usize>,
}

impl LockedNonFungibleResource {
    pub fn is_locked(&self) -> bool {
        !self.ids.is_empty()
    }

    pub fn amount(&self) -> Decimal {
        self.ids.len().into()
    }
}

pub fn check_amount(amount: Decimal, divisibility: u8) -> Result<(), ResourceError> {
    if amount.is_negative()
        || amount.0 % BnumI256::from(10i128.pow((18 - divisibility).into())) != BnumI256::from(0)
    {
        Err(ResourceError::InvalidAmount(amount, divisibility))
    } else {
        Ok(())
    }
}
