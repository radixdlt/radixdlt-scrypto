use sbor::*;
use scrypto::kernel::*;
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
    resource_type: ResourceType,
    metadata: HashMap<String, String>,
    supply: Decimal,
    minter: Option<Address>,
}

impl ResourceDef {
    pub fn new(
        resource_type: ResourceType,
        metadata: HashMap<String, String>,
        supply: Decimal,
        minter: Option<Address>,
    ) -> Result<Self, ResourceDefError> {
        Self::check_amount(&supply, resource_type)?;

        Ok(Self {
            resource_type,
            metadata,
            supply,
            minter,
        })
    }

    pub fn resource_type(&self) -> ResourceType {
        self.resource_type
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

    pub fn mint(&mut self, amount: Decimal, auth: Auth) -> Result<(), ResourceDefError> {
        Self::check_amount(&amount, self.resource_type)?;

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
        Self::check_amount(&amount, self.resource_type)?;

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

    fn check_amount(amount: &Decimal, resource_type: ResourceType) -> Result<(), ResourceDefError> {
        if amount.is_negative() {
            return Err(ResourceDefError::NegativeAmount);
        }

        let granularity = match resource_type {
            ResourceType::Fungible { granularity } => granularity,
            ResourceType::NonFungible => 19,
        };

        if granularity >= 1 && granularity <= 36 {
            if amount.0 % 10i128.pow((granularity - 1).into()) != 0.into() {
                Err(ResourceDefError::GranularityCheckFailed)
            } else {
                Ok(())
            }
        } else {
            Err(ResourceDefError::InvalidGranularity)
        }
    }
}
