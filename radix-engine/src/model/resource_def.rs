use sbor::*;
use scrypto::engine::types::*;
use scrypto::resource::resource_flags::*;
use scrypto::resource::resource_permissions::*;
use scrypto::rust::collections::HashMap;
use scrypto::rust::string::String;

use crate::model::ResourceAmount;

/// Represents an error when accessing a bucket.
#[derive(Debug, Clone)]
pub enum ResourceDefError {
    ResourceTypeNotMatching,
    OperationNotAllowed,
    PermissionNotAllowed,
    InvalidDivisibility,
    InvalidAmount(Decimal),
    InvalidResourceFlags(u64),
    InvalidResourcePermission(u64),
    InvalidFlagUpdate {
        flags: u64,
        mutable_flags: u64,
        new_flags: u64,
        new_mutable_flags: u64,
    },
}

/// The definition of a resource.
#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct ResourceDef {
    resource_type: ResourceType,
    metadata: HashMap<String, String>,
    flags: u64,
    mutable_flags: u64,
    authorities: HashMap<ResourceDefId, u64>,
    total_supply: Decimal,
}

impl ResourceDef {
    pub fn new(
        resource_type: ResourceType,
        metadata: HashMap<String, String>,
        flags: u64,
        mutable_flags: u64,
        authorities: HashMap<ResourceDefId, u64>,
        total_supply: Decimal,
    ) -> Result<Self, ResourceDefError> {
        let resource_def = Self {
            resource_type,
            metadata,
            flags,
            mutable_flags,
            authorities,
            total_supply,
        };

        if !resource_flags_are_valid(flags) {
            return Err(ResourceDefError::InvalidResourceFlags(flags));
        }

        if !resource_flags_are_valid(mutable_flags) {
            return Err(ResourceDefError::InvalidResourceFlags(mutable_flags));
        }

        for (_, permission) in &resource_def.authorities {
            if !resource_permissions_are_valid(*permission) {
                return Err(ResourceDefError::InvalidResourcePermission(*permission));
            }
        }

        Ok(resource_def)
    }

    pub fn resource_type(&self) -> ResourceType {
        self.resource_type
    }

    pub fn metadata(&self) -> &HashMap<String, String> {
        &self.metadata
    }

    pub fn flags(&self) -> u64 {
        self.flags
    }

    pub fn mutable_flags(&self) -> u64 {
        self.mutable_flags
    }

    pub fn authorities(&self) -> &HashMap<ResourceDefId, u64> {
        &self.authorities
    }

    pub fn total_supply(&self) -> Decimal {
        self.total_supply
    }

    pub fn is_flag_on(&self, flag: u64) -> bool {
        self.flags() & flag == flag
    }

    pub fn mint(
        &mut self,
        amount: &ResourceAmount,
        badge: Option<ResourceDefId>,
        initial_supply: bool,
    ) -> Result<(), ResourceDefError> {
        if !initial_supply {
            self.check_mint_auth(badge)?;
        }

        match (self.resource_type, amount) {
            (ResourceType::Fungible { .. }, ResourceAmount::Fungible { .. })
            | (ResourceType::NonFungible, ResourceAmount::NonFungible { .. }) => {
                self.total_supply += amount.as_quantity();
                Ok(())
            }
            _ => Err(ResourceDefError::ResourceTypeNotMatching),
        }
    }

    pub fn burn(
        &mut self,
        amount: ResourceAmount,
        badge: Option<ResourceDefId>,
    ) -> Result<(), ResourceDefError> {
        self.check_burn_auth(badge)?;

        match (self.resource_type, &amount) {
            (ResourceType::Fungible { .. }, ResourceAmount::Fungible { .. })
            | (ResourceType::NonFungible, ResourceAmount::NonFungible { .. }) => {
                self.total_supply -= amount.as_quantity();
                Ok(())
            }
            _ => Err(ResourceDefError::ResourceTypeNotMatching),
        }
    }

