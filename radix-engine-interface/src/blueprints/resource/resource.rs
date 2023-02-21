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
        let divisibility = self.resource_type().divisibility();
        Self::check_amount(amount_to_take, divisibility)?;

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

    pub fn check_amount(amount: Decimal, divisibility: u8) -> Result<(), ResourceError> {
        if amount.is_negative()
            || amount.0 % BnumI256::from(10i128.pow((18 - divisibility).into()))
                != BnumI256::from(0)
        {
            Err(ResourceError::InvalidAmount(amount, divisibility))
        } else {
            Ok(())
        }
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
        let divisibility = self.resource_type().divisibility();
        Self::check_amount(amount_to_take, divisibility)?;

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

    pub fn check_amount(amount: Decimal, divisibility: u8) -> Result<(), ResourceError> {
        if amount.is_negative()
            || amount.0 % BnumI256::from(10i128.pow((18 - divisibility).into()))
                != BnumI256::from(0)
        {
            Err(ResourceError::InvalidAmount(amount, divisibility))
        } else {
            Ok(())
        }
    }
}

/// Resource that can be partially or completely locked for proofs.
#[derive(Debug, PartialEq, Eq)]
pub struct LockedFungibleResource {
    /// The locked amounts and the corresponding times of being locked.
    amounts: BTreeMap<Decimal, usize>,
}

