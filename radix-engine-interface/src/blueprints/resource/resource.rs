use crate::data::scrypto::model::*;
use crate::math::*;
use crate::*;
use sbor::rust::prelude::*;

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum ResourceError {
    InsufficientBalance,
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub struct LiquidFungibleResource {
    /// The total amount.
    amount: Decimal,
}

impl LiquidFungibleResource {
    pub fn new(amount: Decimal) -> Self {
        Self { amount }
    }

    pub fn default() -> Self {
        Self::new(Decimal::zero())
    }

    pub fn amount(&self) -> Decimal {
        self.amount.clone()
    }

    pub fn is_empty(&self) -> bool {
        self.amount.is_zero()
    }

    pub fn put(&mut self, other: LiquidFungibleResource) -> Result<(), ResourceError> {
        // update liquidity
        self.amount += other.amount();

        Ok(())
    }

    pub fn take_by_amount(
        &mut self,
        amount_to_take: Decimal,
    ) -> Result<LiquidFungibleResource, ResourceError> {
        // deduct from liquidity pool
        if self.amount < amount_to_take {
            return Err(ResourceError::InsufficientBalance);
        }
        self.amount -= amount_to_take;
        Ok(LiquidFungibleResource::new(amount_to_take))
    }

    pub fn take_all(&mut self) -> LiquidFungibleResource {
        self.take_by_amount(self.amount())
            .expect("Take all from `Resource` should not fail")
    }
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub struct LiquidNonFungibleResource {
    /// The total non-fungible ids.
    ids: BTreeSet<NonFungibleLocalId>,
}

impl LiquidNonFungibleResource {
    pub fn new(ids: BTreeSet<NonFungibleLocalId>) -> Self {
        Self { ids }
    }

    pub fn default() -> Self {
        Self::new(BTreeSet::new())
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

    pub fn put(&mut self, other: LiquidNonFungibleResource) -> Result<(), ResourceError> {
        // update liquidity
        self.ids.extend(other.ids);
        Ok(())
    }

    pub fn take_by_amount(
        &mut self,
        amount_to_take: Decimal,
    ) -> Result<LiquidNonFungibleResource, ResourceError> {
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
        for id in ids_to_take {
            if !self.ids.remove(&id) {
                return Err(ResourceError::InsufficientBalance);
            }
        }
        Ok(LiquidNonFungibleResource::new(ids_to_take.clone()))
    }

    pub fn take_all(&mut self) -> LiquidNonFungibleResource {
        self.take_by_amount(self.amount())
            .expect("Take all from `Resource` should not fail")
    }
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub struct LockedFungibleResource {
    /// The locked amounts and the corresponding times of being locked.
    pub amounts: BTreeMap<Decimal, usize>,
}

impl LockedFungibleResource {
    pub fn default() -> Self {
        Self {
            amounts: BTreeMap::new(),
        }
    }

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

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub struct LockedNonFungibleResource {
    /// The locked non-fungible ids and the corresponding times of being locked.
    pub ids: BTreeMap<NonFungibleLocalId, usize>,
}

impl LockedNonFungibleResource {
    pub fn default() -> Self {
        Self {
            ids: BTreeMap::new(),
        }
    }

    pub fn is_locked(&self) -> bool {
        !self.ids.is_empty()
    }

    pub fn amount(&self) -> Decimal {
        self.ids.len().into()
    }

    pub fn ids(&self) -> BTreeSet<NonFungibleLocalId> {
        self.ids.keys().cloned().collect()
    }
}
