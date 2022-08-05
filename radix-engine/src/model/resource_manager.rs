use sbor::rust::collections::*;
use sbor::rust::string::String;
use sbor::rust::string::ToString;
use sbor::rust::vec::*;
use sbor::*;
use scrypto::buffer::scrypto_decode;
use scrypto::engine::types::*;
use scrypto::prelude::{
    ResourceManagerCreateBucketInput, ResourceManagerCreateInput, ResourceManagerCreateVaultInput,
    ResourceManagerGetNonFungibleInput, ResourceManagerGetResourceTypeInput,
    ResourceManagerGetTotalSupplyInput, ResourceManagerLockAuthInput, ResourceManagerMintInput,
    ResourceManagerNonFungibleExistsInput, ResourceManagerUpdateAuthInput,
    ResourceManagerUpdateMetadataInput, ResourceManagerUpdateNonFungibleDataInput,
};
use scrypto::resource::AccessRule::{self, *};
use scrypto::resource::Mutability::{self, *};
use scrypto::resource::ResourceManagerGetMetadataInput;
use scrypto::resource::ResourceMethodAuthKey::{self, *};
use scrypto::values::ScryptoValue;

use crate::engine::{HeapRENode, RuntimeError, SystemApi};
use crate::fee::FeeReserve;
use crate::fee::FeeReserveError;
use crate::model::resource_manager::ResourceMethodRule::{Protected, Public};
use crate::model::NonFungibleWrapper;
use crate::model::ResourceManagerError::InvalidMethod;
use crate::model::{convert, MethodAuthorization, ResourceContainer};
use crate::model::{Bucket, NonFungible, Vault};
use crate::wasm::*;

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
    InvalidMethod,
    CostingError(FeeReserveError),
}

enum MethodAccessRuleMethod {
    Lock(),
    Update(AccessRule),
}

#[derive(Debug, Clone, TypeId, Encode, Decode, PartialEq, Eq)]
struct MethodAccessRule {
    auth: MethodAuthorization,
    update_auth: MethodAuthorization,
}

