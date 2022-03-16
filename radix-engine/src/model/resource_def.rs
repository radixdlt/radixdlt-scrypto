use sbor::*;
use scrypto::engine::types::*;
use scrypto::resource::resource_flags::*;
use scrypto::resource::resource_permissions::*;
use scrypto::rust::collections::BTreeSet;
use scrypto::rust::collections::HashMap;
use scrypto::rust::string::String;
use scrypto::rust::vec::Vec;

use crate::model::Proof;

/// Represents an error when accessing a bucket.
#[derive(Debug, Clone, PartialEq)]
pub enum ResourceDefError {
    TypeAndAmountNotMatching,
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
    AmountError(AmountError),
}

/// The definition of a resource.
#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct ResourceDef {
    resource_type: ResourceType,
    metadata: HashMap<String, String>,
    flags: u64,
    mutable_flags: u64,
    authorities: HashMap<ResourceDefId, u64>,
    total_supply: Amount,
}

impl ResourceDef {
    pub fn new(
        resource_type: ResourceType,
        metadata: HashMap<String, String>,
        flags: u64,
        mutable_flags: u64,
        authorities: HashMap<ResourceDefId, u64>,
    ) -> Result<Self, ResourceDefError> {
        if !resource_flags_are_valid(flags) {
            return Err(ResourceDefError::InvalidResourceFlags(flags));
        }

        if !resource_flags_are_valid(mutable_flags) {
            return Err(ResourceDefError::InvalidResourceFlags(mutable_flags));
        }

        for (_, permission) in &authorities {
            if !resource_permissions_are_valid(*permission) {
                return Err(ResourceDefError::InvalidResourcePermission(*permission));
            }
        }

        let total_supply = match resource_type {
            ResourceType::Fungible { .. } => Amount::Fungible { amount: 0.into() },
            ResourceType::NonFungible => Amount::NonFungible {
                ids: BTreeSet::new(),
            },
        };

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
        self.total_supply.as_quantity()
    }

    pub fn is_flag_on(&self, flag: u64) -> bool {
        self.flags() & flag == flag
    }

    pub fn mint(
        &mut self,
        amount: &Amount,
        badge: Option<ResourceDefId>,
        is_initial_supply: bool,
    ) -> Result<(), ResourceDefError> {
        if !is_initial_supply {
            self.check_mint_auth(badge)?;
        }

        self.total_supply
            .add(amount)
            .map_err(ResourceDefError::AmountError)
    }

    pub fn burn(
        &mut self,
        amount: &Amount,
        badge: Option<ResourceDefId>,
    ) -> Result<(), ResourceDefError> {
        self.check_burn_auth(badge)?;

        self.total_supply
            .subtract(amount)
            .map_err(ResourceDefError::AmountError)
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
        proofs: Vec<&[Proof]>,
    ) -> Result<(), ResourceDefError> {
        if !self.is_flag_on(RESTRICTED_TRANSFER) {
            Ok(())
        } else {
            self.check_proof_permission(proofs, MAY_TRANSFER)
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

    pub fn check_proof_permission(
        &self,
        proofs_vector: Vec<&[Proof]>,
        permission: u64,
    ) -> Result<(), ResourceDefError> {
        for proofs in proofs_vector {
            for p in proofs {
                let proof_resource_def_id = p.resource_def_id();
                if let Some(auth) = self.authorities.get(&proof_resource_def_id) {
                    if auth & permission == permission {
                        return Ok(());
                    }
                }
            }
        }

        Err(ResourceDefError::PermissionNotAllowed)
    }
}
