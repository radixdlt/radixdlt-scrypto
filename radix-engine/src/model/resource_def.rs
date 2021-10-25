use sbor::*;
use scrypto::rust::collections::HashMap;
use scrypto::rust::string::String;
use scrypto::rust::vec::Vec;
use scrypto::types::*;

/// Represents an error when accessing a bucket.
#[derive(Debug, Clone)]
pub enum ResourceDefError {
    UnauthorizedAccess,
    MintNotAllowed,
}

/// The definition of a resource.
#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct ResourceDef {
    metadata: HashMap<String, String>,
    supply: Amount,
    mint_auth: Option<Address>,
}

impl ResourceDef {
    pub fn new(
        metadata: HashMap<String, String>,
        supply: Amount,
        mint_auth: Option<Address>,
    ) -> Self {
        Self {
            metadata,
            supply,
            mint_auth,
        }
    }

    pub fn metadata(&self) -> &HashMap<String, String> {
        &self.metadata
    }

    pub fn supply(&self) -> Amount {
        self.supply
    }

    pub fn mint_auth(&self) -> Option<Address> {
        self.mint_auth.clone()
    }

    pub fn mint(&mut self, amount: Amount, auth: Vec<Address>) -> Result<(), ResourceDefError> {
        match self.mint_auth() {
            Some(m) => {
                if auth.contains(&m) {
                    self.supply += amount;
                    Ok(())
                } else {
                    Err(ResourceDefError::UnauthorizedAccess)
                }
            }
            None => Err(ResourceDefError::MintNotAllowed),
        }
    }

    pub fn burn(&mut self, amount: Amount) {
        self.supply -= amount;
    }
}
