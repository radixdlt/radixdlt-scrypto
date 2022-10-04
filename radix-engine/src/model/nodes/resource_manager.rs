use crate::engine::{HeapRENode, SystemApi};
use crate::fee::FeeReserve;
use crate::model::{
    Bucket, InvokeError, MethodAuthorization, NonFungible, NonFungibleSubstate, Resource,
    ResourceMethodRule::{Protected, Public},
    Vault,
};
use crate::model::{
    MethodAccessRule, MethodAccessRuleMethod, ResourceManagerSubstate, ResourceMethodRule,
};
use crate::types::AccessRule::*;
use crate::types::ResourceMethodAuthKey::*;
use crate::types::*;
use crate::wasm::*;
use scrypto::core::ResourceManagerFunctionFnIdent;

/// Represents an error when accessing a bucket.
#[derive(Debug, TypeId, Encode, Decode)]
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

/// The definition of a resource.
#[derive(Debug, Clone, TypeId, Encode, Decode, PartialEq, Eq)]
pub struct ResourceManager {
    pub info: ResourceManagerSubstate,
}

impl ResourceManager {
    pub fn new(
        resource_type: ResourceType,
        metadata: HashMap<String, String>,
        mut auth: HashMap<ResourceMethodAuthKey, (AccessRule, Mutability)>,
    ) -> Result<Self, InvokeError<ResourceManagerError>> {
        let mut vault_method_table: HashMap<VaultMethodFnIdent, ResourceMethodRule> =
            HashMap::new();
        vault_method_table.insert(VaultMethodFnIdent::LockFee, Protected(Withdraw));
        vault_method_table.insert(VaultMethodFnIdent::LockContingentFee, Protected(Withdraw));
        vault_method_table.insert(VaultMethodFnIdent::Take, Protected(Withdraw));
        vault_method_table.insert(VaultMethodFnIdent::Put, Protected(Deposit));
        vault_method_table.insert(VaultMethodFnIdent::GetAmount, Public);
        vault_method_table.insert(VaultMethodFnIdent::GetResourceAddress, Public);
        vault_method_table.insert(VaultMethodFnIdent::GetNonFungibleIds, Public);
        vault_method_table.insert(VaultMethodFnIdent::CreateProof, Public);
        vault_method_table.insert(VaultMethodFnIdent::CreateProofByAmount, Public);
        vault_method_table.insert(VaultMethodFnIdent::CreateProofByIds, Public);
        vault_method_table.insert(VaultMethodFnIdent::TakeNonFungibles, Protected(Withdraw));

        let mut bucket_method_table: HashMap<BucketMethodFnIdent, ResourceMethodRule> =
            HashMap::new();
        bucket_method_table.insert(BucketMethodFnIdent::Burn, Protected(Burn));

        let mut method_table: HashMap<ResourceManagerMethodFnIdent, ResourceMethodRule> =
            HashMap::new();
        method_table.insert(ResourceManagerMethodFnIdent::Mint, Protected(Mint));
        method_table.insert(
            ResourceManagerMethodFnIdent::UpdateMetadata,
            Protected(UpdateMetadata),
        );
        method_table.insert(ResourceManagerMethodFnIdent::CreateBucket, Public);
        method_table.insert(ResourceManagerMethodFnIdent::GetMetadata, Public);
        method_table.insert(ResourceManagerMethodFnIdent::GetResourceType, Public);
        method_table.insert(ResourceManagerMethodFnIdent::GetTotalSupply, Public);
        method_table.insert(ResourceManagerMethodFnIdent::CreateVault, Public);

        // Non Fungible methods
        method_table.insert(
            ResourceManagerMethodFnIdent::UpdateNonFungibleData,
            Protected(UpdateNonFungibleData),
        );
        method_table.insert(ResourceManagerMethodFnIdent::NonFungibleExists, Public);
        method_table.insert(ResourceManagerMethodFnIdent::GetNonFungible, Public);

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
            info: ResourceManagerSubstate {
                resource_type,
                metadata,
                method_table,
                vault_method_table,
                bucket_method_table,
                authorization,
                total_supply: 0.into(),
            },
        };