impl LockedFungibleResource {
    pub fn is_locked(&self) -> bool {
        !self.amounts.is_empty()
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct LockedNonFungibleResource {
    /// The locked non-fungible ids and the corresponding times of being locked.
    ids: BTreeMap<NonFungibleLocalId, usize>,
}

impl LockedNonFungibleResource {
    pub fn is_locked(&self) -> bool {
        !self.ids.is_empty()
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

    pub fn non_fungible_ids(&self) -> Option<&BTreeSet<NonFungibleLocalId>> {
        match self {
            LiquidResource::Fungible(f) => None,
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

//=============================
// TODO: remove code below
//=============================

#[derive(Debug)]
pub struct FungibleResource {
    liquid: LiquidFungibleResource,
    locked: LockedFungibleResource,
}

impl FungibleResource {
    pub fn new(liquid: LiquidFungibleResource) -> Self {
        Self {
            liquid,
            locked: LockedFungibleResource {
                amounts: BTreeMap::default(),
            },
        }
    }

    pub fn into_liquid(self) -> Result<LiquidFungibleResource, ResourceError> {
        if self.is_locked() {
            Err(ResourceError::ResourceLocked)
        } else {
            Ok(self.liquid)
        }
    }

    pub fn put(&mut self, other: LiquidFungibleResource) -> Result<(), ResourceError> {
        self.liquid.put(other)
    }

    pub fn take_by_amount(
        &mut self,
        amount: Decimal,
    ) -> Result<LiquidFungibleResource, ResourceError> {
        self.liquid.take_by_amount(amount)
    }

    pub fn take_all_liquid(&mut self) -> Result<LiquidFungibleResource, ResourceError> {
        self.take_by_amount(self.liquid_amount())
    }

    pub fn lock_by_amount(&mut self, amount: Decimal) -> Result<Decimal, ResourceError> {
        // check amount granularity
        let divisibility = self.resource_type().divisibility();
        Self::check_amount(amount, divisibility)?;

        let max_locked = self.max_locked_amount();
        if amount > max_locked {
            let delta = amount - max_locked;
            if self.liquid.amount >= delta {
                self.liquid.amount -= delta;
            } else {
                return Err(ResourceError::InsufficientBalance);
            }
        }

        self.locked.amounts.insert(
            amount,
            self.locked.amounts.get(&amount).cloned().unwrap_or(0) + 1,
        );

        Ok(amount)
    }

    pub fn unlock(&mut self, amount: Decimal) {
        let max_locked = self.max_locked_amount();
        let count = self
            .locked
            .amounts
            .remove(&amount)
            .expect("Attempted to unlock an amount that is not locked in container");
        if count > 1 {
            self.locked.amounts.insert(amount, count - 1);
        } else {
            let new_max_locked = self.max_locked_amount();
            self.liquid.amount += max_locked - new_max_locked;
        }
    }

    pub fn max_locked_amount(&self) -> Decimal {
        self.locked
            .amounts
            .last_key_value()
            .map(|(k, _)| k)
            .cloned()
            .unwrap_or(Decimal::zero())
    }

    pub fn liquid_amount(&self) -> Decimal {
        self.liquid.amount
    }

    pub fn total_amount(&self) -> Decimal {
        self.max_locked_amount() + self.liquid_amount()
    }

    pub fn is_locked(&self) -> bool {
        !self.locked.amounts.is_empty()
    }

    pub fn is_empty(&self) -> bool {
        self.total_amount().is_zero()
    }

    pub fn resource_address(&self) -> ResourceAddress {
        self.liquid.resource_address()
    }

    pub fn divisibility(&self) -> u8 {
        self.liquid.divisibility()
    }

    pub fn resource_type(&self) -> ResourceType {
        self.liquid.resource_type()
    }

    fn check_amount(amount: Decimal, divisibility: u8) -> Result<(), ResourceError> {
        if amount.is_negative()
            || amount.0 % BnumI256::from(10i128.pow((18 - divisibility).into()))
                != BnumI256::from(0)
        {
            Err(ResourceError::InvalidAmount(amount, divisibility))
        } else {
            Ok(())
        }
    }

    pub fn peek_resource(&self) -> LiquidFungibleResource {
        LiquidFungibleResource {
            resource_address: self.resource_address(),
            divisibility: self.divisibility(),
            amount: self.liquid_amount(),
        }
    }
}

#[derive(Debug)]
pub struct NonFungibleResource {
    liquid: LiquidNonFungibleResource,
    locked: LockedNonFungibleResource,
}

impl NonFungibleResource {
    pub fn new(liquid: LiquidNonFungibleResource) -> Self {
        Self {
            liquid,
            locked: LockedNonFungibleResource {
                ids: BTreeMap::default(),
            },
        }
    }

    pub fn into_liquid(self) -> Result<LiquidNonFungibleResource, ResourceError> {
        if self.is_locked() {
            Err(ResourceError::ResourceLocked)
        } else {
            Ok(self.liquid)
        }
    }

    pub fn put(&mut self, other: LiquidNonFungibleResource) -> Result<(), ResourceError> {
        self.liquid.put(other)
    }

    pub fn take_by_amount(
        &mut self,
        amount: Decimal,
    ) -> Result<LiquidNonFungibleResource, ResourceError> {
        // check amount granularity
        let divisibility = self.resource_type().divisibility();
        Self::check_amount(amount, divisibility)?;

        // calculate the non-fungible id to take
        if Decimal::from(self.liquid.ids.len()) < amount {
            return Err(ResourceError::InsufficientBalance);
        }
        let n: usize = amount
            .to_string()
            .parse()
            .expect("Failed to convert amount to usize");
        let ids: BTreeSet<NonFungibleLocalId> = self.liquid.ids.iter().take(n).cloned().collect();

        self.take_by_ids(&ids)
    }

    pub fn take_by_ids(
        &mut self,
        ids: &BTreeSet<NonFungibleLocalId>,
    ) -> Result<LiquidNonFungibleResource, ResourceError> {
        for id in ids {
            if !self.liquid.ids.remove(&id) {
                return Err(ResourceError::InsufficientBalance);
            }
        }
        Ok(LiquidNonFungibleResource::new(
            self.resource_address(),
            self.id_type(),
            ids.clone(),
        ))
    }

    pub fn take_all_liquid(&mut self) -> Result<LiquidNonFungibleResource, ResourceError> {
        self.take_by_amount(self.liquid_amount())
    }

    pub fn lock_by_amount(
        &mut self,
        amount: Decimal,
    ) -> Result<BTreeSet<NonFungibleLocalId>, ResourceError> {
        // check amount granularity
        let divisibility = self.resource_type().divisibility();
        Self::check_amount(amount, divisibility)?;

        if Decimal::from(self.locked.ids.len() + self.liquid.ids.len()) < amount {
            return Err(ResourceError::InsufficientBalance);
        }
        let n: usize = amount
            .to_string()
            .parse()
            .expect("Failed to convert amount to usize");
        let mut ids: BTreeSet<NonFungibleLocalId> =
            self.locked.ids.keys().take(n).cloned().collect();
        if ids.len() < n {
            ids.extend(self.liquid.ids.iter().take(n - ids.len()).cloned());
        }

        self.lock_by_ids(&ids)
    }

    pub fn lock_by_ids(
        &mut self,
        ids: &BTreeSet<NonFungibleLocalId>,
    ) -> Result<BTreeSet<NonFungibleLocalId>, ResourceError> {
        for id in ids {
            if self.liquid.ids.remove(id) {
                // if the non-fungible is liquid, move it to locked.
                self.locked.ids.insert(id.clone(), 1);
            } else if let Some(cnt) = self.locked.ids.get_mut(id) {
                // if the non-fungible is locked, increase the ref count.
                *cnt += 1;
            } else {
                return Err(ResourceError::InsufficientBalance);
            }
        }

        Ok(ids.clone())
    }

    pub fn unlock(&mut self, ids: &BTreeSet<NonFungibleLocalId>) {
        for id in ids {
            if let Some(cnt) = self.locked.ids.remove(&id) {
                if cnt > 1 {
                    self.locked.ids.insert(id.clone(), cnt - 1);
                } else {
                    self.liquid.ids.insert(id.clone());
                }
            } else {
                panic!("Attempted to unlock a non-fungible that is not locked in container");
            }
        }
    }

    pub fn max_locked_amount(&self) -> Decimal {
        self.locked.ids.len().into()
    }

    pub fn max_locked_ids(&self) -> BTreeSet<NonFungibleLocalId> {
        self.locked.ids.keys().cloned().collect()
    }

    pub fn liquid_amount(&self) -> Decimal {
        self.liquid.ids.len().into()
    }

    pub fn liquid_ids(&self) -> BTreeSet<NonFungibleLocalId> {
        self.liquid.ids.clone()
    }

    pub fn total_amount(&self) -> Decimal {
        self.max_locked_amount() + self.liquid_amount()
    }

    pub fn total_ids(&self) -> BTreeSet<NonFungibleLocalId> {
        let mut total = BTreeSet::new();
        total.extend(self.max_locked_ids());
        total.extend(self.liquid_ids());
        total
    }

    pub fn is_locked(&self) -> bool {
        !self.locked.ids.is_empty()
    }

    pub fn is_empty(&self) -> bool {
        self.total_amount().is_zero()
    }

    pub fn resource_address(&self) -> ResourceAddress {
        self.liquid.resource_address()
    }

    pub fn id_type(&self) -> NonFungibleIdType {
        self.liquid.id_type()
    }

    pub fn resource_type(&self) -> ResourceType {
        ResourceType::NonFungible {
            id_type: self.liquid.id_type,
        }
    }

    fn check_amount(amount: Decimal, divisibility: u8) -> Result<(), ResourceError> {
        if amount.is_negative()
            || amount.0 % BnumI256::from(10i128.pow((18 - divisibility).into()))
                != BnumI256::from(0)
        {
            Err(ResourceError::InvalidAmount(amount, divisibility))
        } else {
            Ok(())
        }
    }

    pub fn peek_resource(&self) -> LiquidNonFungibleResource {
        LiquidNonFungibleResource {
            resource_address: self.resource_address(),
            ids: self.liquid_ids(),
            id_type: self.id_type(),
        }
    }
}
