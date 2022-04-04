use sbor::*;
use sbor::any::Value;
use scrypto::engine::types::*;
use scrypto::prelude::{ComponentAuthorization, MethodAuth, ToString};
use scrypto::resource::resource_flags::*;
use scrypto::rust::collections::HashMap;
use scrypto::rust::string::String;

use crate::model::{convert, MethodAuthorization};

/// Represents an error when accessing a bucket.
#[derive(Debug, Clone, PartialEq)]
pub enum ResourceDefError {
    InvalidDivisibility,
    InvalidAmount(Decimal, u8),
    InvalidResourceFlags(u64),
    InvalidMintPermission,
    TakeFromVaultNotDefined,
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

impl ResourceDef {
    pub fn new(
        resource_type: ResourceType,
        metadata: HashMap<String, String>,
        flags: u64,
        mutable_flags: u64,
        auth: ComponentAuthorization
    ) -> Result<Self, ResourceDefError> {
        if !resource_flags_are_valid(flags) {
            return Err(ResourceDefError::InvalidResourceFlags(flags));
        }

        if !resource_flags_are_valid(mutable_flags) {
            return Err(ResourceDefError::InvalidResourceFlags(mutable_flags));
        }

        let mut authorization: HashMap<String, MethodAuthorization> = HashMap::new();
        if let Some(mint_auth) = auth.get("mint") {
            // TODO: Check for other invalid mint permissions?
            if let MethodAuth::AllowAll = mint_auth {
                return Err(ResourceDefError::InvalidMintPermission);
            }

            authorization.insert("mint".to_string(), convert(&Type::Unit, &Value::Unit, mint_auth));
        }

        if let Some(burn_auth) = auth.get("burn") {
            authorization.insert("burn".to_string(), convert(&Type::Unit, &Value::Unit, burn_auth));
        }

        if let Some(take_auth) = auth.get("take_from_vault") {
            authorization.insert("take_from_vault".to_string(), convert(&Type::Unit, &Value::Unit, take_auth));
        } else {
            return Err(ResourceDefError::TakeFromVaultNotDefined);
        }

        if let Some(update_metadata_auth) = auth.get("update_metadata") {
            authorization.insert("update_metadata".to_string(), convert(&Type::Unit, &Value::Unit, update_metadata_auth));
        }

        if let Some(update_non_fungible_mutable_data_auth) = auth.get("update_non_fungible_mutable_data") {
            authorization.insert("update_non_fungible_mutable_data".to_string(), convert(&Type::Unit, &Value::Unit, update_non_fungible_mutable_data_auth));
        }

        let resource_def = Self {
            resource_type,
            metadata,
            flags,
            mutable_flags,
            authorization,
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
