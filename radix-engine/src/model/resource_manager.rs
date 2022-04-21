use crate::engine::SystemApi;
use crate::model::NonFungible;
use sbor::*;
use scrypto::buffer::scrypto_decode;
use scrypto::engine::types::*;
use scrypto::prelude::MethodAuth::{AllowAll, DenyAll};
use scrypto::prelude::ResourceMethod::Withdraw;
use scrypto::resource::Mutability::LOCKED;
use scrypto::resource::ResourceMethod::{Burn, Mint, UpdateMetadata, UpdateNonFungibleData};
use scrypto::resource::*;
use scrypto::rust::collections::*;
use scrypto::rust::string::String;
use scrypto::rust::string::ToString;
use scrypto::rust::vec::*;
use scrypto::values::ScryptoValue;

use crate::model::{convert, MethodAuthorization, ResourceContainer};

/// Converts soft authorization rule to a hard authorization rule.
/// Currently required as all auth is defined by soft authorization rules.
macro_rules! convert_auth {
    ($auth:expr) => {
        convert(&Type::Unit, &Value::Unit, &$auth)
    };
}

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
    CouldNotCreateVault,
}

#[derive(Debug, Clone, TypeId, Encode, Decode)]
struct MethodEntry {
    auth: MethodAuthorization,
    update_auth: MethodAuthorization,
}

impl MethodEntry {
    pub fn new(entry: (MethodAuth, Mutability)) -> Self {
        MethodEntry {
            auth: convert_auth!(entry.0),
            update_auth: match entry.1 {
                Mutability::LOCKED => MethodAuthorization::DenyAll,
                Mutability::MUTABLE(method_auth) => convert_auth!(method_auth),
            },
        }
    }

    pub fn get_method_auth(&self) -> &MethodAuthorization {
        &self.auth
    }

    pub fn get_update_auth(&self, args: &[ScryptoValue]) -> &MethodAuthorization {
        let method: String = match scrypto_decode(&args[0].raw) {
            Ok(m) => m,
            _ => return &MethodAuthorization::Unsupported,
        };
        match method.as_str() {
            "lock" | "update" => &self.update_auth,
            _ => &MethodAuthorization::Unsupported,
        }
    }

    pub fn main(
        &mut self,
        method: &str,
        args: Vec<ScryptoValue>,
    ) -> Result<ScryptoValue, ResourceManagerError> {
        match method {
            "lock" => self.lock(),
            "update" => {
                let auth: MethodAuth = scrypto_decode(&args[0].raw)
                    .map_err(|e| ResourceManagerError::InvalidRequestData(e))?;
                self.update(auth);
            }
            _ => return Err(ResourceManagerError::MethodNotFound(method.to_string())),
        }

        Ok(ScryptoValue::from_value(&()))
    }

    fn update(&mut self, method_auth: MethodAuth) {
        self.auth = convert_auth!(method_auth)
    }

    fn lock(&mut self) {
        self.update_auth = MethodAuthorization::DenyAll;
    }
}

/// The definition of a resource.
#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct ResourceManager {
    resource_type: ResourceType,
    metadata: HashMap<String, String>,
    method_table: HashMap<String, Option<ResourceMethod>>,
    authorization: HashMap<ResourceMethod, MethodEntry>,
    total_supply: Decimal,
}

impl ResourceManager {
    pub fn new(
        resource_type: ResourceType,
        metadata: HashMap<String, String>,
        mut auth: HashMap<ResourceMethod, (MethodAuth, Mutability)>,
    ) -> Result<Self, ResourceManagerError> {
        let mut method_table: HashMap<String, Option<ResourceMethod>> = HashMap::new();
        method_table.insert("mint".to_string(), Some(Mint));
        method_table.insert("burn".to_string(), Some(Burn));
        method_table.insert("take_from_vault".to_string(), Some(Withdraw));
        method_table.insert("put_into_vault".to_string(), Some(Deposit));
        method_table.insert("update_metadata".to_string(), Some(UpdateMetadata));
        if let ResourceType::NonFungible = resource_type {
            method_table.insert("take_non_fungibles_from_vault".to_string(), Some(Withdraw));
        }

        for pub_method in [
            "create_bucket",
            "get_metadata",
            "get_resource_type",
            "get_total_supply",
            "create_vault",
            "get_vault_amount",
            "get_vault_resource_address",
            "create_vault_proof",
            "create_vault_proof_by_amount",
        ] {
            method_table.insert(pub_method.to_string(), None);
        }

        if let ResourceType::NonFungible = resource_type {
            method_table.insert(
                "update_non_fungible_mutable_data".to_string(),
                Some(UpdateNonFungibleData),
            );
            for pub_method in [
                "non_fungible_exists",
                "get_non_fungible",
                "get_non_fungible_ids_in_vault",
                "create_vault_proof_by_ids",
            ] {
                method_table.insert(pub_method.to_string(), None);
            }
        }

        let mut authorization: HashMap<ResourceMethod, MethodEntry> = HashMap::new();
        for (auth_entry_key, default) in [
            (Mint, (DenyAll, LOCKED)),
            (Burn, (DenyAll, LOCKED)),
            (Withdraw, (AllowAll, LOCKED)),
            (Deposit, (AllowAll, LOCKED)),
            (UpdateMetadata, (DenyAll, LOCKED)),
            (UpdateNonFungibleData, (DenyAll, LOCKED)),
        ] {
            let entry = auth.remove(&auth_entry_key).unwrap_or(default);
            authorization.insert(auth_entry_key, MethodEntry::new(entry));
        }

        let resource_manager = Self {
            resource_type,
            metadata,
            method_table,
            authorization,
            total_supply: 0.into(),
        };

        Ok(resource_manager)
    }

