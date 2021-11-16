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
    TypeSupplyNotMatching,
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
    pub fn new_fixed(
        resource_type: ResourceType,
        metadata: HashMap<String, String>,
        supply: ResourceSupply,
        minter: Option<Address>,
    ) -> Result<(Self, ResourceSupply), ResourceDefError> {
        let total = match resource_type {
            ResourceType::Fungible { .. } => {
                if let ResourceSupply::Fungible { amount } = supply.clone() {
                    Self::check_amount( amount, resource_type)?;
                    amount
                } else {
                    return Err(ResourceDefError::TypeSupplyNotMatching);
                }
            }
            ResourceType::NonFungible => {
                if let ResourceSupply::NonFungible { entries } = supply.clone() {
                    entries.len().into()
                } else {
                    return Err(ResourceDefError::TypeSupplyNotMatching);
                }
            }
        };

        Ok((
            Self {
                resource_type,
                metadata,
                supply: total,
                minter,
            },
            supply,
        ))
    }

    pub fn new_mutable(
        resource_type: ResourceType,
        metadata: HashMap<String, String>,
        minter: Option<Address>,
    ) -> Result<Self, ResourceDefError> {
        Ok(Self {
            resource_type,
            metadata,
            supply: 0.into(),
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

    pub fn mint(
        &mut self,
        supply: ResourceSupply,
        auth: Auth,
    ) -> Result<ResourceSupply, ResourceDefError> {
        match self.minter() {
            Some(a) => {
                if !auth.contains(a) {
                    return Err(ResourceDefError::UnauthorizedAccess);
                }
            }
            None => {
                return Err(ResourceDefError::BurnNotAllowed);
            }
        };

        match self.resource_type {
            ResourceType::Fungible { .. } => {
                if let ResourceSupply::Fungible { amount } = supply {
                    Self::check_amount(amount, self.resource_type)?;
                    self.supply += amount;
                    Ok(ResourceSupply::Fungible { amount })
                } else {
                    Err(ResourceDefError::TypeSupplyNotMatching)
                }
            }
            ResourceType::NonFungible => {
                if let ResourceSupply::NonFungible { entries } = supply {
                    // TODO check existence and store in state.
                    self.supply += entries.len();
                    Ok(ResourceSupply::NonFungible { entries })
                } else {
                    Err(ResourceDefError::TypeSupplyNotMatching)
                }
            }
        }
    }

    pub fn burn(&mut self, supply: ResourceSupply, auth: Auth) -> Result<(), ResourceDefError> {
        match self.minter() {
            Some(a) => {
                if !auth.contains(a) {
                    return Err(ResourceDefError::UnauthorizedAccess);
                }
            }
            None => {
                return Err(ResourceDefError::BurnNotAllowed);
            }
        };

        match self.resource_type {
            ResourceType::Fungible { .. } => {
                if let ResourceSupply::Fungible { amount } = supply {
                    Self::check_amount(amount, self.resource_type)?;
                    self.supply -= amount;
                    Ok(())
                } else {
                    Err(ResourceDefError::TypeSupplyNotMatching)
                }
            }
            ResourceType::NonFungible => {
                if let ResourceSupply::NonFungible { entries } = supply {
                    // TODO check existence and remove in state.
                    self.supply -= entries.len();
                    Ok(())
                } else {
                    Err(ResourceDefError::TypeSupplyNotMatching)
                }
            }
        }
    }

    fn check_amount(amount: Decimal, resource_type: ResourceType) -> Result<(), ResourceDefError> {
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
