use sbor::*;
use scrypto::rust::collections::HashMap;
use scrypto::rust::string::String;
use scrypto::types::*;

use crate::model::Auth;

/// Represents an error when accessing a bucket.
#[derive(Debug, Clone)]
pub enum ResourceDefError {
    UnauthorizedAccess,
    MintNotAllowed,
    BurnNotAllowed,
    InvalidGranularity,
    GranularityCheckFailed,
    NegativeAmount,
}

/// The definition of a resource.
#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct ResourceDef {
    granularity: u8,
    metadata: HashMap<String, String>,
    supply: Decimal,
    minter: Option<Address>,
}

impl ResourceDef {
    pub fn new(
        granularity: u8,
        metadata: HashMap<String, String>,
        supply: Decimal,
        minter: Option<Address>,
    ) -> Result<Self, ResourceDefError> {
        Self::check_amount(&supply, granularity)?;

        Ok(Self {
            granularity,
            metadata,
            supply,
            minter,
        })
    }

    pub fn metadata(&self) -> &HashMap<String, String> {
        &self.metadata
    }

    pub fn supply(&self) -> Decimal {
        self.supply
    }

    pub fn minter(&self) -> Option<Address> {
        self.minter.clone()
    }

    pub fn granularity(&self) -> u8 {
        self.granularity
    }

    pub fn mint(&mut self, amount: Decimal, auth: Auth) -> Result<(), ResourceDefError> {
        Self::check_amount(&amount, self.granularity)?;

        match self.minter() {
            Some(a) => {
                if auth.contains(a) {
                    self.supply += amount;
                    Ok(())
                } else {
                    Err(ResourceDefError::UnauthorizedAccess)
                }
            }
            None => Err(ResourceDefError::MintNotAllowed),
        }
    }

    pub fn burn(&mut self, amount: Decimal, auth: Auth) -> Result<(), ResourceDefError> {
        Self::check_amount(&amount, self.granularity)?;

        match self.minter() {
            Some(a) => {
                if auth.contains(a) {
                    self.supply -= amount;
                    Ok(())
                } else {
                    Err(ResourceDefError::UnauthorizedAccess)
                }
            }
            None => Err(ResourceDefError::BurnNotAllowed),
        }
    }

    fn check_amount(amount: &Decimal, granularity: u8) -> Result<(), ResourceDefError> {
        if amount.is_negative() {
            return Err(ResourceDefError::NegativeAmount);
        }

        match granularity {
            1 => Ok(()),
            18 => {
                if amount.0 % 10i128.pow(18) != 0.into() {
                    Err(ResourceDefError::GranularityCheckFailed)
                } else {
                    Ok(())
                }
            }
            _ => Err(ResourceDefError::InvalidGranularity),
        }
    }
}
