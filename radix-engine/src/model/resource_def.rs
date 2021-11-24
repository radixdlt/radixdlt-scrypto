use sbor::*;
use scrypto::kernel::*;
use scrypto::rust::collections::HashMap;
use scrypto::rust::string::String;
use scrypto::types::*;

use crate::model::{Auth, Supply};

/// Represents an error when accessing a bucket.
#[derive(Debug, Clone)]
pub enum ResourceDefError {
    UnauthorizedAccess,
    InvalidGranularity,
    TypeAndSupplyNotMatching,
    UnsupportedOperation,
    MutableOperationNotAllowed,
    InvalidAmount(Decimal),
}

/// The definition of a resource.
#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct ResourceDef {
    resource_type: ResourceType,
    metadata: HashMap<String, String>,
    total_supply: Decimal,
    mutable: bool,
    auth_configs: Option<ResourceAuthConfigs>,
}

impl ResourceDef {
    pub fn new_fixed(
        resource_type: ResourceType,
        metadata: HashMap<String, String>,
        supply: &Supply,
    ) -> Result<Self, ResourceDefError> {
        let total_supply = match resource_type {
            ResourceType::Fungible { .. } => {
                if let Supply::Fungible { amount } = supply {
                    Self::check_amount(*amount, resource_type)?;
                    *amount
                } else {
                    return Err(ResourceDefError::TypeAndSupplyNotMatching);
                }
            }
            ResourceType::NonFungible => {
                if let Supply::NonFungible { entries } = supply {
                    entries.len().into()
                } else {
                    return Err(ResourceDefError::TypeAndSupplyNotMatching);
                }
            }
        };

        Ok(Self {
            resource_type,
            metadata,
            total_supply,
            mutable: false,
            auth_configs: None,
        })
    }

    pub fn new_mutable(
        resource_type: ResourceType,
        metadata: HashMap<String, String>,
        auth_configs: ResourceAuthConfigs,
    ) -> Result<Self, ResourceDefError> {
        Ok(Self {
            resource_type,
            metadata,
            total_supply: 0.into(),
            mutable: true,
            auth_configs: Some(auth_configs),
        })
    }

    pub fn resource_type(&self) -> ResourceType {
        self.resource_type
    }

    pub fn metadata(&self) -> &HashMap<String, String> {
        &self.metadata
    }

    pub fn total_supply(&self) -> Decimal {
        self.total_supply
    }

    pub fn mutable(&self) -> bool {
        self.mutable
    }

    pub fn auth_configs(&self) -> Option<ResourceAuthConfigs> {
        self.auth_configs.clone()
    }

    pub fn mint(&mut self, supply: &Supply, auth: Auth) -> Result<(), ResourceDefError> {
        if !self.mutable {
            return Err(ResourceDefError::MutableOperationNotAllowed);
        }
        if !auth.contains(self.auth_configs().unwrap().mint_badge) {
            return Err(ResourceDefError::UnauthorizedAccess);
        }

        match self.resource_type {
            ResourceType::Fungible { .. } => {
                if let Supply::Fungible { amount } = supply {
                    Self::check_amount(*amount, self.resource_type)?;
                    self.total_supply += *amount;
                    Ok(())
                } else {
                    Err(ResourceDefError::TypeAndSupplyNotMatching)
                }
            }
            ResourceType::NonFungible => {
                if let Supply::NonFungible { entries } = supply {
                    self.total_supply += entries.len();
                    Ok(())
                } else {
                    Err(ResourceDefError::TypeAndSupplyNotMatching)
                }
            }
        }
    }

    pub fn burn(&mut self, supply: Supply, auth: Auth) -> Result<(), ResourceDefError> {
        if !self.mutable {
            return Err(ResourceDefError::MutableOperationNotAllowed);
        }
        if !auth.contains(self.auth_configs().unwrap().mint_badge) {
            return Err(ResourceDefError::UnauthorizedAccess);
        }

        match self.resource_type {
            ResourceType::Fungible { .. } => {
                if let Supply::Fungible { amount } = supply {
                    Self::check_amount(amount, self.resource_type)?;
                    self.total_supply -= amount;
                    Ok(())
                } else {
                    Err(ResourceDefError::TypeAndSupplyNotMatching)
                }
            }
            ResourceType::NonFungible => {
                if let Supply::NonFungible { entries } = supply {
                    // Note that the underlying NFTs are not deleted from the simulated ledger.
                    // This is not an issue when integrated with UTXO-based state model, where
                    // the UP state should have been spun down when the NFTs are withdrawn from
                    // the vault.
                    self.total_supply -= entries.len();
                    Ok(())
                } else {
                    Err(ResourceDefError::TypeAndSupplyNotMatching)
                }
            }
        }
    }

    pub fn change_to_immutable(&mut self, auth: Auth) -> Result<(), ResourceDefError> {
        if !self.mutable {
            return Err(ResourceDefError::MutableOperationNotAllowed);
        }
        if !auth.contains(self.auth_configs().unwrap().update_badge) {
            return Err(ResourceDefError::UnauthorizedAccess);
        }

        self.mutable = false;
        Ok(())
    }

    fn check_amount(amount: Decimal, resource_type: ResourceType) -> Result<(), ResourceDefError> {
        if amount.is_negative() {
            return Err(ResourceDefError::InvalidAmount(amount));
        }

        let granularity = match resource_type {
            ResourceType::Fungible { granularity } => granularity,
            ResourceType::NonFungible => 19,
        };

        if granularity >= 1 && granularity <= 36 {
            if amount.0 % 10i128.pow((granularity - 1).into()) != 0.into() {
                Err(ResourceDefError::InvalidAmount(amount))
            } else {
                Ok(())
            }
        } else {
            Err(ResourceDefError::InvalidGranularity)
        }
    }
}
