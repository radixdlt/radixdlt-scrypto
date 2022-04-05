use sbor::*;
use scrypto::engine::types::*;
use scrypto::resource::resource_flags::*;
use scrypto::resource::resource_permissions::*;
use scrypto::rust::collections::HashMap;
use scrypto::rust::mem;
use scrypto::rust::string::String;
use scrypto::rust::string::ToString;
use scrypto::rust::vec;

use crate::model::method_authorization::{HardAuthRule, HardProofRule, HardProofRuleResourceList};
use crate::model::resource_manager::FlagCondition::{AlwaysTrue, IsNotSet, IsSet};
use crate::model::MethodAuthorization;

/// Represents an error when accessing a bucket.
#[derive(Debug, Clone, PartialEq)]
pub enum ResourceManagerError {
    InvalidDivisibility,
    InvalidAmount(Decimal, u8),
    InvalidResourceFlags(u64),
    InvalidResourcePermission(u64),
    FlagsLocked,
    ResourceTypeDoesNotMatch,
    MaxMintAmountExceeded,
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
    auth: MethodAuthorization,
}

impl MethodState {
    fn new(enabled: FlagCondition, use_auth: FlagCondition) -> Self {
        MethodState {
            enabled,
            use_auth,
            auth: MethodAuthorization::Public,
        }
    }

    fn get_auth(&self, flags: u64) -> &MethodAuthorization {
        if !self.is_enabled(flags) {
            &MethodAuthorization::Private
        } else if self.use_auth(flags) {
            &self.auth
        } else {
            &MethodAuthorization::Public
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
pub struct ResourceManager {
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
        &["enable_flags", "disable_flags", "lock_flags"],
    ),
    (MAY_CHANGE_SHARED_METADATA, &["update_metadata"]),
    (
        MAY_CHANGE_INDIVIDUAL_METADATA,
        &["update_non_fungible_mutable_data"],
    ),
];

impl ResourceManager {
    pub fn new(
        resource_type: ResourceType,
        metadata: HashMap<String, String>,
        flags: u64,
        mutable_flags: u64,
        authorities: HashMap<ResourceAddress, u64>,
    ) -> Result<Self, ResourceManagerError> {
        if !resource_flags_are_valid(flags) {
            return Err(ResourceManagerError::InvalidResourceFlags(flags));
        }

        if !resource_flags_are_valid(mutable_flags) {
            return Err(ResourceManagerError::InvalidResourceFlags(mutable_flags));
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
            "enable_flags".to_string(),
            MethodState::new(AlwaysTrue, AlwaysTrue),
        );
        method_states.insert(
            "disable_flags".to_string(),
            MethodState::new(AlwaysTrue, AlwaysTrue),
        );
        method_states.insert(
            "lock_flags".to_string(),
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

        for (resource_address, permission) in authorities {
            if !resource_permissions_are_valid(permission) {
                return Err(ResourceManagerError::InvalidResourcePermission(permission));
            }

            for (flag, methods) in PERMISSION_MAP.iter() {
                if permission & flag != 0 {
                    for method in methods.iter() {
                        let method_state = method_states.get_mut(*method).unwrap();
                        let cur_rule =
                            mem::replace(&mut method_state.auth, MethodAuthorization::Public);
                        method_state.auth = match cur_rule {
                            MethodAuthorization::Public => MethodAuthorization::Protected(
                                HardAuthRule::ProofRule(HardProofRule::AnyOf(
                                    HardProofRuleResourceList::List(vec![resource_address.into()]),
                                )),
                            ),
                            MethodAuthorization::Protected(HardAuthRule::ProofRule(
                                HardProofRule::AnyOf(HardProofRuleResourceList::List(
                                    mut resources,
                                )),
                            )) => {
                                resources.push(resource_address.into());
                                MethodAuthorization::Protected(HardAuthRule::ProofRule(
                                    HardProofRule::AnyOf(HardProofRuleResourceList::List(
                                        resources,
                                    )),
                                ))
                            }
                            _ => panic!("Should never get here."),
                        };
                    }
                }
            }
        }

        let resource_manager = Self {
            resource_type,
            metadata,
            flags,
            mutable_flags,
            method_states,
            total_supply: 0.into(),
        };

        Ok(resource_manager)
    }

    pub fn get_auth(&self, method_name: &str) -> &MethodAuthorization {
        match self.method_states.get(method_name) {
            None => &MethodAuthorization::Unsupported,
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

    pub fn mint(&mut self, mint_params: &MintParams) -> Result<(), ResourceManagerError> {
        // check resource type
        if !mint_params.matches_type(&self.resource_type) {
            return Err(ResourceManagerError::ResourceTypeDoesNotMatch);
        }

        // check amount
        let amount = mint_params.amount();
        self.check_amount(amount)?;

        // It takes `1,701,411,835` mint operations to reach `Decimal::MAX`,
        // which will be impossible with metering.
        if amount > 100_000_000_000i128.into() {
            return Err(ResourceManagerError::MaxMintAmountExceeded);
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
    ) -> Result<(), ResourceManagerError> {
        self.metadata = new_metadata;

        Ok(())
    }

    pub fn enable_flags(&mut self, flags: u64) -> Result<(), ResourceManagerError> {
        if !resource_flags_are_valid(flags) {
            return Err(ResourceManagerError::InvalidResourceFlags(flags));
        }

        if self.mutable_flags | flags != self.mutable_flags {
            return Err(ResourceManagerError::FlagsLocked);
        }
        self.flags |= flags;

        Ok(())
    }

    pub fn disable_flags(&mut self, flags: u64) -> Result<(), ResourceManagerError> {
        if !resource_flags_are_valid(flags) {
            return Err(ResourceManagerError::InvalidResourceFlags(flags));
        }

        if self.mutable_flags | flags != self.mutable_flags {
            return Err(ResourceManagerError::FlagsLocked);
        }
        self.flags &= !flags;

        Ok(())
    }

    pub fn lock_flags(&mut self, flags: u64) -> Result<(), ResourceManagerError> {
        if !resource_flags_are_valid(flags) {
            return Err(ResourceManagerError::InvalidResourceFlags(flags));
        }

        if self.mutable_flags | flags != self.mutable_flags {
            return Err(ResourceManagerError::FlagsLocked);
        }
        self.mutable_flags &= !flags;

        Ok(())
    }

    pub fn check_amount(&self, amount: Decimal) -> Result<(), ResourceManagerError> {
        let divisibility = self.resource_type.divisibility();

        if amount.is_negative() || amount.0 % 10i128.pow((18 - divisibility).into()) != 0.into() {
            Err(ResourceManagerError::InvalidAmount(amount, divisibility))
        } else {
            Ok(())
        }
    }
}
