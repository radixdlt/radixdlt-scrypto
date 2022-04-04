use crate::model::method_authorization::{HardAuthRule, HardProofRule, HardProofRuleResourceList};
use sbor::*;
use scrypto::engine::types::*;
use scrypto::prelude::ToString;
use scrypto::resource::resource_flags::*;
use scrypto::resource::resource_permissions::*;
use scrypto::rust::collections::HashMap;
use scrypto::rust::string::String;
use scrypto::rust::vec;

use crate::model::MethodAuthorization;

/// Represents an error when accessing a bucket.
#[derive(Debug, Clone, PartialEq)]
pub enum ResourceDefError {
    InvalidDivisibility,
    InvalidAmount(Decimal, u8),
    InvalidResourceFlags(u64),
    InvalidResourcePermission(u64),
    FlagsLocked,
    ResourceTypeDoesNotMatch,
    MaxMintAmountExceeded,
}

/// The definition of a resource.
#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct ResourceDef {
    resource_type: ResourceType,
    metadata: HashMap<String, String>,
    flags: u64,
    mutable_flags: u64,
    authorization: HashMap<String, MethodAuthorization>,
    total_supply: Decimal,
}

pub const PERMISSION_MAP: [(u64, &[&str]); 6] = [
    (MAY_MINT, &["mint"]),
    (MAY_BURN, &["burn"]),
    (MAY_TRANSFER, &["take_from_vault"]),
    (
        MAY_MANAGE_RESOURCE_FLAGS,
        &["enable_flags", "disable_flags", "lock_flags"],
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
    ) -> Result<Self, ResourceDefError> {
        if !resource_flags_are_valid(flags) {
            return Err(ResourceDefError::InvalidResourceFlags(flags));
        }

        if !resource_flags_are_valid(mutable_flags) {
            return Err(ResourceDefError::InvalidResourceFlags(mutable_flags));
        }

        let mut method_states: HashMap<String, MethodAuthorization> = HashMap::new();
        if flags & MINTABLE > 0 {
            method_states.insert(
                "mint".to_string(),
                MethodAuthorization::Protected(HardAuthRule::AllOf(vec![]))
            );
        }

        if flags & BURNABLE > 0 {
            method_states.insert(
                "burn".to_string(),
                MethodAuthorization::Protected(HardAuthRule::AllOf(vec![]))
            );
        }

        if flags & FREELY_BURNABLE > 0 {
            method_states.insert(
                "burn".to_string(),
                MethodAuthorization::Public
            );
        }

        if flags & RESTRICTED_TRANSFER > 0 {
            method_states.insert(
                "take_from_vault".to_string(),
                MethodAuthorization::Protected(HardAuthRule::AllOf(vec![]))
            );
        } else {
            method_states.insert(
                "take_from_vault".to_string(),
                MethodAuthorization::Public
            );
        }


        method_states.insert(
            "enable_flags".to_string(),
            MethodAuthorization::Protected(HardAuthRule::AllOf(vec![]))
        );
        method_states.insert(
            "disable_flags".to_string(),
            MethodAuthorization::Protected(HardAuthRule::AllOf(vec![]))
        );
        method_states.insert(
            "lock_flags".to_string(),
            MethodAuthorization::Protected(HardAuthRule::AllOf(vec![]))
        );

        if flags & SHARED_METADATA_MUTABLE > 0 {
            method_states.insert(
                "update_metadata".to_string(),
                MethodAuthorization::Protected(HardAuthRule::AllOf(vec![]))
            );
        }

        if flags & INDIVIDUAL_METADATA_MUTABLE > 0 {
            method_states.insert(
                "update_non_fungible_mutable_data".to_string(),
                MethodAuthorization::Protected(HardAuthRule::AllOf(vec![]))
            );
        }

        for (resource_def_id, permission) in authorities {
            if !resource_permissions_are_valid(permission) {
                return Err(ResourceDefError::InvalidResourcePermission(permission));
            }

            for (flag, methods) in PERMISSION_MAP.iter() {
                if permission & flag != 0 {
                    for method in methods.iter() {
                        let method_auth_maybe = method_states.remove(*method);
                        if let None = method_auth_maybe {
                            continue;
                        }
                        let mut cur_auth = method_auth_maybe.unwrap();
                        cur_auth = match cur_auth {
                            MethodAuthorization::Public => MethodAuthorization::Public,
                            MethodAuthorization::Protected(HardAuthRule::AllOf(_)) => {
                                MethodAuthorization::Protected(HardAuthRule::ProofRule(HardProofRule::AnyOf(
                                    HardProofRuleResourceList::List(vec![resource_def_id.into()])
                                )))
                            }
                            MethodAuthorization::Protected(HardAuthRule::ProofRule(
                                HardProofRule::AnyOf(HardProofRuleResourceList::List(
                                    mut resources,
                                )),
                            )) => {
                                resources.push(resource_def_id.into());
                                MethodAuthorization::Protected(HardAuthRule::ProofRule(
                                    HardProofRule::AnyOf(HardProofRuleResourceList::List(
                                        resources,
                                    )),
                                ))
                            }
                            _ => panic!("Should never get here."),
                        };

                        method_states.insert(method.to_string(), cur_auth);
                    }
                }
            }
        }

        let resource_def = Self {
            resource_type,
            metadata,
            flags,
            mutable_flags,
            authorization: method_states,
            total_supply: 0.into(),
        };

        Ok(resource_def)
    }

    pub fn get_auth(&self, method_name: &str) -> &MethodAuthorization {
        match self.authorization.get(method_name) {
            None => &MethodAuthorization::Unsupported,
            Some(authorization) => authorization,
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

    pub fn mint(&mut self, mint_params: &MintParams) -> Result<(), ResourceDefError> {
        // check resource type
        if !mint_params.matches_type(&self.resource_type) {
            return Err(ResourceDefError::ResourceTypeDoesNotMatch);
        }

        // check amount
        let amount = mint_params.amount();
        self.check_amount(amount)?;

        // It takes `1,701,411,835` mint operations to reach `Decimal::MAX`,
        // which will be impossible with metering.
        if amount > 100_000_000_000i128.into() {
            return Err(ResourceDefError::MaxMintAmountExceeded);
        }

        self.total_supply += amount;
        Ok(())
    }

    pub fn burn(&mut self, amount: Decimal) {
        self.total_supply -= amount;
    }

    pub fn update_metadata(
        &mut self,
        new_metadata: HashMap<String, String>,
    ) -> Result<(), ResourceDefError> {
        self.metadata = new_metadata;

        Ok(())
    }

    pub fn enable_flags(&mut self, flags: u64) -> Result<(), ResourceDefError> {
        if !resource_flags_are_valid(flags) {
            return Err(ResourceDefError::InvalidResourceFlags(flags));
        }

        if self.mutable_flags | flags != self.mutable_flags {
            return Err(ResourceDefError::FlagsLocked);
        }
        self.flags |= flags;

        Ok(())
    }

    pub fn disable_flags(&mut self, flags: u64) -> Result<(), ResourceDefError> {
        if !resource_flags_are_valid(flags) {
            return Err(ResourceDefError::InvalidResourceFlags(flags));
        }

        if self.mutable_flags | flags != self.mutable_flags {
            return Err(ResourceDefError::FlagsLocked);
        }
        self.flags &= !flags;

        Ok(())
    }

    pub fn lock_flags(&mut self, flags: u64) -> Result<(), ResourceDefError> {
        if !resource_flags_are_valid(flags) {
            return Err(ResourceDefError::InvalidResourceFlags(flags));
        }

        if self.mutable_flags | flags != self.mutable_flags {
            return Err(ResourceDefError::FlagsLocked);
        }
        self.mutable_flags &= !flags;

        Ok(())
    }

    pub fn check_amount(&self, amount: Decimal) -> Result<(), ResourceDefError> {
        let divisibility = self.resource_type.divisibility();

        if amount.is_negative() || amount.0 % 10i128.pow((18 - divisibility).into()) != 0.into() {
            Err(ResourceDefError::InvalidAmount(amount, divisibility))
        } else {
            Ok(())
        }
    }
}