    pub fn get_consuming_bucket_auth(&self, arg: &ScryptoValue) -> &MethodAuthorization {
        let method = scrypto_decode(&arg.raw);
        match method {
            Err(_) => &MethodAuthorization::Unsupported,
            Ok(ConsumingBucketMethod::Burn()) => {
                match self.method_table.get("burn") {
                    None => &MethodAuthorization::Unsupported,
                    Some(None) => &MethodAuthorization::AllowAll,
                    Some(Some(method)) => self.authorization.get(method).unwrap().get_method_auth(),
                }
            }
        }
    }

    pub fn get_auth(&self, method_name: &str, args: &[ScryptoValue]) -> &MethodAuthorization {
        if method_name.eq("method_auth") {
            let method: ResourceMethod = match scrypto_decode(&args[0].raw) {
                Ok(r) => r,
                Err(_) => return &MethodAuthorization::Unsupported,
            };

            match self.authorization.get(&method) {
                None => &MethodAuthorization::Unsupported,
                Some(entry) => {
                    let auth_args = args.split_at(1).1;
                    entry.get_update_auth(auth_args)
                }
            }
        } else {
            match self.method_table.get(method_name) {
                None => &MethodAuthorization::Unsupported,
                Some(None) => &MethodAuthorization::AllowAll,
                Some(Some(method)) => self.authorization.get(method).unwrap().get_method_auth(),
            }
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
            let non_fungible = NonFungible::new(immutable_data.raw, mutable_data.raw);

            system_api.set_non_fungible(non_fungible_address, Some(non_fungible));
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
        mut args: Vec<ScryptoValue>,
        system_api: &mut S,
    ) -> Result<ScryptoValue, ResourceManagerError> {
        match function {
            "method_auth" => {
                let method: ResourceMethod = scrypto_decode(&args.remove(0).raw)
                    .map_err(|e| ResourceManagerError::InvalidRequestData(e))?;
                let method_entry = self.authorization.get_mut(&method).unwrap();
                let method_entry_method: String = scrypto_decode(&args.remove(0).raw)
                    .map_err(|e| ResourceManagerError::InvalidRequestData(e))?;
                method_entry.main(&method_entry_method, args)
            }
            "create_vault" => {
                let container =
                    ResourceContainer::new_empty(resource_address, self.resource_type());
                let vault_id = system_api
                    .create_vault(container)
                    .map_err(|_| ResourceManagerError::CouldNotCreateVault)?;
                Ok(ScryptoValue::from_value(&scrypto::resource::Vault(
                    vault_id,
                )))
            }
            "create_empty_bucket" => {
                let container =
                    ResourceContainer::new_empty(resource_address, self.resource_type());
                let bucket_id = system_api
                    .create_bucket(container)
                    .map_err(|_| ResourceManagerError::CouldNotCreateBucket)?;
                Ok(ScryptoValue::from_value(&scrypto::resource::Bucket(
                    bucket_id,
                )))
            }
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
                let mut non_fungible = system_api
                    .get_non_fungible(&non_fungible_address)
                    .cloned()
                    .ok_or(ResourceManagerError::NonFungibleNotFound(
                        non_fungible_address.clone(),
                    ))?;
                non_fungible.set_mutable_data(data.raw);
                system_api.set_non_fungible(non_fungible_address, Some(non_fungible));

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
