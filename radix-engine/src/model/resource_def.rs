use sbor::*;
use scrypto::kernel::*;
use scrypto::resource::resource_flags::*;
use scrypto::resource::resource_permissions::*;
use scrypto::rust::collections::HashMap;
use scrypto::rust::string::String;
use scrypto::types::*;

use crate::model::{Auth, Supply};

/// Represents an error when accessing a bucket.
#[derive(Debug, Clone)]
pub enum ResourceDefError {
    UnauthorizedAccess,
    TypeAndSupplyNotMatching,
    UnsupportedOperation,
    OperationNotAllowed,
    InvalidGranularity,
    InvalidAmount(Decimal),
}

/// The definition of a resource.
#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct ResourceDef {
    resource_type: ResourceType,
    metadata: HashMap<String, String>,
    flags: u16,
    mutable_flags: u16,
    authorities: HashMap<Address, u16>,
    total_supply: Decimal,
}

impl ResourceDef {
    pub fn new(
        resource_type: ResourceType,
        metadata: HashMap<String, String>,
        flags: u16,
        mutable_flags: u16,
        authorities: HashMap<Address, u16>,
        initial_supply: &Option<NewSupply>,
    ) -> Result<Self, ResourceDefError> {
        let total_supply = match (resource_type, initial_supply) {
            (ResourceType::Fungible { granularity }, Some(NewSupply::Fungible { amount })) => {
                if granularity >= 36 {
                    Err(ResourceDefError::InvalidGranularity)
                } else {
                    Self::check_amount(*amount, granularity)?;
                    Ok(*amount)
                }
            }
            (ResourceType::NonFungible, Some(NewSupply::NonFungible { entries })) => {
                Ok(entries.len().into())
            }
            (_, None) => Ok(0.into()),
            _ => Err(ResourceDefError::TypeAndSupplyNotMatching),
        }?;

        Ok(Self {
            resource_type,
            metadata,
            flags,
            mutable_flags,
            authorities,
            total_supply,
        })
    }

    pub fn resource_type(&self) -> ResourceType {
        self.resource_type
    }

    pub fn metadata(&self) -> &HashMap<String, String> {
        &self.metadata
    }

    pub fn flags(&self) -> u16 {
        self.flags
    }

    pub fn mutable_flags(&self) -> u16 {
        self.mutable_flags
    }

    pub fn authorities(&self) -> &HashMap<Address, u16> {
        &self.authorities
    }

    pub fn total_supply(&self) -> Decimal {
        self.total_supply
    }

    pub fn mint(&mut self, supply: &Supply, auth: Auth) -> Result<(), ResourceDefError> {
        if self.flags() & MINTABLE == 0 {
            return Err(ResourceDefError::OperationNotAllowed);
        }
        if !auth.check_for(self.authorities(), MAY_MINT) {
            return Err(ResourceDefError::UnauthorizedAccess);
        }

        match self.resource_type {
            ResourceType::Fungible { .. } => {
                if let Supply::Fungible { amount } = supply {
                    Self::check_amount(*amount, self.resource_type.granularity())?;
                    self.total_supply += *amount;
                    Ok(())
                } else {
                    Err(ResourceDefError::TypeAndSupplyNotMatching)
                }
            }
            ResourceType::NonFungible => {
                if let Supply::NonFungible { ids } = supply {
                    self.total_supply += ids.len();
                    Ok(())
                } else {
                    Err(ResourceDefError::TypeAndSupplyNotMatching)
                }
            }
        }
    }

    pub fn burn(&mut self, supply: Supply, auth: Auth) -> Result<(), ResourceDefError> {
        if self.flags() & BURNABLE == 0 {
            return Err(ResourceDefError::OperationNotAllowed);
        }
        if !auth.check_for(self.authorities(), MAY_BURN) {
            return Err(ResourceDefError::UnauthorizedAccess);
        }

        match self.resource_type {
            ResourceType::Fungible { .. } => {
                if let Supply::Fungible { amount } = supply {
                    Self::check_amount(amount, self.resource_type.granularity())?;
                    self.total_supply -= amount;
                    Ok(())
                } else {
                    Err(ResourceDefError::TypeAndSupplyNotMatching)
                }
            }
            ResourceType::NonFungible => {
                if let Supply::NonFungible { ids } = supply {
                    // Note that the underlying NFTs are not deleted from the simulated ledger.
                    // This is not an issue when integrated with UTXO-based state model, where
                    // the UP state should have been spun down when the NFTs are withdrawn from
                    // the vault.
                    self.total_supply -= ids.len();
                    Ok(())
                } else {
                    Err(ResourceDefError::TypeAndSupplyNotMatching)
                }
            }
        }
    }

    pub fn update_nft_data(&self, auth: Auth) -> Result<(), ResourceDefError> {
        if self.flags() & INDIVIDUAL_METADATA_MUTABLE == 0 {
            return Err(ResourceDefError::OperationNotAllowed);
        }
        if !auth.check_for(self.authorities(), MAY_CHANGE_INDIVIDUAL_METADATA) {
            return Err(ResourceDefError::UnauthorizedAccess);
        }

        Ok(())
    }

    fn check_amount(amount: Decimal, granularity: u8) -> Result<(), ResourceDefError> {
        if !amount.is_negative() && amount.0 % 10i128.pow(granularity.into()) != 0.into() {
            Err(ResourceDefError::InvalidAmount(amount))
        } else {
            Ok(())
        }
    }
}
