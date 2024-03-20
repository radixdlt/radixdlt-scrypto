use crate::internal_prelude::*;
use radix_common::data::scrypto::model::*;
use radix_common::math::*;
use radix_engine_interface::blueprints::resource::VaultFreezeFlags;
use sbor::rust::prelude::*;

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum ResourceError {
    InsufficientBalance { requested: Decimal, actual: Decimal },
    InvalidTakeAmount,
    MissingNonFungibleLocalId(NonFungibleLocalId),
    DecimalOverflow,
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
#[sbor(transparent)]
pub struct LiquidFungibleResource {
    /// The total amount.
    amount: Decimal,
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
#[sbor(transparent)]
pub struct VaultFrozenFlag {
    pub frozen: VaultFreezeFlags,
}

impl Default for VaultFrozenFlag {
    fn default() -> Self {
        Self {
            frozen: VaultFreezeFlags::empty(),
        }
    }
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

    pub fn put(&mut self, other: LiquidFungibleResource) {
        // update liquidity
        // NOTE: Decimal arithmetic operation safe unwrap.
        // Mint limit should prevent from overflowing
        self.amount = self.amount.checked_add(other.amount()).expect("Overflow");
    }

    pub fn take_by_amount(
        &mut self,
        amount_to_take: Decimal,
    ) -> Result<LiquidFungibleResource, ResourceError> {
        // deduct from liquidity pool
        if self.amount < amount_to_take {
            return Err(ResourceError::InsufficientBalance {
                requested: amount_to_take,
                actual: self.amount,
            });
        }
        self.amount = self
            .amount
            .checked_sub(amount_to_take)
            .ok_or(ResourceError::DecimalOverflow)?;
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
    pub ids: IndexSet<NonFungibleLocalId>,
}

impl LiquidNonFungibleResource {
    pub fn new(ids: IndexSet<NonFungibleLocalId>) -> Self {
        Self { ids }
    }

    pub fn default() -> Self {
        Self::new(IndexSet::default())
    }

    pub fn ids(&self) -> &IndexSet<NonFungibleLocalId> {
        &self.ids
    }

    pub fn into_ids(self) -> IndexSet<NonFungibleLocalId> {
        self.ids
    }

    pub fn amount(&self) -> Decimal {
        self.ids.len().into()
    }

    pub fn is_empty(&self) -> bool {
        self.ids.is_empty()
    }

    pub fn put(&mut self, other: LiquidNonFungibleResource) -> Result<(), ResourceError> {
        self.ids.extend(other.ids);
        Ok(())
    }

    pub fn take_by_amount(&mut self, n: u32) -> Result<LiquidNonFungibleResource, ResourceError> {
        if self.ids.len() < n as usize {
            return Err(ResourceError::InsufficientBalance {
                actual: Decimal::from(self.ids.len()),
                requested: Decimal::from(n),
            });
        }
        let ids: IndexSet<NonFungibleLocalId> = self.ids.iter().take(n as usize).cloned().collect();
        self.take_by_ids(&ids)
    }

    pub fn take_by_ids(
        &mut self,
        ids_to_take: &IndexSet<NonFungibleLocalId>,
    ) -> Result<LiquidNonFungibleResource, ResourceError> {
        for id in ids_to_take {
            if !self.ids.swap_remove(id) {
                return Err(ResourceError::MissingNonFungibleLocalId(id.clone()));
            }
        }
        Ok(LiquidNonFungibleResource::new(ids_to_take.clone()))
    }

    pub fn take_all(&mut self) -> LiquidNonFungibleResource {
        LiquidNonFungibleResource {
            ids: core::mem::replace(&mut self.ids, indexset!()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub struct LockedFungibleResource {
    /// The locked amounts and the corresponding times of being locked.
    pub amounts: IndexMap<Decimal, usize>,
}

impl LockedFungibleResource {
    pub fn default() -> Self {
        Self {
            amounts: index_map_new(),
        }
    }

    pub fn is_locked(&self) -> bool {
        !self.amounts.is_empty()
    }

    pub fn amount(&self) -> Decimal {
        let mut max = Decimal::ZERO;
        for amount in self.amounts.keys() {
            if amount > &max {
                max = amount.clone()
            }
        }
        max
    }
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub struct LockedNonFungibleResource {
    /// The locked non-fungible ids and the corresponding times of being locked.
    pub ids: IndexMap<NonFungibleLocalId, usize>,
}

impl LockedNonFungibleResource {
    pub fn default() -> Self {
        Self {
            ids: index_map_new(),
        }
    }

    pub fn is_locked(&self) -> bool {
        !self.ids.is_empty()
    }

    pub fn amount(&self) -> Decimal {
        self.ids.len().into()
    }

    pub fn ids(&self) -> IndexSet<NonFungibleLocalId> {
        self.ids.keys().cloned().collect()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
#[sbor(transparent)]
pub struct LiquidNonFungibleVault {
    pub amount: Decimal,
}
