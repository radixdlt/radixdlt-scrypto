use crate::engine::{HeapRENode, SystemApi};
use crate::fee::FeeReserve;
use crate::model::resource_manager::ResourceMethodRule::{Protected, Public};
use crate::model::ResourceManagerError::InvalidMethod;
use crate::model::{convert, MethodAuthorization, ResourceContainer};
use crate::model::{Bucket, NonFungible, Vault};
use crate::model::{InvokeError, NonFungibleWrapper};
use crate::types::AccessRule::*;
use crate::types::ResourceMethodAuthKey::*;
use crate::types::*;
use crate::wasm::*;

/// Converts soft authorization rule to a hard authorization rule.
/// Currently required as all auth is defined by soft authorization rules.
macro_rules! convert_auth {
    ($auth:expr) => {
        convert(&Type::Unit, &ScryptoValue::unit(), &$auth)
    };
}

/// Represents an error when accessing a bucket.
#[derive(Debug, Clone, PartialEq, Eq, TypeId, Encode, Decode)]
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
    ) -> Result<ScryptoValue, InvokeError<ResourceManagerError>> {
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
    method_table: HashMap<ResourceManagerFnIdentifier, ResourceMethodRule>,
    vault_method_table: HashMap<VaultFnIdentifier, ResourceMethodRule>,
    bucket_method_table: HashMap<BucketFnIdentifier, ResourceMethodRule>,
    authorization: HashMap<ResourceMethodAuthKey, MethodAccessRule>,
    total_supply: Decimal,
}

impl ResourceManager {
    pub fn new(
        resource_type: ResourceType,
        metadata: HashMap<String, String>,
        mut auth: HashMap<ResourceMethodAuthKey, (AccessRule, Mutability)>,
    ) -> Result<Self, InvokeError<ResourceManagerError>> {
        let mut vault_method_table: HashMap<VaultFnIdentifier, ResourceMethodRule> = HashMap::new();
        vault_method_table.insert(VaultFnIdentifier::LockFee, Protected(Withdraw));
        vault_method_table.insert(VaultFnIdentifier::LockContingentFee, Protected(Withdraw));
        vault_method_table.insert(VaultFnIdentifier::Take, Protected(Withdraw));
        vault_method_table.insert(VaultFnIdentifier::Put, Protected(Deposit));
        vault_method_table.insert(VaultFnIdentifier::GetAmount, Public);
        vault_method_table.insert(VaultFnIdentifier::GetResourceAddress, Public);
        vault_method_table.insert(VaultFnIdentifier::GetNonFungibleIds, Public);
        vault_method_table.insert(VaultFnIdentifier::CreateProof, Public);
        vault_method_table.insert(VaultFnIdentifier::CreateProofByAmount, Public);
        vault_method_table.insert(VaultFnIdentifier::CreateProofByIds, Public);
        vault_method_table.insert(VaultFnIdentifier::TakeNonFungibles, Protected(Withdraw));

        let mut bucket_method_table: HashMap<BucketFnIdentifier, ResourceMethodRule> =
            HashMap::new();
        bucket_method_table.insert(BucketFnIdentifier::Burn, Protected(Burn));

        let mut method_table: HashMap<ResourceManagerFnIdentifier, ResourceMethodRule> =
            HashMap::new();
        method_table.insert(ResourceManagerFnIdentifier::Mint, Protected(Mint));
        method_table.insert(
            ResourceManagerFnIdentifier::UpdateMetadata,
            Protected(UpdateMetadata),
        );
        method_table.insert(ResourceManagerFnIdentifier::CreateBucket, Public);
        method_table.insert(ResourceManagerFnIdentifier::GetMetadata, Public);
        method_table.insert(ResourceManagerFnIdentifier::GetResourceType, Public);
        method_table.insert(ResourceManagerFnIdentifier::GetTotalSupply, Public);
        method_table.insert(ResourceManagerFnIdentifier::CreateVault, Public);

        // Non Fungible methods
        method_table.insert(
            ResourceManagerFnIdentifier::UpdateNonFungibleData,
            Protected(UpdateNonFungibleData),
        );
        method_table.insert(ResourceManagerFnIdentifier::NonFungibleExists, Public);
        method_table.insert(ResourceManagerFnIdentifier::GetNonFungible, Public);

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
            bucket_method_table,
            authorization,
            total_supply: 0.into(),
        };