impl MethodAccessRule {
    pub fn new(entry: (AccessRule, Mutability)) -> Self {
        MethodAccessRule {
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

    pub fn get_update_auth(&self, method: MethodAccessRuleMethod) -> &MethodAuthorization {
        match method {
            MethodAccessRuleMethod::Lock() | MethodAccessRuleMethod::Update(_) => &self.update_auth,
        }
    }

    pub fn main(
        &mut self,
        method: MethodAccessRuleMethod,
    ) -> Result<ScryptoValue, ResourceManagerError> {
        match method {
            MethodAccessRuleMethod::Lock() => self.lock(),
            MethodAccessRuleMethod::Update(method_auth) => {
                self.update(method_auth);
            }
        }

        Ok(ScryptoValue::from_typed(&()))
    }

    fn update(&mut self, method_auth: AccessRule) {
        self.auth = convert_auth!(method_auth)
    }

    fn lock(&mut self) {
        self.update_auth = MethodAuthorization::DenyAll;
    }
}

#[derive(Debug, Clone, TypeId, Encode, Decode, PartialEq, Eq)]
enum ResourceMethodRule {
    Public,
    Protected(ResourceMethodAuthKey),
}

/// The definition of a resource.
#[derive(Debug, Clone, TypeId, Encode, Decode, PartialEq, Eq)]
pub struct ResourceManager {
    resource_type: ResourceType,
    metadata: HashMap<String, String>,
    method_table: HashMap<String, ResourceMethodRule>,
    vault_method_table: HashMap<String, ResourceMethodRule>,
    authorization: HashMap<ResourceMethodAuthKey, MethodAccessRule>,
    total_supply: Decimal,
}

impl ResourceManager {
    pub fn new(
        resource_type: ResourceType,
        metadata: HashMap<String, String>,
        mut auth: HashMap<ResourceMethodAuthKey, (AccessRule, Mutability)>,
    ) -> Result<Self, ResourceManagerError> {
        let mut vault_method_table: HashMap<String, ResourceMethodRule> = HashMap::new();
        vault_method_table.insert("lock_fee".to_string(), Protected(Withdraw));
        vault_method_table.insert("take".to_string(), Protected(Withdraw));
        vault_method_table.insert("put".to_string(), Protected(Deposit));
        for pub_method in [
            "amount",
            "resource_address",
            "non_fungible_ids",
            "create_proof",
            "create_proof_by_amount",
            "create_proof_by_ids",
        ] {
            vault_method_table.insert(pub_method.to_string(), Public);
        }
        // Non Fungible methods
        vault_method_table.insert("take_non_fungibles".to_string(), Protected(Withdraw));

        let mut method_table: HashMap<String, ResourceMethodRule> = HashMap::new();
        method_table.insert("mint".to_string(), Protected(Mint));
        method_table.insert("burn".to_string(), Protected(Burn));
        method_table.insert("update_metadata".to_string(), Protected(UpdateMetadata));
        for pub_method in [
            "create_bucket",
            "metadata",
            "resource_type",
            "total_supply",
            "create_vault",
        ] {
            method_table.insert(pub_method.to_string(), Public);
        }

        // Non Fungible methods
        method_table.insert(
            "update_non_fungible_data".to_string(),
            Protected(UpdateNonFungibleData),
        );
        for pub_method in ["non_fungible_exists", "non_fungible_data"] {
            method_table.insert(pub_method.to_string(), Public);
        }

        let mut authorization: HashMap<ResourceMethodAuthKey, MethodAccessRule> = HashMap::new();
        for (auth_entry_key, default) in [
            (Mint, (DenyAll, LOCKED)),
            (Burn, (DenyAll, LOCKED)),
            (Withdraw, (AllowAll, LOCKED)),
            (Deposit, (AllowAll, LOCKED)),
            (UpdateMetadata, (DenyAll, LOCKED)),
            (UpdateNonFungibleData, (DenyAll, LOCKED)),
        ] {
            let entry = auth.remove(&auth_entry_key).unwrap_or(default);
            authorization.insert(auth_entry_key, MethodAccessRule::new(entry));
        }

        let resource_manager = Self {
            resource_type,
            metadata,
            method_table,
            vault_method_table,
            authorization,
            total_supply: 0.into(),
        };

        Ok(resource_manager)
    }

    pub fn get_vault_auth(&self, method_name: &str) -> &MethodAuthorization {
        match self.vault_method_table.get(method_name) {
            None => &MethodAuthorization::Unsupported,
            Some(Public) => &MethodAuthorization::AllowAll,
            Some(Protected(auth_key)) => {
                self.authorization.get(auth_key).unwrap().get_method_auth()
            }
        }
    }

    pub fn get_consuming_bucket_auth(&self, method_name: &str) -> &MethodAuthorization {
        match self.method_table.get(method_name) {
            None => &MethodAuthorization::Unsupported,
            Some(Public) => &MethodAuthorization::AllowAll,
            Some(Protected(method)) => self.authorization.get(method).unwrap().get_method_auth(),
        }
    }

    pub fn get_auth(&self, method_name: &str, arg: &ScryptoValue) -> &MethodAuthorization {
        match method_name {
            "update_auth" => {
                let input: ResourceManagerUpdateAuthInput = scrypto_decode(&arg.raw).unwrap();
                match self.authorization.get(&input.method) {
                    None => &MethodAuthorization::Unsupported,
                    Some(entry) => {
                        entry.get_update_auth(MethodAccessRuleMethod::Update(input.access_rule))
                    }
                }
            }
            "lock_auth" => {
                let input: ResourceManagerLockAuthInput = scrypto_decode(&arg.raw).unwrap();
                match self.authorization.get(&input.method) {
                    None => &MethodAuthorization::Unsupported,
                    Some(entry) => entry.get_update_auth(MethodAccessRuleMethod::Lock()),
                }
            }
            _ => match self.method_table.get(method_name) {
                None => &MethodAuthorization::Unsupported,
                Some(Public) => &MethodAuthorization::AllowAll,
                Some(Protected(method)) => {
                    self.authorization.get(method).unwrap().get_method_auth()
                }
            },
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

    pub fn mint<
        'p,
        's,
        Y: SystemApi<'p, 's, W, I, C>,
        W: WasmEngine<I>,
        I: WasmInstance,
        C: FeeReserve,
    >(
        &mut self,
        mint_params: MintParams,
        self_address: ResourceAddress,
        system_api: &mut Y,
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

    pub fn mint_non_fungibles<
        'p,
        's,
        Y: SystemApi<'p, 's, W, I, C>,
        W: WasmEngine<I>,
        I: WasmInstance,
        C: FeeReserve,
    >(
        &mut self,
        entries: HashMap<NonFungibleId, (Vec<u8>, Vec<u8>)>,
        self_address: ResourceAddress,
        system_api: &mut Y,
    ) -> Result<ResourceContainer, ResourceManagerError> {
        // check resource type
        if !matches!(self.resource_type, ResourceType::NonFungible) {
            return Err(ResourceManagerError::ResourceTypeDoesNotMatch);
        }

        // check amount
        let amount: Decimal = entries.len().into();
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
            let value = system_api
                .substate_read(SubstateId::NonFungible(self_address, id.clone()))
                .expect("Should never fail");
            let wrapper: NonFungibleWrapper = scrypto_decode(&value.raw).unwrap();
            if wrapper.0.is_some() {
                return Err(ResourceManagerError::NonFungibleAlreadyExists(
                    NonFungibleAddress::new(self_address, id.clone()),
                ));
            }

            let non_fungible = NonFungible::new(data.0, data.1);
            system_api
                .substate_write(
                    SubstateId::NonFungible(self_address, id.clone()),
                    ScryptoValue::from_typed(&NonFungibleWrapper(Some(non_fungible))),
                )
                .expect("Should never fail");
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

        if amount.is_negative()
            || amount.0 % 10i128.pow((18 - divisibility).into()) != I256::from(0)
        {
            Err(ResourceManagerError::InvalidAmount(amount, divisibility))
        } else {
            Ok(())
        }
    }

    pub fn static_main<
        'p,
        's,
        Y: SystemApi<'p, 's, W, I, C>,
        W: WasmEngine<I>,
        I: WasmInstance,
        C: FeeReserve,
    >(
        method_name: &str,
        arg: ScryptoValue,
        system_api: &mut Y,
    ) -> Result<ScryptoValue, ResourceManagerError> {
        match method_name {
            "create" => {
                let input: ResourceManagerCreateInput = scrypto_decode(&arg.raw)
                    .map_err(|e| ResourceManagerError::InvalidRequestData(e))?;

                let mut resource_manager =
                    ResourceManager::new(input.resource_type, input.metadata, input.access_rules)?;

                let resource_node_id = if matches!(input.resource_type, ResourceType::NonFungible) {
                    let mut non_fungibles: HashMap<NonFungibleId, NonFungible> = HashMap::new();
                    if let Some(mint_params) = &input.mint_params {
                        if let MintParams::NonFungible { entries } = mint_params {
                            for (non_fungible_id, data) in entries {
                                let non_fungible = NonFungible::new(data.0.clone(), data.1.clone());
                                non_fungibles.insert(non_fungible_id.clone(), non_fungible);
                            }
                            resource_manager.total_supply = entries.len().into();
                        } else {
                            return Err(ResourceManagerError::ResourceTypeDoesNotMatch);
                        }
                    }
                    system_api
                        .node_create(HeapRENode::Resource(resource_manager, Some(non_fungibles)))
                        .expect("Should never fail")
                } else {
                    if let Some(mint_params) = &input.mint_params {
                        if let MintParams::Fungible { amount } = mint_params {
                            resource_manager.check_amount(*amount)?;
                            // It takes `1,701,411,835` mint operations to reach `Decimal::MAX`,
                            // which will be impossible with metering.
                            if *amount > 100_000_000_000i128.into() {
                                return Err(ResourceManagerError::MaxMintAmountExceeded);
                            }
                            resource_manager.total_supply = amount.clone();
                        } else {
                            return Err(ResourceManagerError::ResourceTypeDoesNotMatch);
                        }
                    }
                    system_api
                        .node_create(HeapRENode::Resource(resource_manager, None))
                        .expect("Should never fail")
                };
                let resource_address = resource_node_id.clone().into();

                let bucket_id = if let Some(mint_params) = input.mint_params {
                    let container = match mint_params {
                        MintParams::NonFungible { entries } => {
                            let ids = entries.into_keys().collect();
                            ResourceContainer::new_non_fungible(resource_address, ids)
                        }
                        MintParams::Fungible { amount } => ResourceContainer::new_fungible(
                            resource_address,
                            input.resource_type.divisibility(),
                            amount,
                        ),
                    };
                    let bucket_id = system_api
                        .node_create(HeapRENode::Bucket(Bucket::new(container)))
                        .unwrap()
                        .into();
                    Some(scrypto::resource::Bucket(bucket_id))
                } else {
                    None
                };

                system_api
                    .node_globalize(resource_node_id)
                    .map_err(|e| match e {
                        RuntimeError::CostingError(cost_unit_error) => {
                            ResourceManagerError::CostingError(cost_unit_error)
                        }
                        _ => panic!("Unexpected error {}", e),
                    })?;

                Ok(ScryptoValue::from_typed(&(resource_address, bucket_id)))
            }
            _ => Err(InvalidMethod),
        }
    }

    pub fn main<
        'p,
        's,
        Y: SystemApi<'p, 's, W, I, C>,
        W: WasmEngine<I>,
        I: WasmInstance,
        C: FeeReserve,
    >(
        resource_address: ResourceAddress,
        method_name: &str,
        arg: ScryptoValue,
        system_api: &mut Y,
    ) -> Result<ScryptoValue, ResourceManagerError> {
        let substate_id = SubstateId::ResourceManager(resource_address);
        let mut ref_mut = system_api
            .substate_borrow_mut(&substate_id)
            .map_err(ResourceManagerError::CostingError)?;
        let resource_manager = ref_mut.resource_manager();

        let rtn = match method_name {
            "update_auth" => {
                let input: ResourceManagerUpdateAuthInput = scrypto_decode(&arg.raw)
                    .map_err(|e| ResourceManagerError::InvalidRequestData(e))?;
                let method_entry = resource_manager
                    .authorization
                    .get_mut(&input.method)
                    .unwrap();
                method_entry.main(MethodAccessRuleMethod::Update(input.access_rule))
            }
            "lock_auth" => {
                let input: ResourceManagerLockAuthInput = scrypto_decode(&arg.raw)
                    .map_err(|e| ResourceManagerError::InvalidRequestData(e))?;
                let method_entry = resource_manager
                    .authorization
                    .get_mut(&input.method)
                    .unwrap();
                method_entry.main(MethodAccessRuleMethod::Lock())
            }
            "create_vault" => {
                let _: ResourceManagerCreateVaultInput = scrypto_decode(&arg.raw)
                    .map_err(|e| ResourceManagerError::InvalidRequestData(e))?;
                let container = ResourceContainer::new_empty(
                    resource_address,
                    resource_manager.resource_type(),
                );
                let vault_id = system_api
                    .node_create(HeapRENode::Vault(Vault::new(container)))
                    .unwrap()
                    .into();
                Ok(ScryptoValue::from_typed(&scrypto::resource::Vault(
                    vault_id,
                )))
            }
            "create_bucket" => {
                let _: ResourceManagerCreateBucketInput = scrypto_decode(&arg.raw)
                    .map_err(|e| ResourceManagerError::InvalidRequestData(e))?;
                let container = ResourceContainer::new_empty(
                    resource_address,
                    resource_manager.resource_type(),
                );
                let bucket_id = system_api
                    .node_create(HeapRENode::Bucket(Bucket::new(container)))
                    .unwrap()
                    .into();
                Ok(ScryptoValue::from_typed(&scrypto::resource::Bucket(
                    bucket_id,
                )))
            }
            "mint" => {
                let input: ResourceManagerMintInput = scrypto_decode(&arg.raw)
                    .map_err(|e| ResourceManagerError::InvalidRequestData(e))?;
                let container =
                    resource_manager.mint(input.mint_params, resource_address, system_api)?;
                let bucket_id = system_api
                    .node_create(HeapRENode::Bucket(Bucket::new(container)))
                    .unwrap()
                    .into();
                Ok(ScryptoValue::from_typed(&scrypto::resource::Bucket(
                    bucket_id,
                )))
            }
            "metadata" => {
                let _: ResourceManagerGetMetadataInput = scrypto_decode(&arg.raw)
                    .map_err(|e| ResourceManagerError::InvalidRequestData(e))?;
                Ok(ScryptoValue::from_typed(&resource_manager.metadata))
            }
            "resource_type" => {
                let _: ResourceManagerGetResourceTypeInput = scrypto_decode(&arg.raw)
                    .map_err(|e| ResourceManagerError::InvalidRequestData(e))?;
                Ok(ScryptoValue::from_typed(&resource_manager.resource_type))
            }
            "total_supply" => {
                let _: ResourceManagerGetTotalSupplyInput = scrypto_decode(&arg.raw)
                    .map_err(|e| ResourceManagerError::InvalidRequestData(e))?;
                Ok(ScryptoValue::from_typed(&resource_manager.total_supply))
            }
            "update_metadata" => {
                let input: ResourceManagerUpdateMetadataInput = scrypto_decode(&arg.raw)
                    .map_err(|e| ResourceManagerError::InvalidRequestData(e))?;
                resource_manager.update_metadata(input.metadata)?;
                Ok(ScryptoValue::from_typed(&()))
            }
            "update_non_fungible_data" => {
                let input: ResourceManagerUpdateNonFungibleDataInput = scrypto_decode(&arg.raw)
                    .map_err(|e| ResourceManagerError::InvalidRequestData(e))?;

                // Read current value
                let value = system_api
                    .substate_read(SubstateId::NonFungible(
                        resource_address.clone(),
                        input.id.clone(),
                    ))
                    .expect("Should never fail");
                let wrapper: NonFungibleWrapper = scrypto_decode(&value.raw).unwrap();

                // Write new value
                if let Some(mut non_fungible) = wrapper.0 {
                    non_fungible.set_mutable_data(input.data);
                    system_api
                        .substate_write(
                            SubstateId::NonFungible(resource_address.clone(), input.id.clone()),
                            ScryptoValue::from_typed(&NonFungibleWrapper(Some(non_fungible))),
                        )
                        .expect("Should never fail");
                } else {
                    let non_fungible_address =
                        NonFungibleAddress::new(resource_address.clone(), input.id);
                    return Err(ResourceManagerError::NonFungibleNotFound(
                        non_fungible_address.clone(),
                    ));
                }

                Ok(ScryptoValue::from_typed(&()))
            }
            "non_fungible_exists" => {
                let input: ResourceManagerNonFungibleExistsInput = scrypto_decode(&arg.raw)
                    .map_err(|e| ResourceManagerError::InvalidRequestData(e))?;

                let value = system_api
                    .substate_read(SubstateId::NonFungible(resource_address.clone(), input.id))
                    .expect("Should never fail");
                let wrapper: NonFungibleWrapper = scrypto_decode(&value.raw).unwrap();
                Ok(ScryptoValue::from_typed(&wrapper.0.is_some()))
            }
            "non_fungible_data" => {
                let input: ResourceManagerGetNonFungibleInput = scrypto_decode(&arg.raw)
                    .map_err(|e| ResourceManagerError::InvalidRequestData(e))?;
                let non_fungible_address =
                    NonFungibleAddress::new(resource_address.clone(), input.id.clone());
                let value = system_api
                    .substate_read(SubstateId::NonFungible(resource_address.clone(), input.id))
                    .expect("Should never fail");
                let wrapper: NonFungibleWrapper = scrypto_decode(&value.raw).unwrap();
                let non_fungible = wrapper.0.ok_or(ResourceManagerError::NonFungibleNotFound(
                    non_fungible_address,
                ))?;
                Ok(ScryptoValue::from_typed(&[
                    non_fungible.immutable_data(),
                    non_fungible.mutable_data(),
                ]))
            }
            _ => Err(InvalidMethod),
        }?;

        system_api
            .substate_return_mut(ref_mut)
            .map_err(ResourceManagerError::CostingError)?;

        Ok(rtn)
    }
}
