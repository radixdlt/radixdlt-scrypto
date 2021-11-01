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
    metadata: HashMap<String, String>,
    supply: Amount,
    mint_burn_auth: Option<Address>,
}

impl ResourceDef {
    pub fn new(
        metadata: HashMap<String, String>,
        supply: Amount,
        mint_burn_auth: Option<Address>,
    ) -> Self {
        Self {
            metadata,
            supply,
            mint_burn_auth,
        }
    }

    pub fn metadata(&self) -> &HashMap<String, String> {
        &self.metadata
    }

    pub fn supply(&self) -> Amount {
        self.supply
    }

    pub fn mint_burn_auth(&self) -> Option<Address> {
        self.mint_burn_auth.clone()
    }

    pub fn mint(&mut self, amount: Amount, auth: Auth) -> Result<(), ResourceDefError> {
        match self.mint_burn_auth() {
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

    pub fn burn(&mut self, amount: Amount, auth: Auth) -> Result<(), ResourceDefError> {
        match self.mint_burn_auth() {
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
