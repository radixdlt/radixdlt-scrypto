use sbor::*;
use scrypto::engine::types::*;
use scrypto::resource::resource_flags::*;
use scrypto::resource::resource_permissions::*;
use scrypto::rust::collections::HashMap;
use scrypto::rust::string::String;
use scrypto::rust::vec::Vec;

use crate::model::Proof;

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

        Ok(Self {
            resource_type,
            metadata,
            flags,
            mutable_flags,
            authorities,
            total_supply: 0.into(),
        })
    }

    pub fn check_auth(
        &self,
        transition: ResourceControllerMethod,
        proofs: Vec<&[Proof]>,
    ) -> Result<(), ResourceDefError> {
        match transition {
            ResourceControllerMethod::Mint => {
                if self.is_flag_on(MINTABLE) {
                    self.check_proof_permission(proofs, MAY_MINT)
                } else {
                    Err(ResourceDefError::OperationNotAllowed)
                }
            }
            ResourceControllerMethod::Burn => {
                if self.is_flag_on(BURNABLE) {
                    if self.is_flag_on(FREELY_BURNABLE) {
                        Ok(())
                    } else {
                        self.check_proof_permission(proofs, MAY_BURN)
                    }
                } else {
                    Err(ResourceDefError::OperationNotAllowed)
                }
            }
            ResourceControllerMethod::TakeFromVault => {
                if !self.is_flag_on(RESTRICTED_TRANSFER) {
                    Ok(())
                } else {
                    self.check_proof_permission(proofs, MAY_TRANSFER)
                }
            }
            ResourceControllerMethod::UpdateFlags
            | ResourceControllerMethod::UpdateMutableFlags => {
                self.check_proof_permission(proofs, MAY_MANAGE_RESOURCE_FLAGS)
            }
            ResourceControllerMethod::UpdateMetadata => {
                if self.is_flag_on(SHARED_METADATA_MUTABLE) {
                    self.check_proof_permission(proofs, MAY_CHANGE_SHARED_METADATA)
                } else {
                    Err(ResourceDefError::OperationNotAllowed)
                }
            }
            ResourceControllerMethod::UpdateNonFungibleMutableData => {
                if self.is_flag_on(INDIVIDUAL_METADATA_MUTABLE) {
                    self.check_proof_permission(proofs, MAY_CHANGE_INDIVIDUAL_METADATA)
                } else {
                    Err(ResourceDefError::OperationNotAllowed)
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

    pub fn authorities(&self) -> &HashMap<ResourceDefId, u64> {
        &self.authorities
    }

    pub fn total_supply(&self) -> Decimal {
        self.total_supply
    }

    pub fn is_flag_on(&self, flag: u64) -> bool {
        self.flags() & flag == flag
    }

    pub fn mint(&mut self, amount: Decimal) {
        self.total_supply += amount;
    }

    pub fn burn(&mut self, amount: Decimal) {
        self.total_supply -= amount;
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

    fn check_proof_permission(
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
