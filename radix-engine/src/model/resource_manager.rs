use crate::engine::Track;
use crate::ledger::SubstateStore;
use crate::model::Bucket;
use crate::model::NonFungible;
use sbor::*;
use scrypto::buffer::scrypto_decode;
use scrypto::engine::types::*;
use scrypto::prelude::ResourceMethod::TakeFromVault;
use scrypto::resource::ResourceMethod::{Burn, Mint, UpdateMetadata, UpdateNonFungibleData};
use scrypto::resource::*;
use scrypto::rust::collections::*;
use scrypto::rust::string::String;
use scrypto::rust::string::ToString;
use scrypto::rust::vec::*;
use scrypto::values::ScryptoValue;

use crate::model::{convert, MethodAuthorization, ResourceContainer};

/// Represents an error when accessing a bucket.
#[derive(Debug, Clone, PartialEq)]
pub enum ResourceManagerError {
    InvalidDivisibility,
    InvalidAmount(Decimal, u8),
    InvalidResourceFlags(u64),
    InvalidMintPermission,
    ResourceTypeDoesNotMatch,
    MaxMintAmountExceeded,
    InvalidNonFungibleData,
    NonFungibleAlreadyExists(NonFungibleAddress),
    NonFungibleNotFound(NonFungibleAddress),
    InvalidRequestData(DecodeError),
    MethodNotFound(String),
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
        }

        if let Some(update_metadata_auth) = auth.get(&UpdateMetadata) {
            authorization.insert(
                "update_metadata".to_string(),
                convert(&Type::Unit, &Value::Unit, update_metadata_auth),
            );
        }

        if let Some(update_non_fungible_mutable_data_auth) = auth.get(&UpdateNonFungibleData) {
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

    pub fn mint(
        &mut self,
        amount: Decimal,
        self_address: ResourceAddress,
    ) -> Result<ResourceContainer, ResourceManagerError> {
        if let ResourceType::Fungible { divisibility } = self.resource_type {
            // check amount
            self.check_amount(amount)?;

            // It takes `1,701,411,835` mint operations to reach `Decimal::MAX`,
            // which will be impossible with metering.
            if amount > 100_000_000_000i128.into() {
                return Err(ResourceManagerError::MaxMintAmountExceeded);
            }

            self.total_supply += amount;

            Ok(ResourceContainer::new_fungible(
                self_address,
                divisibility,
                amount,
            ))
        } else {
            Err(ResourceManagerError::ResourceTypeDoesNotMatch)
        }
    }

    fn process_non_fungible_data(data: &[u8]) -> Result<ScryptoValue, ResourceManagerError> {
        let validated = ScryptoValue::from_slice(data)
            .map_err(|_| ResourceManagerError::InvalidNonFungibleData)?;
        if !validated.bucket_ids.is_empty() {
            return Err(ResourceManagerError::InvalidNonFungibleData);
        }
        if !validated.proof_ids.is_empty() {
            return Err(ResourceManagerError::InvalidNonFungibleData);
        }
        if !validated.lazy_map_ids.is_empty() {
            return Err(ResourceManagerError::InvalidNonFungibleData);
        }
        if !validated.vault_ids.is_empty() {
            return Err(ResourceManagerError::InvalidNonFungibleData);
        }
        Ok(validated)
    }

    fn mint_non_fungibles<'s, S: SubstateStore>(
        &mut self,
        entries: HashMap<NonFungibleId, (Vec<u8>, Vec<u8>)>,
        self_address: ResourceAddress,
        track: &mut Track<'s, S>,
    ) -> Result<ResourceContainer, ResourceManagerError> {
        // check resource type
        if !matches!(self.resource_type, ResourceType::NonFungible) {
            return Err(ResourceManagerError::ResourceTypeDoesNotMatch);
        }

        // check amount
        let amount = entries.len().into();
        self.check_amount(amount)?;

        // It takes `1,701,411,835` mint operations to reach `Decimal::MAX`,
        // which will be impossible with metering.
        if amount > 100_000_000_000i128.into() {
            return Err(ResourceManagerError::MaxMintAmountExceeded);
        }

        self.total_supply += amount;

        // Allocate non-fungibles
        let mut ids = BTreeSet::new();
        for (id, data) in entries {
            let non_fungible_address = NonFungibleAddress::new(self_address, id.clone());
            if track.get_non_fungible(&non_fungible_address).is_some() {
                return Err(ResourceManagerError::NonFungibleAlreadyExists(
                    non_fungible_address,
                ));
            }

            let immutable_data = Self::process_non_fungible_data(&data.0)?;
            let mutable_data = Self::process_non_fungible_data(&data.1)?;

            track.put_non_fungible(
                non_fungible_address,
                NonFungible::new(immutable_data.raw, mutable_data.raw),
            );
            ids.insert(id);
        }

        Ok(ResourceContainer::new_non_fungible(self_address, ids))
    }

    pub fn burn(&mut self, amount: Decimal) {
        self.total_supply -= amount;
    }

    fn update_metadata(
        &mut self,
        new_metadata: HashMap<String, String>,
    ) -> Result<(), ResourceManagerError> {
        self.metadata = new_metadata;

        Ok(())
    }

    fn check_amount(&self, amount: Decimal) -> Result<(), ResourceManagerError> {
        let divisibility = self.resource_type.divisibility();

        if amount.is_negative() || amount.0 % 10i128.pow((18 - divisibility).into()) != 0.into() {
            Err(ResourceManagerError::InvalidAmount(amount, divisibility))
        } else {
            Ok(())
        }
    }

    pub fn main<'s, S: SubstateStore>(
        &mut self,
        resource_address: ResourceAddress,
        function: &str,
        args: Vec<ScryptoValue>,
        track: &mut Track<'s, S>,
    ) -> Result<Option<Bucket>, ResourceManagerError> {
        match function {
            "mint" => {
                // TODO: cleanup
                let mint_params: MintParams = scrypto_decode(&args[0].raw)
                    .map_err(|e| ResourceManagerError::InvalidRequestData(e))?;
                let container = match mint_params {
                    MintParams::Fungible { amount } => self.mint(amount, resource_address.clone()),
                    MintParams::NonFungible { entries } => {
                        self.mint_non_fungibles(entries, resource_address.clone(), track)
                    }
                }?;

                Ok(Option::Some(Bucket::new(container)))
            }
            "update_metadata" => {
                let new_metadata: HashMap<String, String> = scrypto_decode(&args[0].raw)
                    .map_err(|e| ResourceManagerError::InvalidRequestData(e))?;
                self.update_metadata(new_metadata)?;
                Ok(Option::None)
            }
            "update_non_fungible_mutable_data" => {
                let non_fungible_id: NonFungibleId = scrypto_decode(&args[0].raw)
                    .map_err(|e| ResourceManagerError::InvalidRequestData(e))?;
                let new_mutable_data: Vec<u8> = scrypto_decode(&args[1].raw)
                    .map_err(|e| ResourceManagerError::InvalidRequestData(e))?;

                let non_fungible_address =
                    NonFungibleAddress::new(resource_address.clone(), non_fungible_id);
                let data = Self::process_non_fungible_data(&new_mutable_data)?;
                track
                    .get_non_fungible_mut(&non_fungible_address)
                    .ok_or(ResourceManagerError::NonFungibleNotFound(
                        non_fungible_address,
                    ))?
                    .set_mutable_data(data.raw);
                Ok(Option::None)
            }
            _ => Err(ResourceManagerError::MethodNotFound(function.to_string())),
        }
    }
}
