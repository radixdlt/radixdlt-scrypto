use crate::errors::RuntimeError;
use sbor::*;
use scrypto::engine::types::*;
use scrypto::prelude::ToString;
use scrypto::resource::resource_flags::*;
use scrypto::resource::resource_permissions::*;
use scrypto::rust::collections::HashMap;
use scrypto::rust::string::String;

use crate::model::{AuthRule, Proof, ResourceAmount};

#[derive(Clone, Copy, Debug)]
pub enum ResourceControllerMethod {
    Mint,
    Burn,
    TakeFromVault,
    UpdateFlags,
    UpdateMutableFlags,
    UpdateMetadata,
    UpdateNonFungibleMutableData,
}

/// Represents an error when accessing a bucket.
#[derive(Debug, Clone, PartialEq)]
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
    auth_rules: HashMap<String, AuthRule>,
    total_supply: Decimal,
}

pub const PERMISSION_MAP: [(u64, &[&str]); 6] = [
    (MAY_MINT, &["mint"]),
    (MAY_BURN, &["burn"]),
    (MAY_TRANSFER, &["take_from_vault"]),
    (
        MAY_MANAGE_RESOURCE_FLAGS,
        &["update_flags", "update_mutable_flags"],
    ),
    (MAY_CHANGE_SHARED_METADATA, &["update_metadata"]),
    (
        MAY_CHANGE_INDIVIDUAL_METADATA,
        &["update_non_fungible_mutable_data"],
    ),
];

impl ResourceDef {
    pub fn new(
        resource_type: ResourceType,
        metadata: HashMap<String, String>,
        flags: u64,
        mutable_flags: u64,
        authorities: HashMap<ResourceDefId, u64>,
        total_supply: Decimal,
    ) -> Result<Self, ResourceDefError> {
        if !resource_flags_are_valid(flags) {
            return Err(ResourceDefError::InvalidResourceFlags(flags));
        }

        if !resource_flags_are_valid(mutable_flags) {
            return Err(ResourceDefError::InvalidResourceFlags(mutable_flags));
        }

        let mut auth_rules: HashMap<String, AuthRule> = HashMap::new();
        auth_rules.insert("mint".to_string(), AuthRule::Empty);
        auth_rules.insert("burn".to_string(), AuthRule::Empty);
        auth_rules.insert("take_from_vault".to_string(), AuthRule::Empty);
        auth_rules.insert("update_flags".to_string(), AuthRule::Empty);
        auth_rules.insert("update_mutable_flags".to_string(), AuthRule::Empty);
        auth_rules.insert("update_metadata".to_string(), AuthRule::Empty);
        auth_rules.insert(
            "update_non_fungible_mutable_data".to_string(),
            AuthRule::Empty,
        );

        for (resource_def_id, permission) in authorities {
            if !resource_permissions_are_valid(permission) {
                return Err(ResourceDefError::InvalidResourcePermission(permission));
            }

            for (flag, methods) in PERMISSION_MAP.iter() {
                if permission & flag != 0 {
                    for method in methods.iter() {
                        let cur_rule = auth_rules.remove(*method).unwrap();
                        let new_rule = AuthRule::JustResource(resource_def_id);
                        auth_rules.insert((*method).to_string(), cur_rule.or(new_rule));
                    }
                }
            }
        }

        let resource_def = Self {
            resource_type,
            metadata,
            flags,
            mutable_flags,
            auth_rules,
            total_supply,
        };

        Ok(resource_def)
    }

    pub fn check_auth(
        &self,
        transition: ResourceControllerMethod,
        proofs: &[&[Proof]],
    ) -> Result<(), RuntimeError> {
        match transition {
            ResourceControllerMethod::Mint => {
                if self.is_flag_on(MINTABLE) {
                    self.auth_rules.get("mint").unwrap().check(proofs)
                } else {
                    Err(RuntimeError::UnsupportedOperation)
                }
            }
            ResourceControllerMethod::Burn => {
                if self.is_flag_on(BURNABLE) {
                    if self.is_flag_on(FREELY_BURNABLE) {
                        Ok(())
                    } else {
                        self.auth_rules.get("burn").unwrap().check(proofs)
                    }
                } else {
                    Err(RuntimeError::UnsupportedOperation)
                }
            }
            ResourceControllerMethod::TakeFromVault => {
                if !self.is_flag_on(RESTRICTED_TRANSFER) {
                    Ok(())
                } else {
                    self.auth_rules
                        .get("take_from_vault")
                        .unwrap()
                        .check(proofs)
                }
            }
            ResourceControllerMethod::UpdateFlags => {
                self.auth_rules.get("update_flags").unwrap().check(proofs)
            }
            ResourceControllerMethod::UpdateMutableFlags => self
                .auth_rules
                .get("update_mutable_flags")
                .unwrap()
                .check(proofs),
            ResourceControllerMethod::UpdateMetadata => {
                if self.is_flag_on(SHARED_METADATA_MUTABLE) {
                    self.auth_rules
                        .get("update_metadata")
                        .unwrap()
                        .check(proofs)
                } else {
                    Err(RuntimeError::UnsupportedOperation)
                }
            }
            ResourceControllerMethod::UpdateNonFungibleMutableData => {
                if self.is_flag_on(INDIVIDUAL_METADATA_MUTABLE) {
                    self.auth_rules
                        .get("update_non_fungible_mutable_data")
                        .unwrap()
                        .check(proofs)
                } else {
                    Err(RuntimeError::UnsupportedOperation)
                }
            }
        }
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

    pub fn total_supply(&self) -> Decimal {
        self.total_supply
    }

    pub fn is_flag_on(&self, flag: u64) -> bool {
        self.flags() & flag == flag
    }

    pub fn mint(&mut self, amount: &ResourceAmount) -> Result<(), ResourceDefError> {
        match (self.resource_type, amount) {
            (ResourceType::Fungible { .. }, ResourceAmount::Fungible { .. })
            | (ResourceType::NonFungible, ResourceAmount::NonFungible { .. }) => {
                self.total_supply += amount.as_quantity();
                Ok(())
            }
            _ => Err(ResourceDefError::ResourceTypeNotMatching),
        }
    }

    pub fn burn(&mut self, amount: ResourceAmount) -> Result<(), ResourceDefError> {
        match (self.resource_type, &amount) {
            (ResourceType::Fungible { .. }, ResourceAmount::Fungible { .. })
            | (ResourceType::NonFungible, ResourceAmount::NonFungible { .. }) => {
                self.total_supply -= amount.as_quantity();
                Ok(())
            }
            _ => Err(ResourceDefError::ResourceTypeNotMatching),
        }
    }

    pub fn update_mutable_flags(&mut self, new_mutable_flags: u64) -> Result<(), ResourceDefError> {
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
    ) -> Result<(), ResourceDefError> {
        self.metadata = new_metadata;

        Ok(())
    }

    pub fn update_flags(&mut self, new_flags: u64) -> Result<(), ResourceDefError> {
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

    pub fn check_amount(&self, amount: Decimal) -> Result<(), ResourceDefError> {
        let divisibility = self.resource_type.divisibility();

        if !amount.is_negative() && amount.0 % 10i128.pow((18 - divisibility).into()) != 0.into() {
            Err(ResourceDefError::InvalidAmount(amount))
        } else {
            Ok(())
        }
    }
}
