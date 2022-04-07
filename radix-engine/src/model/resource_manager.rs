use sbor::*;
use scrypto::engine::types::*;
use scrypto::prelude::ResourceMethod::TakeFromVault;
use scrypto::rust::collections::HashMap;
use scrypto::rust::string::String;
use scrypto::rust::string::ToString;
use scrypto::resource::*;
use scrypto::resource::ResourceMethod::{Burn, Mint, UpdateMetadata, UpdateNonFungibleData};

use crate::model::{convert, MethodAuthorization};

/// Represents an error when accessing a bucket.
#[derive(Debug, Clone, PartialEq)]
pub enum ResourceManagerError {
    InvalidDivisibility,
    InvalidAmount(Decimal, u8),
    InvalidResourceFlags(u64),
    InvalidMintPermission,
    TakeFromVaultNotDefined,
    ResourceTypeDoesNotMatch,
    MaxMintAmountExceeded,
}

/// The definition of a resource.
#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct ResourceManager {
    resource_type: ResourceType,
    metadata: HashMap<String, String>,
    authorization: HashMap<String, MethodAuthorization>,
    total_supply: Decimal,
}

impl ResourceManager {
    pub fn new(
        resource_type: ResourceType,
        metadata: HashMap<String, String>,
        auth: HashMap<ResourceMethod, MethodAuth>,
    ) -> Result<Self, ResourceManagerError> {
        let mut authorization: HashMap<String, MethodAuthorization> = HashMap::new();
        if let Some(mint_auth) = auth.get(&Mint) {
            // TODO: Check for other invalid mint permissions?
            if let MethodAuth::AllowAll = mint_auth {
                return Err(ResourceManagerError::InvalidMintPermission);
            }

            authorization.insert(
                "mint".to_string(),
                convert(&Type::Unit, &Value::Unit, mint_auth),
            );
        }

        if let Some(burn_auth) = auth.get(&Burn) {
            authorization.insert(
                "burn".to_string(),
                convert(&Type::Unit, &Value::Unit, burn_auth),
            );
        }

        if let Some(take_auth) = auth.get(&TakeFromVault) {
            authorization.insert(
                "take_from_vault".to_string(),
                convert(&Type::Unit, &Value::Unit, take_auth),
            );
            if let ResourceType::NonFungible = resource_type {
                authorization.insert(
                    "take_non_fungibles_from_vault".to_string(),
                    convert(&Type::Unit, &Value::Unit, take_auth),
                );
            }
        } else {
            return Err(ResourceManagerError::TakeFromVaultNotDefined);
        }

        if let Some(update_metadata_auth) = auth.get(&UpdateMetadata) {
            authorization.insert(
                "update_metadata".to_string(),
                convert(&Type::Unit, &Value::Unit, update_metadata_auth),
            );
        }

        if let Some(update_non_fungible_mutable_data_auth) =
            auth.get(&UpdateNonFungibleData)
        {
            authorization.insert(
                "update_non_fungible_mutable_data".to_string(),
                convert(
                    &Type::Unit,
                    &Value::Unit,
                    update_non_fungible_mutable_data_auth,
                ),
            );
        }

        let resource_manager = Self {
            resource_type,
            metadata,
            authorization,
            total_supply: 0.into(),
        };

        Ok(resource_manager)
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

    pub fn total_supply(&self) -> Decimal {
        self.total_supply
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

    pub fn check_amount(&self, amount: Decimal) -> Result<(), ResourceManagerError> {
        let divisibility = self.resource_type.divisibility();

        if amount.is_negative() || amount.0 % 10i128.pow((18 - divisibility).into()) != 0.into() {
            Err(ResourceManagerError::InvalidAmount(amount, divisibility))
        } else {
            Ok(())
        }
    }
}
