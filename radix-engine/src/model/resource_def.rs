use sbor::*;
use scrypto::kernel::*;
use scrypto::resource::resource_flags::*;
use scrypto::resource::resource_permissions::*;
use scrypto::rust::collections::HashMap;
use scrypto::rust::string::String;
use scrypto::types::*;

use crate::model::{Actor, Supply};

/// Represents an error when accessing a bucket.
#[derive(Debug, Clone)]
pub enum ResourceDefError {
    UnauthorizedAccess,
    TypeAndSupplyNotMatching,
    UnsupportedOperation,
    OperationNotAllowed,
    InvalidDivisibility,
    InvalidAmount(Decimal),
    InvalidFlagUpdate {
        flags: u16,
        mutable_flags: u16,
        new_flags: u16,
        new_mutable_flags: u16,
    },
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
        let mut resource_def = Self {
            resource_type,
            metadata,
            flags,
            mutable_flags,
            authorities,
            total_supply: Decimal::zero(),
        };

        resource_def.total_supply = match (resource_type, initial_supply) {
            (ResourceType::Fungible { divisibility }, Some(NewSupply::Fungible { amount })) => {
                if divisibility > 18 {
                    Err(ResourceDefError::InvalidDivisibility)
                } else {
                    resource_def.check_amount(*amount)?;
                    Ok(*amount)
                }
            }
            (ResourceType::NonFungible, Some(NewSupply::NonFungible { entries })) => {
                Ok(entries.len().into())
            }
            (_, None) => Ok(0.into()),
            _ => Err(ResourceDefError::TypeAndSupplyNotMatching),
        }?;

        Ok(resource_def)
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

    pub fn is_flag_on(&self, flag: u16) -> bool {
        self.flags() & flag == flag
    }

    pub fn mint(&mut self, supply: &Supply, actor: Actor) -> Result<(), ResourceDefError> {
        self.check_mint_auth(actor)?;

        match self.resource_type {
            ResourceType::Fungible { .. } => {
                if let Supply::Fungible { amount } = supply {
                    self.check_amount(*amount)?;
                    self.total_supply += *amount;
                    Ok(())
                } else {
                    Err(ResourceDefError::TypeAndSupplyNotMatching)
                }
            }
            ResourceType::NonFungible => {
                if let Supply::NonFungible { keys } = supply {
                    self.total_supply += keys.len();
                    Ok(())
                } else {
                    Err(ResourceDefError::TypeAndSupplyNotMatching)
                }
            }
        }
    }

    pub fn burn(&mut self, supply: Supply, actor: Actor) -> Result<(), ResourceDefError> {
        self.check_burn_auth(actor)?;

        match self.resource_type {
            ResourceType::Fungible { .. } => {
                if let Supply::Fungible { amount } = supply {
                    self.check_amount(amount)?;
                    self.total_supply -= amount;
                    Ok(())
                } else {
                    Err(ResourceDefError::TypeAndSupplyNotMatching)
                }
            }
            ResourceType::NonFungible => {
                if let Supply::NonFungible { keys } = supply {
                    // Note that the underlying NFTs are not deleted from the simulated ledger.
                    // This is not an issue when integrated with UTXO-based state model, where
                    // the UP state should have been spun down when the NFTs are withdrawn from
                    // the vault.
                    self.total_supply -= keys.len();
                    Ok(())
                } else {
                    Err(ResourceDefError::TypeAndSupplyNotMatching)
                }
            }
        }
    }

    pub fn update_flags(&mut self, new_flags: u16, actor: Actor) -> Result<(), ResourceDefError> {
        self.check_manage_flags_auth(actor)?;

        let changed = self.flags ^ new_flags;
        if self.mutable_flags | changed != self.mutable_flags {
            return Err(ResourceDefError::InvalidFlagUpdate {
                flags: self.flags,
                mutable_flags: self.mutable_flags,
                new_flags,
                new_mutable_flags: self.mutable_flags,
            });
        }
        self.flags = new_flags;

        Ok(())
    }