    pub fn update_flags(
        &mut self,
        new_flags: u64,
        badge: Option<ResourceDefId>,
    ) -> Result<(), ResourceDefError> {
        self.check_manage_flags_auth(badge)?;

        let changed = self.flags ^ new_flags;

        if !resource_flags_are_valid(changed) {
            return Err(ResourceDefError::InvalidResourceFlags(changed));
        }

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
        new_mutable_flags: u64,
        badge: Option<ResourceDefId>,
    ) -> Result<(), ResourceDefError> {
        self.check_manage_flags_auth(badge)?;

        let changed = self.mutable_flags ^ new_mutable_flags;

        if !resource_flags_are_valid(changed) {
            return Err(ResourceDefError::InvalidResourceFlags(changed));
        }

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
        badge: Option<ResourceDefId>,
    ) -> Result<(), ResourceDefError> {
        self.check_update_metadata_auth(badge)?;

        self.metadata = new_metadata;

        Ok(())
    }

    pub fn check_take_from_vault_auth(
        &self,
        badge: Option<ResourceDefId>,
    ) -> Result<(), ResourceDefError> {
        if !self.is_flag_on(RESTRICTED_TRANSFER) {
            Ok(())
        } else {
            self.check_permission(badge, MAY_TRANSFER)
        }
    }

    pub fn check_mint_auth(&self, badge: Option<ResourceDefId>) -> Result<(), ResourceDefError> {
        if self.is_flag_on(MINTABLE) {
            self.check_permission(badge, MAY_MINT)
        } else {
            Err(ResourceDefError::OperationNotAllowed)
        }
    }

    pub fn check_burn_auth(&self, badge: Option<ResourceDefId>) -> Result<(), ResourceDefError> {
        if self.is_flag_on(BURNABLE) {
            if self.is_flag_on(FREELY_BURNABLE) {
                Ok(())
            } else {
                self.check_permission(badge, MAY_BURN)
            }
        } else {
            Err(ResourceDefError::OperationNotAllowed)
        }
    }

    pub fn check_update_non_fungible_mutable_data_auth(
        &self,
        badge: Option<ResourceDefId>,
    ) -> Result<(), ResourceDefError> {
        if self.is_flag_on(INDIVIDUAL_METADATA_MUTABLE) {
            self.check_permission(badge, MAY_CHANGE_INDIVIDUAL_METADATA)
        } else {
            Err(ResourceDefError::OperationNotAllowed)
        }
    }

    pub fn check_update_metadata_auth(
        &self,
        badge: Option<ResourceDefId>,
    ) -> Result<(), ResourceDefError> {
        if self.is_flag_on(SHARED_METADATA_MUTABLE) {
            self.check_permission(badge, MAY_CHANGE_SHARED_METADATA)
        } else {
            Err(ResourceDefError::OperationNotAllowed)
        }
    }

    pub fn check_manage_flags_auth(
        &self,
        badge: Option<ResourceDefId>,
    ) -> Result<(), ResourceDefError> {
        self.check_permission(badge, MAY_MANAGE_RESOURCE_FLAGS)
    }

    pub fn check_amount(&self, amount: Decimal) -> Result<(), ResourceDefError> {
        let divisibility = self.resource_type.divisibility();

        if !amount.is_negative() && amount.0 % 10i128.pow((18 - divisibility).into()) != 0.into() {
            Err(ResourceDefError::InvalidAmount(amount))
        } else {
            Ok(())
        }
    }

    pub fn check_permission(
        &self,
        badge: Option<ResourceDefId>,
        permission: u64,
    ) -> Result<(), ResourceDefError> {
        if let Some(badge) = badge {
            if let Some(auth) = self.authorities.get(&badge) {
                if auth & permission == permission {
                    return Ok(());
                }
            }
        }

        Err(ResourceDefError::PermissionNotAllowed)
    }
}