        Ok(resource_manager)
    }

    pub fn get_vault_auth(&self, vault_fn: VaultMethodFnIdent) -> &MethodAuthorization {
        match self.info.vault_method_table.get(&vault_fn) {
            None => &MethodAuthorization::Unsupported,
            Some(Public) => &MethodAuthorization::AllowAll,
            Some(Protected(auth_key)) => self
                .info
                .authorization
                .get(auth_key)
                .expect(&format!("Authorization for {:?} not specified", vault_fn))
                .get_method_auth(),
        }
    }

    pub fn get_bucket_auth(&self, bucket_fn: BucketMethodFnIdent) -> &MethodAuthorization {
        match self.info.bucket_method_table.get(&bucket_fn) {
            None => &MethodAuthorization::Unsupported,
            Some(Public) => &MethodAuthorization::AllowAll,
            Some(Protected(method)) => self
                .info
                .authorization
                .get(method)
                .expect(&format!("Authorization for {:?} not specified", bucket_fn))
                .get_method_auth(),
        }
    }

    pub fn get_auth(
        &self,
        fn_identifier: ResourceManagerMethodFnIdent,
        args: &ScryptoValue,
    ) -> &MethodAuthorization {
        match &fn_identifier {
            ResourceManagerMethodFnIdent::UpdateAuth => {
                // FIXME we can't assume the input always match the function identifier
                // especially for the auth module code path
                let input: ResourceManagerUpdateAuthInput = scrypto_decode(&args.raw).unwrap();
                match self.info.authorization.get(&input.method) {
                    None => &MethodAuthorization::Unsupported,
                    Some(entry) => {
                        entry.get_update_auth(MethodAccessRuleMethod::Update(input.access_rule))
                    }
                }
            }
            ResourceManagerMethodFnIdent::LockAuth => {
                // FIXME we can't assume the input always match the function identifier
                // especially for the auth module code path
                let input: ResourceManagerLockAuthInput = scrypto_decode(&args.raw).unwrap();
                match self.info.authorization.get(&input.method) {
                    None => &MethodAuthorization::Unsupported,
                    Some(entry) => entry.get_update_auth(MethodAccessRuleMethod::Lock()),
                }
            }
            _ => match self.info.method_table.get(&fn_identifier) {
                None => &MethodAuthorization::Unsupported,
                Some(Public) => &MethodAuthorization::AllowAll,
                Some(Protected(method)) => self
                    .info
                    .authorization
                    .get(method)
                    .expect(&format!("Authorization for {:?} not specified", method))
                    .get_method_auth(),
            },
        }
    }

    pub fn resource_type(&self) -> ResourceType {
        self.info.resource_type
    }

    pub fn metadata(&self) -> &HashMap<String, String> {
        &self.info.metadata
    }

    pub fn total_supply(&self) -> Decimal {
        self.info.total_supply
    }

    pub fn mint(
        &mut self,
        mint_params: MintParams,
        self_address: ResourceAddress,
    ) -> Result<(Resource, HashMap<NonFungibleId, NonFungible>), InvokeError<ResourceManagerError>>
    {
        match mint_params {
            MintParams::Fungible { amount } => self.mint_fungible(amount, self_address),
            MintParams::NonFungible { entries } => self.mint_non_fungibles(entries, self_address),
        }
    }

    pub fn mint_fungible(
        &mut self,
        amount: Decimal,
        self_address: ResourceAddress,
    ) -> Result<(Resource, HashMap<NonFungibleId, NonFungible>), InvokeError<ResourceManagerError>>
    {
        if let ResourceType::Fungible { divisibility } = self.info.resource_type {
            // check amount
            self.check_amount(amount)?;

            // Practically impossible to overflow the Decimal type with this limit in place.
            if amount > dec!("1000000000000000000") {
                return Err(InvokeError::Error(
                    ResourceManagerError::MaxMintAmountExceeded,
                ));
            }

            self.info.total_supply += amount;

            Ok((
                Resource::new_fungible(self_address, divisibility, amount),
                HashMap::new(),
            ))
        } else {
            Err(InvokeError::Error(
                ResourceManagerError::ResourceTypeDoesNotMatch,
            ))
        }
    }

    pub fn mint_non_fungibles(
        &mut self,
        entries: HashMap<NonFungibleId, (Vec<u8>, Vec<u8>)>,
        self_address: ResourceAddress,
    ) -> Result<(Resource, HashMap<NonFungibleId, NonFungible>), InvokeError<ResourceManagerError>>
    {
        // check resource type
        if !matches!(self.info.resource_type, ResourceType::NonFungible) {
            return Err(InvokeError::Error(
                ResourceManagerError::ResourceTypeDoesNotMatch,
            ));
        }

        // check amount
        let amount: Decimal = entries.len().into();
        self.check_amount(amount)?;

        self.info.total_supply += amount;

        // Allocate non-fungibles
        let mut ids = BTreeSet::new();
        let mut non_fungibles = HashMap::new();
        for (id, data) in entries {
            let non_fungible = NonFungible::new(data.0, data.1);
            ids.insert(id.clone());
            non_fungibles.insert(id, non_fungible);
        }

        Ok((Resource::new_non_fungible(self_address, ids), non_fungibles))
    }

    pub fn burn(&mut self, amount: Decimal) {
        self.info.total_supply -= amount;
    }

    fn update_metadata(
        &mut self,
        new_metadata: HashMap<String, String>,
    ) -> Result<(), InvokeError<ResourceManagerError>> {
        self.info.metadata = new_metadata;

        Ok(())
    }

    fn check_amount(&self, amount: Decimal) -> Result<(), InvokeError<ResourceManagerError>> {
        let divisibility = self.info.resource_type.divisibility();

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
        resource_manager_fn: ResourceManagerFunctionFnIdent,
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
            ResourceManagerFunctionFnIdent::Create => {
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
                            resource_manager.info.total_supply = entries.len().into();
                        } else {
                            return Err(InvokeError::Error(
                                ResourceManagerError::ResourceTypeDoesNotMatch,
                            ));
                        }
                    }
                    system_api
                        .node_create(HeapRENode::ResourceManager(
                            resource_manager,
                            Some(non_fungibles),
                        ))
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
                            resource_manager.info.total_supply = amount.clone();
                        } else {
                            return Err(InvokeError::Error(
                                ResourceManagerError::ResourceTypeDoesNotMatch,
                            ));
                        }
                    }
                    system_api
                        .node_create(HeapRENode::ResourceManager(resource_manager, None))
                        .map_err(InvokeError::Downstream)?
                };
                let resource_address = resource_node_id.clone().into();

                let bucket_id = if let Some(mint_params) = input.mint_params {
                    let container = match mint_params {
                        MintParams::NonFungible { entries } => {
                            let ids = entries.into_keys().collect();
                            Resource::new_non_fungible(resource_address, ids)
                        }
                        MintParams::Fungible { amount } => Resource::new_fungible(
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

                let global_address = system_api
                    .node_globalize(resource_node_id)
                    .map_err(InvokeError::Downstream)?;
                let resource_address: ResourceAddress = global_address.into();

                Ok(ScryptoValue::from_typed(&(resource_address, bucket_id)))
            }
        }
    }

    pub fn main<'s, Y, W, I, R>(
        resource_address: ResourceAddress,
        resource_manager_fn: ResourceManagerMethodFnIdent,
        args: ScryptoValue,
        system_api: &mut Y,
    ) -> Result<ScryptoValue, InvokeError<ResourceManagerError>>
    where
        Y: SystemApi<'s, W, I, R>,
        W: WasmEngine<I>,
        I: WasmInstance,
        R: FeeReserve,
    {
        let rtn = match resource_manager_fn {
            ResourceManagerMethodFnIdent::UpdateAuth => {
                let input: ResourceManagerUpdateAuthInput = scrypto_decode(&args.raw)
                    .map_err(|e| InvokeError::Error(ResourceManagerError::InvalidRequestData(e)))?;
                let mut node_ref = system_api
                    .borrow_node_mut(&RENodeId::ResourceManager(resource_address))
                    .map_err(InvokeError::Downstream)?;
                let resource_manager = node_ref.resource_manager_mut();

                let method_entry = resource_manager
                    .info
                    .authorization
                    .get_mut(&input.method)
                    .expect(&format!(
                        "Authorization for {:?} not specified",
                        input.method
                    ));
                method_entry.main(MethodAccessRuleMethod::Update(input.access_rule))
            }
            ResourceManagerMethodFnIdent::LockAuth => {
                let input: ResourceManagerLockAuthInput = scrypto_decode(&args.raw)
                    .map_err(|e| InvokeError::Error(ResourceManagerError::InvalidRequestData(e)))?;
                let mut node_ref = system_api
                    .borrow_node_mut(&RENodeId::ResourceManager(resource_address))
                    .map_err(InvokeError::Downstream)?;
                let resource_manager = node_ref.resource_manager_mut();

                let method_entry = resource_manager
                    .info
                    .authorization
                    .get_mut(&input.method)
                    .expect(&format!(
                        "Authorization for {:?} not specified",
                        input.method
                    ));
                method_entry.main(MethodAccessRuleMethod::Lock())
            }
            ResourceManagerMethodFnIdent::CreateVault => {
                let _: ResourceManagerCreateVaultInput = scrypto_decode(&args.raw)
                    .map_err(|e| InvokeError::Error(ResourceManagerError::InvalidRequestData(e)))?;
                let mut node_ref = system_api
                    .borrow_node_mut(&RENodeId::ResourceManager(resource_address))
                    .map_err(InvokeError::Downstream)?;
                let resource_manager = node_ref.resource_manager_mut();

                let resource =
                    Resource::new_empty(resource_address, resource_manager.resource_type());
                let vault_id = system_api
                    .node_create(HeapRENode::Vault(Vault::new(resource)))
                    .map_err(InvokeError::Downstream)?
                    .into();
                Ok(ScryptoValue::from_typed(&scrypto::resource::Vault(
                    vault_id,
                )))
            }
            ResourceManagerMethodFnIdent::CreateBucket => {
                let _: ResourceManagerCreateBucketInput = scrypto_decode(&args.raw)
                    .map_err(|e| InvokeError::Error(ResourceManagerError::InvalidRequestData(e)))?;
                let mut node_ref = system_api
                    .borrow_node_mut(&RENodeId::ResourceManager(resource_address))
                    .map_err(InvokeError::Downstream)?;
                let resource_manager = node_ref.resource_manager_mut();

                let container =
                    Resource::new_empty(resource_address, resource_manager.resource_type());
                let bucket_id = system_api
                    .node_create(HeapRENode::Bucket(Bucket::new(container)))
                    .map_err(InvokeError::Downstream)?
                    .into();
                Ok(ScryptoValue::from_typed(&scrypto::resource::Bucket(
                    bucket_id,
                )))
            }
            ResourceManagerMethodFnIdent::Mint => {
                let input: ResourceManagerMintInput = scrypto_decode(&args.raw)
                    .map_err(|e| InvokeError::Error(ResourceManagerError::InvalidRequestData(e)))?;
                let mut node_ref = system_api
                    .borrow_node_mut(&RENodeId::ResourceManager(resource_address))
                    .map_err(InvokeError::Downstream)?;
                let resource_manager = node_ref.resource_manager_mut();

                let (resource, non_fungibles) =
                    resource_manager.mint(input.mint_params, resource_address)?;

                let bucket_id = system_api
                    .node_create(HeapRENode::Bucket(Bucket::new(resource)))
                    .map_err(InvokeError::Downstream)?
                    .into();

                for (id, non_fungible) in non_fungibles {
                    let value = system_api
                        .substate_read(SubstateId(
                            RENodeId::ResourceManager(resource_address.clone()),
                            SubstateOffset::Resource(ResourceManagerOffset::NonFungible(
                                id.clone(),
                            )),
                        ))
                        .map_err(InvokeError::Downstream)?;
                    let wrapper: NonFungibleSubstate =
                        scrypto_decode(&value.raw).expect("Failed to decode NonFungibleSubstate");
                    if wrapper.0.is_some() {
                        return Err(InvokeError::Error(
                            ResourceManagerError::NonFungibleAlreadyExists(
                                NonFungibleAddress::new(resource_address, id.clone()),
                            ),
                        ));
                    }
                    system_api
                        .substate_write(
                            SubstateId(
                                RENodeId::ResourceManager(resource_address.clone()),
                                SubstateOffset::Resource(ResourceManagerOffset::NonFungible(
                                    id.clone(),
                                )),
                            ),
                            ScryptoValue::from_typed(&NonFungibleSubstate(Some(non_fungible))),
                        )
                        .map_err(InvokeError::Downstream)?;
                }

                Ok(ScryptoValue::from_typed(&scrypto::resource::Bucket(
                    bucket_id,
                )))
            }
            ResourceManagerMethodFnIdent::GetMetadata => {
                let _: ResourceManagerGetMetadataInput = scrypto_decode(&args.raw)
                    .map_err(|e| InvokeError::Error(ResourceManagerError::InvalidRequestData(e)))?;
                let mut node_ref = system_api
                    .borrow_node_mut(&RENodeId::ResourceManager(resource_address))
                    .map_err(InvokeError::Downstream)?;
                let resource_manager = node_ref.resource_manager_mut();

                Ok(ScryptoValue::from_typed(&resource_manager.info.metadata))
            }
            ResourceManagerMethodFnIdent::GetResourceType => {
                let _: ResourceManagerGetResourceTypeInput = scrypto_decode(&args.raw)
                    .map_err(|e| InvokeError::Error(ResourceManagerError::InvalidRequestData(e)))?;
                let mut node_ref = system_api
                    .borrow_node_mut(&RENodeId::ResourceManager(resource_address))
                    .map_err(InvokeError::Downstream)?;
                let resource_manager = node_ref.resource_manager_mut();

                Ok(ScryptoValue::from_typed(
                    &resource_manager.info.resource_type,
                ))
            }
            ResourceManagerMethodFnIdent::GetTotalSupply => {
                let _: ResourceManagerGetTotalSupplyInput = scrypto_decode(&args.raw)
                    .map_err(|e| InvokeError::Error(ResourceManagerError::InvalidRequestData(e)))?;
                let mut node_ref = system_api
                    .borrow_node_mut(&RENodeId::ResourceManager(resource_address))
                    .map_err(InvokeError::Downstream)?;
                let resource_manager = node_ref.resource_manager_mut();

                Ok(ScryptoValue::from_typed(
                    &resource_manager.info.total_supply,
                ))
            }
            ResourceManagerMethodFnIdent::UpdateMetadata => {
                let input: ResourceManagerUpdateMetadataInput = scrypto_decode(&args.raw)
                    .map_err(|e| InvokeError::Error(ResourceManagerError::InvalidRequestData(e)))?;
                let mut node_ref = system_api
                    .borrow_node_mut(&RENodeId::ResourceManager(resource_address))
                    .map_err(InvokeError::Downstream)?;
                let resource_manager = node_ref.resource_manager_mut();

                resource_manager.update_metadata(input.metadata)?;
                Ok(ScryptoValue::from_typed(&()))
            }
            ResourceManagerMethodFnIdent::UpdateNonFungibleData => {
                let input: ResourceManagerUpdateNonFungibleDataInput = scrypto_decode(&args.raw)
                    .map_err(|e| InvokeError::Error(ResourceManagerError::InvalidRequestData(e)))?;
                // Read current value
                let value = system_api
                    .substate_read(SubstateId(
                        RENodeId::ResourceManager(resource_address.clone()),
                        SubstateOffset::Resource(ResourceManagerOffset::NonFungible(
                            input.id.clone(),
                        )),
                    ))
                    .map_err(InvokeError::Downstream)?;
                let wrapper: NonFungibleSubstate =
                    scrypto_decode(&value.raw).expect("Failed to decode NonFungibleSubstate");

                // Write new value
                if let Some(mut non_fungible) = wrapper.0 {
                    non_fungible.set_mutable_data(input.data);
                    system_api
                        .substate_write(
                            SubstateId(
                                RENodeId::ResourceManager(resource_address.clone()),
                                SubstateOffset::Resource(ResourceManagerOffset::NonFungible(
                                    input.id,
                                )),
                            ),
                            ScryptoValue::from_typed(&NonFungibleSubstate(Some(non_fungible))),
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
            ResourceManagerMethodFnIdent::NonFungibleExists => {
                let input: ResourceManagerNonFungibleExistsInput = scrypto_decode(&args.raw)
                    .map_err(|e| InvokeError::Error(ResourceManagerError::InvalidRequestData(e)))?;
                let value = system_api
                    .substate_read(SubstateId(
                        RENodeId::ResourceManager(resource_address.clone()),
                        SubstateOffset::Resource(ResourceManagerOffset::NonFungible(input.id)),
                    ))
                    .map_err(InvokeError::Downstream)?;
                let wrapper: NonFungibleSubstate =
                    scrypto_decode(&value.raw).expect("Failed to decode NonFungibleSubstate");
                Ok(ScryptoValue::from_typed(&wrapper.0.is_some()))
            }
            ResourceManagerMethodFnIdent::GetNonFungible => {
                let input: ResourceManagerGetNonFungibleInput = scrypto_decode(&args.raw)
                    .map_err(|e| InvokeError::Error(ResourceManagerError::InvalidRequestData(e)))?;
                let non_fungible_address =
                    NonFungibleAddress::new(resource_address.clone(), input.id.clone());
                let value = system_api
                    .substate_read(SubstateId(
                        RENodeId::ResourceManager(resource_address.clone()),
                        SubstateOffset::Resource(ResourceManagerOffset::NonFungible(input.id)),
                    ))
                    .map_err(InvokeError::Downstream)?;
                let wrapper: NonFungibleSubstate =
                    scrypto_decode(&value.raw).expect("Failed to decode NonFungibleSubstate");
                let non_fungible = wrapper.0.ok_or(InvokeError::Error(
                    ResourceManagerError::NonFungibleNotFound(non_fungible_address),
                ))?;
                Ok(ScryptoValue::from_typed(&[
                    non_fungible.immutable_data(),
                    non_fungible.mutable_data(),
                ]))
            }
        }?;

        Ok(rtn)
    }
}
