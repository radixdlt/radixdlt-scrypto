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
    ) -> Self {
        Self {
            granularity,
            metadata,
            supply,
            minter,
        }
    }

    pub fn metadata(&self) -> &HashMap<String, String> {
        &self.metadata
    }

    pub fn supply(&self) -> Decimal {
        self.supply.clone()
    }

    pub fn minter(&self) -> Option<Address> {
        self.minter.clone()
    }

    pub fn granularity(&self) -> u8 {
        self.granularity
    }

    pub fn mint(&mut self, amount: Decimal, auth: Auth) -> Result<(), ResourceDefError> {
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
}
