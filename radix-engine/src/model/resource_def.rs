use sbor::*;
use scrypto::engine::types::*;
use scrypto::prelude::ToString;
use scrypto::resource::resource_flags::*;
use scrypto::resource::resource_permissions::*;
use scrypto::rust::collections::HashMap;
use scrypto::rust::mem;
use scrypto::rust::string::String;

use crate::model::auth_rule::Rule;
use crate::model::resource_def::FlagCondition::{AlwaysTrue, IsNotSet, IsSet};
use crate::model::AuthRule;

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

#[derive(Debug, Clone, TypeId, Encode, Decode)]
enum FlagCondition {
    IsSet(u64),
    IsNotSet(u64),
    AlwaysTrue,
}

impl FlagCondition {
    fn matches(&self, flags: u64) -> bool {
        match self {
            IsSet(flag) => flags & flag > 0,
            IsNotSet(flag) => flags & flag == 0,
            AlwaysTrue => true,
        }
    }
}

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct MethodState {
    enabled: FlagCondition,
    use_auth: FlagCondition,
    auth_rule: AuthRule,
}

impl MethodState {
    fn new(enabled: FlagCondition, use_auth: FlagCondition) -> Self {
        MethodState {
            enabled,
            use_auth,
            auth_rule: AuthRule::Public,
        }
    }

    fn get_auth(&self, flags: u64) -> &AuthRule {
        if !self.is_enabled(flags) {
            &AuthRule::Private
        } else if self.use_auth(flags) {
            &self.auth_rule
        } else {
            &AuthRule::Public
        }
    }

    fn is_enabled(&self, flags: u64) -> bool {
        self.enabled.matches(flags)
    }

    fn use_auth(&self, flags: u64) -> bool {
        self.use_auth.matches(flags)
    }
}

/// The definition of a resource.
#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct ResourceDef {
    resource_type: ResourceType,
    metadata: HashMap<String, String>,
    flags: u64,
    mutable_flags: u64,
    method_states: HashMap<String, MethodState>,
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
    ) -> Result<Self, ResourceDefError> {
        if !resource_flags_are_valid(flags) {
            return Err(ResourceDefError::InvalidResourceFlags(flags));
        }

        if !resource_flags_are_valid(mutable_flags) {
            return Err(ResourceDefError::InvalidResourceFlags(mutable_flags));
        }

        let mut method_states: HashMap<String, MethodState> = HashMap::new();
        method_states.insert(
            "mint".to_string(),
            MethodState::new(IsSet(MINTABLE), AlwaysTrue),
        );
        method_states.insert(
            "burn".to_string(),
            MethodState::new(IsSet(BURNABLE), IsNotSet(FREELY_BURNABLE)),
        );
        method_states.insert(
            "take_from_vault".to_string(),
            MethodState::new(AlwaysTrue, IsSet(RESTRICTED_TRANSFER)),
        );
        method_states.insert(
            "update_flags".to_string(),
            MethodState::new(AlwaysTrue, AlwaysTrue),
        );
        method_states.insert(
            "update_mutable_flags".to_string(),
            MethodState::new(AlwaysTrue, AlwaysTrue),
        );
        method_states.insert(
            "update_metadata".to_string(),
            MethodState::new(IsSet(SHARED_METADATA_MUTABLE), AlwaysTrue),
        );
        method_states.insert(
            "update_non_fungible_mutable_data".to_string(),
            MethodState::new(IsSet(INDIVIDUAL_METADATA_MUTABLE), AlwaysTrue),
        );

        for (resource_def_id, permission) in authorities {
            if !resource_permissions_are_valid(permission) {
                return Err(ResourceDefError::InvalidResourcePermission(permission));
            }

            for (flag, methods) in PERMISSION_MAP.iter() {
                if permission & flag != 0 {
                    for method in methods.iter() {
                        let method_state = method_states.get_mut(*method).unwrap();
                        let cur_rule = mem::replace(&mut method_state.auth_rule, AuthRule::Public);
                        let new_rule = Rule::AnyOfResource(resource_def_id);
                        method_state.auth_rule = match cur_rule {
                            AuthRule::Public => AuthRule::Protected(new_rule),
                            AuthRule::Protected(rule) => AuthRule::Protected(rule.or(new_rule)),
                            _ => panic!("Should never get here."),
                        };
                    }
                }
            }
        }

        let resource_def = Self {
            resource_type,
            metadata,
            flags,
            mutable_flags,
            method_states,
            total_supply: 0.into(),
        };

        Ok(resource_def)
    }

    pub fn get_auth(&self, method_name: &str) -> &AuthRule {
        match self.method_states.get(method_name) {
            None => &AuthRule::Unsupported,
            Some(method_state) => method_state.get_auth(self.flags),
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
}