        Ok(resource_manager)
    }

    pub fn get_vault_auth(&self, vault_fn: VaultFnIdentifier) -> &MethodAuthorization {
        match self.vault_method_table.get(&vault_fn) {
            None => &MethodAuthorization::Unsupported,
            Some(Public) => &MethodAuthorization::AllowAll,
            Some(Protected(auth_key)) => self
                .authorization
                .get(auth_key)
                .expect(&format!("Authorization for {:?} not specified", vault_fn))
                .get_method_auth(),
        }
    }

    pub fn get_bucket_auth(&self, bucket_fn: BucketFnIdentifier) -> &MethodAuthorization {
        match self.bucket_method_table.get(&bucket_fn) {
            None => &MethodAuthorization::Unsupported,
            Some(Public) => &MethodAuthorization::AllowAll,
            Some(Protected(method)) => self
                .authorization
                .get(method)
                .expect(&format!("Authorization for {:?} not specified", bucket_fn))
                .get_method_auth(),
        }
    }

    pub fn get_auth(
        &self,
        fn_identifier: ResourceManagerFnIdentifier,
        args: &ScryptoValue,
    ) -> &MethodAuthorization {
        match &fn_identifier {
            ResourceManagerFnIdentifier::UpdateAuth => {
                // FIXME we can't assume the input always match the function identifier
                // especially for the auth module code path
                let input: ResourceManagerUpdateAuthInput = scrypto_decode(&args.raw).unwrap();
                match self.authorization.get(&input.method) {
                    None => &MethodAuthorization::Unsupported,
                    Some(entry) => {
                        entry.get_update_auth(MethodAccessRuleMethod::Update(input.access_rule))
                    }
                }
            }
            ResourceManagerFnIdentifier::LockAuth => {
                // FIXME we can't assume the input always match the function identifier
                // especially for the auth module code path
                let input: ResourceManagerLockAuthInput = scrypto_decode(&args.raw).unwrap();
                match self.authorization.get(&input.method) {
                    None => &MethodAuthorization::Unsupported,
                    Some(entry) => entry.get_update_auth(MethodAccessRuleMethod::Lock()),
                }
            }
            _ => match self.method_table.get(&fn_identifier) {
                None => &MethodAuthorization::Unsupported,
                Some(Public) => &MethodAuthorization::AllowAll,
                Some(Protected(method)) => self
                    .authorization
                    .get(method)
                    .expect(&format!("Authorization for {:?} not specified", method))
                    .get_method_auth(),
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

    pub fn mint<'s, Y, W, I, R>(
        &mut self,
        mint_params: MintParams,
        self_address: ResourceAddress,
        system_api: &mut Y,
    ) -> Result<ResourceContainer, InvokeError<ResourceManagerError>>
    where
        Y: SystemApi<'s, W, I, R>,
        W: WasmEngine<I>,
        I: WasmInstance,
        R: FeeReserve,
    {
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
    ) -> Result<ResourceContainer, InvokeError<ResourceManagerError>> {
        if let ResourceType::Fungible { divisibility } = self.resource_type {
            // check amount
            self.check_amount(amount)?;

            // Practically impossible to overflow the Decimal type with this limit in place.
            if amount > dec!("1000000000000000000") {
                return Err(InvokeError::Error(
                    ResourceManagerError::MaxMintAmountExceeded,
                ));
            }

            self.total_supply += amount;

            Ok(ResourceContainer::new_fungible(
                self_address,
                divisibility,
                amount,
            ))
        } else {
            Err(InvokeError::Error(
                ResourceManagerError::ResourceTypeDoesNotMatch,
            ))
        }
    }

    pub fn mint_non_fungibles<'s, Y, W, I, R>(
        &mut self,
        entries: HashMap<NonFungibleId, (Vec<u8>, Vec<u8>)>,
        self_address: ResourceAddress,
        system_api: &mut Y,
    ) -> Result<ResourceContainer, InvokeError<ResourceManagerError>>
    where
        Y: SystemApi<'s, W, I, R>,
        W: WasmEngine<I>,
        I: WasmInstance,
        R: FeeReserve,
    {
        // check resource type
        if !matches!(self.resource_type, ResourceType::NonFungible) {
            return Err(InvokeError::Error(
                ResourceManagerError::ResourceTypeDoesNotMatch,
            ));
        }

        // check amount
        let amount: Decimal = entries.len().into();
        self.check_amount(amount)?;

        self.total_supply += amount;

        // Allocate non-fungibles
        let mut ids = BTreeSet::new();
        for (id, data) in entries {
            let value = system_api
                .substate_read(SubstateId::NonFungible(self_address, id.clone()))
                .map_err(InvokeError::Downstream)?;
            let wrapper: NonFungibleWrapper =
                scrypto_decode(&value.raw).expect("Failed to decode NonFungibleWrapper substate");
            if wrapper.0.is_some() {
                return Err(InvokeError::Error(
                    ResourceManagerError::NonFungibleAlreadyExists(NonFungibleAddress::new(
                        self_address,
                        id.clone(),
                    )),
                ));
            }

            let non_fungible = NonFungible::new(data.0, data.1);
            system_api
                .substate_write(
                    SubstateId::NonFungible(self_address, id.clone()),
                    ScryptoValue::from_typed(&NonFungibleWrapper(Some(non_fungible))),
                )
                .map_err(InvokeError::Downstream)?;
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
    ) -> Result<(), InvokeError<ResourceManagerError>> {
        self.metadata = new_metadata;

        Ok(())
    }

    fn check_amount(&self, amount: Decimal) -> Result<(), InvokeError<ResourceManagerError>> {
        let divisibility = self.resource_type.divisibility();

        if amount.is_negative()
            || amount.0 % I256::from(10i128.pow((18 - divisibility).into())) != I256::from(0)
        {
            Err(InvokeError::Error(ResourceManagerError::InvalidAmount(
                amount,
                divisibility,
            )))
        } else {
            Ok(())
        }
    }

    pub fn static_main<'s, Y, W, I, R>(
        resource_manager_fn: ResourceManagerFnIdentifier,
        args: ScryptoValue,
        system_api: &mut Y,
    ) -> Result<ScryptoValue, InvokeError<ResourceManagerError>>
    where
        Y: SystemApi<'s, W, I, R>,
        W: WasmEngine<I>,
        I: WasmInstance,
        R: FeeReserve,
    {
        match resource_manager_fn {
            ResourceManagerFnIdentifier::Create => {
                let input: ResourceManagerCreateInput = scrypto_decode(&args.raw)
                    .map_err(|e| InvokeError::Error(ResourceManagerError::InvalidRequestData(e)))?;

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
                            return Err(InvokeError::Error(
                                ResourceManagerError::ResourceTypeDoesNotMatch,
                            ));
                        }
                    }
                    system_api
                        .node_create(HeapRENode::Resource(resource_manager, Some(non_fungibles)))
                        .map_err(InvokeError::Downstream)?
                } else {
                    if let Some(mint_params) = &input.mint_params {
                        if let MintParams::Fungible { amount } = mint_params {
                            resource_manager.check_amount(*amount)?;
                            // TODO: refactor this into mint function
                            if *amount > dec!("1000000000000000000") {
                                return Err(InvokeError::Error(
                                    ResourceManagerError::MaxMintAmountExceeded,
                                ));
                            }
                            resource_manager.total_supply = amount.clone();
                        } else {
                            return Err(InvokeError::Error(
                                ResourceManagerError::ResourceTypeDoesNotMatch,
                            ));
                        }
                    }
                    system_api
                        .node_create(HeapRENode::Resource(resource_manager, None))
                        .map_err(InvokeError::Downstream)?
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
                        .map_err(InvokeError::Downstream)?
                        .into();
                    Some(scrypto::resource::Bucket(bucket_id))
                } else {
                    None
                };

                system_api
                    .node_globalize(resource_node_id)
                    .map_err(InvokeError::Downstream)?;

                Ok(ScryptoValue::from_typed(&(resource_address, bucket_id)))
            }
            _ => Err(InvokeError::Error(InvalidMethod)),
        }
    }

    pub fn main<'s, Y, W, I, R>(
        resource_address: ResourceAddress,
        resource_manager_fn: ResourceManagerFnIdentifier,
        args: ScryptoValue,
        system_api: &mut Y,
    ) -> Result<ScryptoValue, InvokeError<ResourceManagerError>>
    where
        Y: SystemApi<'s, W, I, R>,
        W: WasmEngine<I>,
        I: WasmInstance,
        R: FeeReserve,
    {
        let substate_id = SubstateId::ResourceManager(resource_address);
        let mut ref_mut = system_api
            .substate_borrow_mut(&substate_id)
            .map_err(InvokeError::Downstream)?;
        let resource_manager = ref_mut.resource_manager();

        let rtn = match resource_manager_fn {
            ResourceManagerFnIdentifier::UpdateAuth => {
                let input: ResourceManagerUpdateAuthInput = scrypto_decode(&args.raw)
                    .map_err(|e| InvokeError::Error(ResourceManagerError::InvalidRequestData(e)))?;
                let method_entry = resource_manager
                    .authorization
                    .get_mut(&input.method)
                    .expect(&format!(
                        "Authorization for {:?} not specified",
                        input.method
                    ));
                method_entry.main(MethodAccessRuleMethod::Update(input.access_rule))
            }
            ResourceManagerFnIdentifier::LockAuth => {
                let input: ResourceManagerLockAuthInput = scrypto_decode(&args.raw)
                    .map_err(|e| InvokeError::Error(ResourceManagerError::InvalidRequestData(e)))?;
                let method_entry = resource_manager
                    .authorization
                    .get_mut(&input.method)
                    .expect(&format!(
                        "Authorization for {:?} not specified",
                        input.method
                    ));
                method_entry.main(MethodAccessRuleMethod::Lock())
            }
            ResourceManagerFnIdentifier::CreateVault => {
                let _: ResourceManagerCreateVaultInput = scrypto_decode(&args.raw)
                    .map_err(|e| InvokeError::Error(ResourceManagerError::InvalidRequestData(e)))?;
                let container = ResourceContainer::new_empty(
                    resource_address,
                    resource_manager.resource_type(),
                );
                let vault_id = system_api
                    .node_create(HeapRENode::Vault(Vault::new(container)))
                    .map_err(InvokeError::Downstream)?
                    .into();
                Ok(ScryptoValue::from_typed(&scrypto::resource::Vault(
                    vault_id,
                )))
            }
            ResourceManagerFnIdentifier::CreateBucket => {
                let _: ResourceManagerCreateBucketInput = scrypto_decode(&args.raw)
                    .map_err(|e| InvokeError::Error(ResourceManagerError::InvalidRequestData(e)))?;
                let container = ResourceContainer::new_empty(
                    resource_address,
                    resource_manager.resource_type(),
                );
                let bucket_id = system_api
                    .node_create(HeapRENode::Bucket(Bucket::new(container)))
                    .map_err(InvokeError::Downstream)?
                    .into();
                Ok(ScryptoValue::from_typed(&scrypto::resource::Bucket(
                    bucket_id,
                )))
            }
            ResourceManagerFnIdentifier::Mint => {
                let input: ResourceManagerMintInput = scrypto_decode(&args.raw)
                    .map_err(|e| InvokeError::Error(ResourceManagerError::InvalidRequestData(e)))?;
                let container =
                    resource_manager.mint(input.mint_params, resource_address, system_api)?;
                let bucket_id = system_api
                    .node_create(HeapRENode::Bucket(Bucket::new(container)))
                    .map_err(InvokeError::Downstream)?
                    .into();
                Ok(ScryptoValue::from_typed(&scrypto::resource::Bucket(
                    bucket_id,
                )))
            }
            ResourceManagerFnIdentifier::GetMetadata => {
                let _: ResourceManagerGetMetadataInput = scrypto_decode(&args.raw)
                    .map_err(|e| InvokeError::Error(ResourceManagerError::InvalidRequestData(e)))?;
                Ok(ScryptoValue::from_typed(&resource_manager.metadata))
            }
            ResourceManagerFnIdentifier::GetResourceType => {
                let _: ResourceManagerGetResourceTypeInput = scrypto_decode(&args.raw)
                    .map_err(|e| InvokeError::Error(ResourceManagerError::InvalidRequestData(e)))?;
                Ok(ScryptoValue::from_typed(&resource_manager.resource_type))
            }
            ResourceManagerFnIdentifier::GetTotalSupply => {
                let _: ResourceManagerGetTotalSupplyInput = scrypto_decode(&args.raw)
                    .map_err(|e| InvokeError::Error(ResourceManagerError::InvalidRequestData(e)))?;
                Ok(ScryptoValue::from_typed(&resource_manager.total_supply))
            }
            ResourceManagerFnIdentifier::UpdateMetadata => {
                let input: ResourceManagerUpdateMetadataInput = scrypto_decode(&args.raw)
                    .map_err(|e| InvokeError::Error(ResourceManagerError::InvalidRequestData(e)))?;
                resource_manager.update_metadata(input.metadata)?;
                Ok(ScryptoValue::from_typed(&()))
            }
            ResourceManagerFnIdentifier::UpdateNonFungibleData => {
                let input: ResourceManagerUpdateNonFungibleDataInput = scrypto_decode(&args.raw)
                    .map_err(|e| InvokeError::Error(ResourceManagerError::InvalidRequestData(e)))?;

                // Read current value
                let value = system_api
                    .substate_read(SubstateId::NonFungible(
                        resource_address.clone(),
                        input.id.clone(),
                    ))
                    .map_err(InvokeError::Downstream)?;
                let wrapper: NonFungibleWrapper = scrypto_decode(&value.raw)
                    .expect("Failed to decode NonFungibleWrapper substate");

                // Write new value
                if let Some(mut non_fungible) = wrapper.0 {
                    non_fungible.set_mutable_data(input.data);
                    system_api
                        .substate_write(
                            SubstateId::NonFungible(resource_address.clone(), input.id.clone()),
                            ScryptoValue::from_typed(&NonFungibleWrapper(Some(non_fungible))),
                        )
                        .map_err(InvokeError::Downstream)?;
                } else {
                    let non_fungible_address =
                        NonFungibleAddress::new(resource_address.clone(), input.id);
                    return Err(InvokeError::Error(
                        ResourceManagerError::NonFungibleNotFound(non_fungible_address.clone()),
                    ));
                }

                Ok(ScryptoValue::from_typed(&()))
            }
            ResourceManagerFnIdentifier::NonFungibleExists => {
                let input: ResourceManagerNonFungibleExistsInput = scrypto_decode(&args.raw)
                    .map_err(|e| InvokeError::Error(ResourceManagerError::InvalidRequestData(e)))?;

                let value = system_api
                    .substate_read(SubstateId::NonFungible(resource_address.clone(), input.id))
                    .map_err(InvokeError::Downstream)?;
                let wrapper: NonFungibleWrapper = scrypto_decode(&value.raw)
                    .expect("Failed to decode NonFungibleWrapper substate");
                Ok(ScryptoValue::from_typed(&wrapper.0.is_some()))
            }
            ResourceManagerFnIdentifier::GetNonFungible => {
                let input: ResourceManagerGetNonFungibleInput = scrypto_decode(&args.raw)
                    .map_err(|e| InvokeError::Error(ResourceManagerError::InvalidRequestData(e)))?;
                let non_fungible_address =
                    NonFungibleAddress::new(resource_address.clone(), input.id.clone());
                let value = system_api
                    .substate_read(SubstateId::NonFungible(resource_address.clone(), input.id))
                    .map_err(InvokeError::Downstream)?;
                let wrapper: NonFungibleWrapper = scrypto_decode(&value.raw)
                    .expect("Failed to decode NonFungibleWrapper substate");
                let non_fungible = wrapper.0.ok_or(InvokeError::Error(
                    ResourceManagerError::NonFungibleNotFound(non_fungible_address),
                ))?;
                Ok(ScryptoValue::from_typed(&[
                    non_fungible.immutable_data(),
                    non_fungible.mutable_data(),
                ]))
            }
            _ => Err(InvokeError::Error(InvalidMethod)),
        }?;

        system_api
            .substate_return_mut(ref_mut)
            .map_err(InvokeError::Downstream)?;

        Ok(rtn)
    }
}
