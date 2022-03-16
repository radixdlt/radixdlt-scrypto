use crate::math::Decimal;
use crate::resource::*;
use crate::rust::collections::BTreeSet;
use sbor::*;

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum AmountError {
    CantTreatFungibleAsNonFungible,
    CantAddFungibleToNonFungible,
    CantAddNonFungibleToFungible,
    CantSubtractNonFungibleFromFungible,
    CantSubtractFungibleFromNonFungible,
    Underflow,
}

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub enum Amount {
    /// Fungible amount is represented by some quantity.
    Fungible { amount: Decimal },
    /// Non-fungible amount is represented by a set of IDs.
    NonFungible { ids: BTreeSet<NonFungibleId> },
}

impl Amount {
    pub fn is_zero(&self) -> bool {
        self.as_quantity().is_zero()
    }

    /// Treats as fungible by returning a quantity.
    pub fn as_quantity(&self) -> Decimal {
        match self {
            Amount::Fungible { amount } => *amount,
            Amount::NonFungible { ids } => ids.len().into(),
        }
    }

    /// Treats as non-fungible by returning the IDs, if possible.
    pub fn as_non_fungible_ids(&self) -> Result<BTreeSet<NonFungibleId>, AmountError> {
        match self {
            Amount::Fungible { .. } => Err(AmountError::CantTreatFungibleAsNonFungible),
            Amount::NonFungible { ids } => Ok(ids.clone()),
        }
    }

    /// Adds another amount to this amount.
    pub fn add(&mut self, other: &Self) -> Result<(), AmountError> {
        match self {
            Amount::Fungible { amount } => {
                *amount += match other {
                    Amount::Fungible { amount } => amount.clone(),
                    Amount::NonFungible { .. } => {
                        return Err(AmountError::CantAddNonFungibleToFungible);
                    }
                };
            }
            Amount::NonFungible { ids } => {
                ids.extend(match other {
                    Amount::Fungible { .. } => {
                        return Err(AmountError::CantAddFungibleToNonFungible);
                    }
                    Amount::NonFungible { ids } => ids.clone(),
                });
            }
        };
        Ok(())
    }

    /// Subtracts another amount from this amount.
    pub fn subtract(&mut self, other: &Self) -> Result<(), AmountError> {
        match self {
            Amount::Fungible { amount } => {
                let other_amount = match other {
                    Amount::Fungible { amount } => amount.clone(),
                    Amount::NonFungible { .. } => {
                        return Err(AmountError::CantSubtractNonFungibleFromFungible);
                    }
                };
                if *amount < other_amount {
                    return Err(AmountError::Underflow);
                }
                *amount -= other_amount;
            }
            Amount::NonFungible { ids } => {
                let other_ids = match other {
                    Amount::Fungible { .. } => {
                        return Err(AmountError::CantSubtractFungibleFromNonFungible);
                    }
                    Amount::NonFungible { ids } => ids,
                };
                if !ids.is_superset(&other_ids) {
                    return Err(AmountError::Underflow);
                }
                for id in other_ids {
                    ids.remove(&id);
                }
            }
        };
        Ok(())
    }
}
