use crate::engine::SystemApi;
use crate::model::NonFungible;
use sbor::*;
use scrypto::buffer::scrypto_decode;
use scrypto::engine::types::*;
use scrypto::prelude::ResourceMethod::TakeFromVault;
use scrypto::resource::ResourceMethod::{Burn, Mint, UpdateMetadata, UpdateNonFungibleData};
use scrypto::resource::*;
use scrypto::resource::Mutability::LOCKED;
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
    CouldNotCreateBucket,
}

/// The definition of a resource.
#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct ResourceManager {
    resource_type: ResourceType,
    metadata: HashMap<String, String>,
    authorization: HashMap<String, (MethodAuthorization, Mutability)>,
    total_supply: Decimal,
}

impl ResourceManager {
    pub fn new(
        resource_type: ResourceType,
        metadata: HashMap<String, String>,
        mut auth: HashMap<ResourceMethod, (MethodAuth, Mutability)>,
    ) -> Result<Self, ResourceManagerError> {
        let mut authorization: HashMap<String, (MethodAuthorization, Mutability)> = HashMap::new();
        if let Some((mint_auth, mutability)) = auth.remove(&Mint) {
            let converted_mint_auth = convert(&Type::Unit, &Value::Unit, &mint_auth);
            authorization.insert("mint".to_string(), (converted_mint_auth, mutability));
        }

        if let Some((burn_auth, mutability)) = auth.remove(&Burn) {
            let converted_burn_auth = convert(&Type::Unit, &Value::Unit, &burn_auth);
            authorization.insert("burn".to_string(), (converted_burn_auth, mutability));
        }

        if let Some((take_auth, mutability)) = auth.remove(&TakeFromVault) {
            let converted_take_auth = convert(&Type::Unit, &Value::Unit, &take_auth);
            authorization.insert("take_from_vault".to_string(), (converted_take_auth.clone(), mutability.clone()));

            if let ResourceType::NonFungible = resource_type {
                authorization.insert(
                    "take_non_fungibles_from_vault".to_string(),
                    (converted_take_auth, mutability)
                );
            }
        }

        if let Some((update_metadata_auth, mutability)) = auth.remove(&UpdateMetadata) {
            let converted_update_metadata_auth = convert(&Type::Unit, &Value::Unit, &update_metadata_auth);
            authorization.insert(
                "update_metadata".to_string(),
                (converted_update_metadata_auth, mutability),
            );
        }

        if let Some((update_non_fungible_mutable_data_auth, mutability)) = auth.remove(&UpdateNonFungibleData) {
            let converted_auth = convert(
                &Type::Unit,
                &Value::Unit,
                &update_non_fungible_mutable_data_auth,
            );
            authorization.insert(
                "update_non_fungible_mutable_data".to_string(),
                (converted_auth, mutability),
            );
        }

        for pub_method in ["get_metadata", "get_resource_type", "get_total_supply"] {
            authorization.insert(pub_method.to_string(), (MethodAuthorization::AllowAll, LOCKED));
        }

        if let ResourceType::NonFungible = resource_type {
            authorization.insert(
                "non_fungible_exists".to_string(),
                (MethodAuthorization::AllowAll, LOCKED),
            );
            authorization.insert("get_non_fungible".to_string(), (MethodAuthorization::AllowAll, LOCKED));
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
            Some((authorization, _)) => authorization,
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

    fn mint<S: SystemApi>(
        &mut self,
        mint_params: MintParams,
        self_address: ResourceAddress,
        system_api: &mut S,
    ) -> Result<ResourceContainer, ResourceManagerError> {
        match mint_params {
            MintParams::Fungible { amount } => self.mint_fungible(amount, self_address),
            MintParams::NonFungible { entries } => {
                self.mint_non_fungibles(entries, self_address, system_api)
            }
        }
    }

    pub fn mint_fungible(
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

    fn mint_non_fungibles<S: SystemApi>(
        &mut self,
        entries: HashMap<NonFungibleId, (Vec<u8>, Vec<u8>)>,
        self_address: ResourceAddress,
        system_api: &mut S,
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
            if system_api.get_non_fungible(&non_fungible_address).is_some() {
                return Err(ResourceManagerError::NonFungibleAlreadyExists(
                    non_fungible_address,
                ));
            }

            let immutable_data = Self::process_non_fungible_data(&data.0)?;
            let mutable_data = Self::process_non_fungible_data(&data.1)?;

            system_api.put_non_fungible(
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

    pub fn static_main<S: SystemApi>(
        function: &str,
        args: Vec<ScryptoValue>,
        system_api: &mut S,
    ) -> Result<ScryptoValue, ResourceManagerError> {
        match function {
            "create" => {
                let resource_type = scrypto_decode(&args[0].raw)
                    .map_err(ResourceManagerError::InvalidRequestData)?;
                let metadata = scrypto_decode(&args[1].raw)
                    .map_err(ResourceManagerError::InvalidRequestData)?;
                let auth = scrypto_decode(&args[2].raw)
                    .map_err(ResourceManagerError::InvalidRequestData)?;
                let mint_params_maybe: Option<MintParams> = scrypto_decode(&args[3].raw)
                    .map_err(ResourceManagerError::InvalidRequestData)?;
                let resource_manager = ResourceManager::new(resource_type, metadata, auth)?;
                let resource_address = system_api.create_resource(resource_manager);

                let bucket_id = if let Some(mint_params) = mint_params_maybe {
                    let mut resource_manager = system_api
                        .borrow_global_mut_resource_manager(resource_address)
                        .unwrap();
                    let container =
                        resource_manager.mint(mint_params, resource_address, system_api)?;
                    system_api.return_borrowed_global_resource_manager(
                        resource_address,
                        resource_manager,
                    );

                    let bucket_id = system_api
                        .create_bucket(container)
                        .map_err(|_| ResourceManagerError::CouldNotCreateBucket)?;
                    Some(scrypto::resource::Bucket(bucket_id))
                } else {
                    None
                };

                Ok(ScryptoValue::from_value(&(resource_address, bucket_id)))
            }
            _ => Err(ResourceManagerError::MethodNotFound(function.to_string())),
        }
    }

    pub fn main<S: SystemApi>(
        &mut self,
        resource_address: ResourceAddress,
        function: &str,
        args: Vec<ScryptoValue>,
        system_api: &mut S,
    ) -> Result<ScryptoValue, ResourceManagerError> {
        match function {
            "mint" => {
                // TODO: cleanup
                let mint_params: MintParams = scrypto_decode(&args[0].raw)
                    .map_err(|e| ResourceManagerError::InvalidRequestData(e))?;
                let container = self.mint(mint_params, resource_address, system_api)?;
                let bucket_id = system_api
                    .create_bucket(container)
                    .map_err(|_| ResourceManagerError::CouldNotCreateBucket)?;
                Ok(ScryptoValue::from_value(&scrypto::resource::Bucket(
                    bucket_id,
                )))
            }
            "get_metadata" => Ok(ScryptoValue::from_value(&self.metadata)),
            "get_resource_type" => Ok(ScryptoValue::from_value(&self.resource_type)),
            "get_total_supply" => Ok(ScryptoValue::from_value(&self.total_supply)),
            "update_metadata" => {
                let new_metadata: HashMap<String, String> = scrypto_decode(&args[0].raw)
                    .map_err(|e| ResourceManagerError::InvalidRequestData(e))?;
                self.update_metadata(new_metadata)?;
                Ok(ScryptoValue::from_value(&()))
            }
            "update_non_fungible_mutable_data" => {
                let non_fungible_id: NonFungibleId = scrypto_decode(&args[0].raw)
                    .map_err(|e| ResourceManagerError::InvalidRequestData(e))?;
                let new_mutable_data: Vec<u8> = scrypto_decode(&args[1].raw)
                    .map_err(|e| ResourceManagerError::InvalidRequestData(e))?;

                let non_fungible_address =
                    NonFungibleAddress::new(resource_address.clone(), non_fungible_id);
                let data = Self::process_non_fungible_data(&new_mutable_data)?;
                system_api
                    .get_non_fungible_mut(&non_fungible_address)
                    .ok_or(ResourceManagerError::NonFungibleNotFound(
                        non_fungible_address,
                    ))?
                    .set_mutable_data(data.raw);
                Ok(ScryptoValue::from_value(&()))
            }
            "non_fungible_exists" => {
                let non_fungible_id: NonFungibleId = scrypto_decode(&args[0].raw)
                    .map_err(|e| ResourceManagerError::InvalidRequestData(e))?;
                let non_fungible_address =
                    NonFungibleAddress::new(resource_address.clone(), non_fungible_id);
                let non_fungible = system_api.get_non_fungible(&non_fungible_address);
                Ok(ScryptoValue::from_value(&non_fungible.is_some()))
            }
            "get_non_fungible" => {
                let non_fungible_id: NonFungibleId = scrypto_decode(&args[0].raw)
                    .map_err(|e| ResourceManagerError::InvalidRequestData(e))?;
                let non_fungible_address =
                    NonFungibleAddress::new(resource_address.clone(), non_fungible_id);
                let non_fungible = system_api.get_non_fungible(&non_fungible_address).ok_or(
                    ResourceManagerError::NonFungibleNotFound(non_fungible_address),
                )?;
                Ok(ScryptoValue::from_value(&[
                    non_fungible.immutable_data(),
                    non_fungible.mutable_data(),
                ]))
            }
            _ => Err(ResourceManagerError::MethodNotFound(function.to_string())),
        }
    }
}