    pub fn update_mutable_flags(
        &mut self,
        new_mutable_flags: u16,
        actor: Actor,
    ) -> Result<(), ResourceDefError> {
        self.check_manage_flags_auth(actor)?;

        let changed = self.mutable_flags ^ new_mutable_flags;
        if self.mutable_flags | changed != self.mutable_flags {
            return Err(ResourceDefError::InvalidFlagUpdate {
                flags: self.flags,
                mutable_flags: self.mutable_flags,
                new_flags: self.flags,
                new_mutable_flags: new_mutable_flags,
            });
        }
        self.mutable_flags = new_mutable_flags;

        Ok(())
    }

    pub fn update_metadata(
        &mut self,
        new_metadata: HashMap<String, String>,
        actor: Actor,
    ) -> Result<(), ResourceDefError> {
        self.check_update_metadata_auth(actor)?;

        self.metadata = new_metadata;

        Ok(())
    }

    pub fn check_take_from_vault_auth(&self, actor: Actor) -> Result<(), ResourceDefError> {
        if !self.is_flag_on(RESTRICTED_TRANSFER) {
            Ok(())
        } else {
            actor
                .check_permission(self.authorities(), MAY_TRANSFER)
                .then(|| ())
                .ok_or(ResourceDefError::UnauthorizedAccess)
        }
    }

    pub fn check_mint_auth(&self, actor: Actor) -> Result<(), ResourceDefError> {
        if self.is_flag_on(MINTABLE) {
            actor
                .check_permission(self.authorities(), MAY_MINT)
                .then(|| ())
                .ok_or(ResourceDefError::UnauthorizedAccess)
        } else {
            Err(ResourceDefError::OperationNotAllowed)
        }
    }

    pub fn check_burn_auth(&self, actor: Actor) -> Result<(), ResourceDefError> {
        if self.is_flag_on(FREELY_BURNABLE) {
            Ok(())
        } else if self.is_flag_on(BURNABLE) {
            actor
                .check_permission(self.authorities(), MAY_BURN)
                .then(|| ())
                .ok_or(ResourceDefError::UnauthorizedAccess)
        } else {
            Err(ResourceDefError::OperationNotAllowed)
        }
    }

    pub fn check_update_nft_mutable_data_auth(&self, actor: Actor) -> Result<(), ResourceDefError> {
        if self.is_flag_on(INDIVIDUAL_METADATA_MUTABLE) {
            actor
                .check_permission(self.authorities(), MAY_CHANGE_INDIVIDUAL_METADATA)
                .then(|| ())
                .ok_or(ResourceDefError::UnauthorizedAccess)
        } else {
            Err(ResourceDefError::OperationNotAllowed)
        }
    }

    pub fn check_update_metadata_auth(&self, actor: Actor) -> Result<(), ResourceDefError> {
        if self.is_flag_on(SHARED_METADATA_MUTABLE) {
            actor
                .check_permission(self.authorities(), MAY_CHANGE_SHARED_METADATA)
                .then(|| ())
                .ok_or(ResourceDefError::UnauthorizedAccess)
        } else {
            Err(ResourceDefError::OperationNotAllowed)
        }
    }

    pub fn check_manage_flags_auth(&self, actor: Actor) -> Result<(), ResourceDefError> {
        actor
            .check_permission(self.authorities(), MAY_MANAGE_RESOURCE_FLAGS)
            .then(|| ())
            .ok_or(ResourceDefError::UnauthorizedAccess)
    }

    pub fn check_amount(&self, amount: Decimal) -> Result<(), ResourceDefError> {
        let divisibility = self.resource_type.divisibility();

        if !amount.is_negative() && amount.0 % 10i128.pow((18 - divisibility).into()) != 0.into() {
            Err(ResourceDefError::InvalidAmount(amount))
        } else {
            Ok(())
        }
    }
}
