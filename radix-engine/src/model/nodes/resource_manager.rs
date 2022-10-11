use crate::engine::{HeapRENode, SystemApi};
use crate::fee::FeeReserve;
use crate::model::{
    Bucket, InvokeError, NonFungible, NonFungibleSubstate, Resource,
    ResourceMethodRule::{Protected, Public},
    Substate, Vault,
};
use crate::model::{
    MethodAccessRule, MethodAccessRuleMethod, NonFungibleStore, ResourceManagerSubstate,
    ResourceMethodRule,
};
use crate::types::AccessRule::*;
use crate::types::ResourceMethodAuthKey::*;
use crate::types::*;
use crate::wasm::*;
use scrypto::core::ResourceManagerFunction;
use scrypto::resource::ResourceManagerBurnInput;

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
    CouldNotCreateBucket,
    CouldNotCreateVault,
    NotNonFungible,
    MismatchingBucketResource,
}

#[derive(Debug)]
pub struct ResourceManager {
    pub info: ResourceManagerSubstate,
}

impl ResourceManager {
    pub fn new(
        resource_type: ResourceType,
        metadata: HashMap<String, String>,
        mut auth: HashMap<ResourceMethodAuthKey, (AccessRule, Mutability)>,
        non_fungible_store_id: Option<NonFungibleStoreId>,
    ) -> Result<Self, InvokeError<ResourceManagerError>> {
        let mut vault_method_table: HashMap<VaultMethod, ResourceMethodRule> = HashMap::new();
        vault_method_table.insert(VaultMethod::LockFee, Protected(Withdraw));
        vault_method_table.insert(VaultMethod::LockContingentFee, Protected(Withdraw));
        vault_method_table.insert(VaultMethod::Take, Protected(Withdraw));
        vault_method_table.insert(VaultMethod::Put, Protected(Deposit));
        vault_method_table.insert(VaultMethod::GetAmount, Public);
        vault_method_table.insert(VaultMethod::GetResourceAddress, Public);
        vault_method_table.insert(VaultMethod::GetNonFungibleIds, Public);
        vault_method_table.insert(VaultMethod::CreateProof, Public);
        vault_method_table.insert(VaultMethod::CreateProofByAmount, Public);
        vault_method_table.insert(VaultMethod::CreateProofByIds, Public);
        vault_method_table.insert(VaultMethod::TakeNonFungibles, Protected(Withdraw));

        let mut bucket_method_table: HashMap<BucketMethod, ResourceMethodRule> = HashMap::new();
        bucket_method_table.insert(BucketMethod::Burn, Protected(Burn));

        let mut method_table: HashMap<ResourceManagerMethod, ResourceMethodRule> = HashMap::new();
        method_table.insert(ResourceManagerMethod::Mint, Protected(Mint));
        method_table.insert(
            ResourceManagerMethod::UpdateMetadata,
            Protected(UpdateMetadata),
        );
        method_table.insert(ResourceManagerMethod::CreateBucket, Public);
        method_table.insert(ResourceManagerMethod::GetMetadata, Public);
        method_table.insert(ResourceManagerMethod::GetResourceType, Public);
        method_table.insert(ResourceManagerMethod::GetTotalSupply, Public);
        method_table.insert(ResourceManagerMethod::CreateVault, Public);
        method_table.insert(ResourceManagerMethod::Burn, Public);

        // Non Fungible methods
        method_table.insert(
            ResourceManagerMethod::UpdateNonFungibleData,
            Protected(UpdateNonFungibleData),
        );
        method_table.insert(ResourceManagerMethod::NonFungibleExists, Public);
        method_table.insert(ResourceManagerMethod::GetNonFungible, Public);

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
                non_fungible_store_id,
            },
        };

        Ok(resource_manager)
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
        func: ResourceManagerFunction,
        args: ScryptoValue,
        system_api: &mut Y,
    ) -> Result<ScryptoValue, InvokeError<ResourceManagerError>>
    where
        Y: SystemApi<'s, W, I, R>,
        W: WasmEngine<I>,
        I: WasmInstance,
        R: FeeReserve,
    {
        match func {
            ResourceManagerFunction::Create => {
                let input: ResourceManagerCreateInput = scrypto_decode(&args.raw)
                    .map_err(|e| InvokeError::Error(ResourceManagerError::InvalidRequestData(e)))?;

                let resource_node_id = if matches!(input.resource_type, ResourceType::NonFungible) {
                    let non_fungible_store_node_id = system_api
                        .node_create(HeapRENode::NonFungibleStore(NonFungibleStore::new()))
                        .map_err(InvokeError::Downstream)?;
                    let non_fungible_store_id: NonFungibleStoreId =
                        non_fungible_store_node_id.into();

                    let mut resource_manager = ResourceManager::new(
                        input.resource_type,
                        input.metadata,
                        input.access_rules,
                        Some(non_fungible_store_id),
                    )?;

                    if let Some(mint_params) = &input.mint_params {
                        if let MintParams::NonFungible { entries } = mint_params {
                            for (non_fungible_id, data) in entries {
                                let offset = SubstateOffset::NonFungibleStore(
                                    NonFungibleStoreOffset::Entry(non_fungible_id.clone()),
                                );
                                let handle = system_api
                                    .lock_substate(non_fungible_store_node_id, offset, true)
                                    .map_err(InvokeError::Downstream)?;
                                let mut substate_mut = system_api
                                    .get_mut(handle)
                                    .map_err(InvokeError::Downstream)?;
                                substate_mut
                                    .overwrite(Substate::NonFungible(NonFungibleSubstate(Some(
                                        NonFungible::new(data.0.clone(), data.1.clone()), // FIXME: verify data
                                    ))))
                                    .map_err(InvokeError::Downstream)?;

                                system_api
                                    .drop_lock(handle)
                                    .map_err(InvokeError::Downstream)?;
                            }
                            resource_manager.info.total_supply = entries.len().into();
                        } else {
                            return Err(InvokeError::Error(
                                ResourceManagerError::ResourceTypeDoesNotMatch,
                            ));
                        }
                    }
                    system_api
                        .node_create(HeapRENode::ResourceManager(resource_manager))
                        .map_err(InvokeError::Downstream)?
                } else {
                    let mut resource_manager = ResourceManager::new(
                        input.resource_type,
                        input.metadata,
                        input.access_rules,
                        None,
                    )?;

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
                        .node_create(HeapRENode::ResourceManager(resource_manager))
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
        method: ResourceManagerMethod,
        args: ScryptoValue,
        system_api: &mut Y,
    ) -> Result<ScryptoValue, InvokeError<ResourceManagerError>>
    where
        Y: SystemApi<'s, W, I, R>,
        W: WasmEngine<I>,
        I: WasmInstance,
        R: FeeReserve,
    {
        let node_id = RENodeId::ResourceManager(resource_address);

        let rtn = match method {
            ResourceManagerMethod::Burn => {
                let input: ResourceManagerBurnInput = scrypto_decode(&args.raw)
                    .map_err(|e| InvokeError::Error(ResourceManagerError::InvalidRequestData(e)))?;

                let bucket: Bucket = system_api
                    .node_drop(RENodeId::Bucket(input.bucket.0))
                    .map_err(InvokeError::Downstream)?
                    .into();

                // Check if resource matches
                // TODO: Move this check into actor check
                if bucket.resource_address() != resource_address {
                    return Err(InvokeError::Error(
                        ResourceManagerError::MismatchingBucketResource,
                    ));
                }

                let offset =
                    SubstateOffset::ResourceManager(ResourceManagerOffset::ResourceManager);
                let handle = system_api
                    .lock_substate(node_id, offset, true)
                    .map_err(InvokeError::Downstream)?;
                // Update total supply
                // TODO: there might be better for maintaining total supply, especially for non-fungibles
                // where we can leverage capabilities of key-value map.
                {
                    let mut substate_mut = system_api
                        .get_mut(handle)
                        .map_err(InvokeError::Downstream)?;
                    let mut raw_mut = substate_mut.get_raw_mut();
                    raw_mut.resource_manager().total_supply -= bucket.total_amount();
                    substate_mut.flush().map_err(InvokeError::Downstream)?;
                }

                let substate_ref = system_api
                    .get_ref(handle)
                    .map_err(InvokeError::Downstream)?;
                let resource_manager = substate_ref.resource_manager();

                // Burn non-fungible
                if let Some(non_fungible_store_id) = resource_manager.non_fungible_store_id {
                    let node_id = RENodeId::NonFungibleStore(non_fungible_store_id);

                    for id in bucket
                        .total_ids()
                        .expect("Failed to list non-fungible IDs on non-fungible Bucket")
                    {
                        let offset =
                            SubstateOffset::NonFungibleStore(NonFungibleStoreOffset::Entry(id));
                        let handle = system_api
                            .lock_substate(node_id, offset, true)
                            .map_err(InvokeError::Downstream)?;
                        let mut substate_mut = system_api
                            .get_mut(handle)
                            .map_err(InvokeError::Downstream)?;
                        substate_mut
                            .overwrite(Substate::NonFungible(NonFungibleSubstate(None)))
                            .map_err(InvokeError::Downstream)?;
                        system_api
                            .drop_lock(handle)
                            .map_err(InvokeError::Downstream)?;
                    }
                }

                Ok(ScryptoValue::from_typed(&()))
            }
            ResourceManagerMethod::UpdateAuth => {
                let input: ResourceManagerUpdateAuthInput = scrypto_decode(&args.raw)
                    .map_err(|e| InvokeError::Error(ResourceManagerError::InvalidRequestData(e)))?;
                let offset =
                    SubstateOffset::ResourceManager(ResourceManagerOffset::ResourceManager);

                let handle = system_api
                    .lock_substate(node_id, offset, true)
                    .map_err(InvokeError::Downstream)?;
                let mut substate_mut = system_api
                    .get_mut(handle)
                    .map_err(InvokeError::Downstream)?;
                let mut raw_mut = substate_mut.get_raw_mut();
                let method_entry = raw_mut
                    .resource_manager()
                    .authorization
                    .get_mut(&input.method)
                    .expect(&format!(
                        "Authorization for {:?} not specified",
                        input.method
                    ));
                method_entry.main(MethodAccessRuleMethod::Update(input.access_rule))?;
                substate_mut.flush().map_err(InvokeError::Downstream)?;

                system_api
                    .drop_lock(handle)
                    .map_err(InvokeError::Downstream)?;

                Ok(ScryptoValue::unit())
            }
            ResourceManagerMethod::LockAuth => {
                let input: ResourceManagerLockAuthInput = scrypto_decode(&args.raw)
                    .map_err(|e| InvokeError::Error(ResourceManagerError::InvalidRequestData(e)))?;
                let offset =
                    SubstateOffset::ResourceManager(ResourceManagerOffset::ResourceManager);
                let handle = system_api
                    .lock_substate(node_id, offset, true)
                    .map_err(InvokeError::Downstream)?;
                let mut substate_mut = system_api
                    .get_mut(handle)
                    .map_err(InvokeError::Downstream)?;
                let mut raw_mut = substate_mut.get_raw_mut();
                let method_entry = raw_mut
                    .resource_manager()
                    .authorization
                    .get_mut(&input.method)
                    .expect(&format!(
                        "Authorization for {:?} not specified",
                        input.method
                    ));
                method_entry.main(MethodAccessRuleMethod::Lock())?;
                substate_mut.flush().map_err(InvokeError::Downstream)?;

                system_api
                    .drop_lock(handle)
                    .map_err(InvokeError::Downstream)?;
                Ok(ScryptoValue::unit())
            }
            ResourceManagerMethod::CreateVault => {
                let _: ResourceManagerCreateVaultInput = scrypto_decode(&args.raw)
                    .map_err(|e| InvokeError::Error(ResourceManagerError::InvalidRequestData(e)))?;

                let offset =
                    SubstateOffset::ResourceManager(ResourceManagerOffset::ResourceManager);
                let resource_type = system_api
                    .read_substate(node_id, offset, |s| s.resource_manager().resource_type)
                    .map_err(InvokeError::Downstream)?;

                let resource = Resource::new_empty(resource_address, resource_type);
                let vault_id = system_api
                    .node_create(HeapRENode::Vault(Vault::new(resource)))
                    .map_err(InvokeError::Downstream)?
                    .into();
                Ok(ScryptoValue::from_typed(&scrypto::resource::Vault(
                    vault_id,
                )))
            }
            ResourceManagerMethod::CreateBucket => {
                let _: ResourceManagerCreateBucketInput = scrypto_decode(&args.raw)
                    .map_err(|e| InvokeError::Error(ResourceManagerError::InvalidRequestData(e)))?;
                let offset =
                    SubstateOffset::ResourceManager(ResourceManagerOffset::ResourceManager);
                let resource_type = system_api
                    .read_substate(node_id, offset, |s| s.resource_manager().resource_type)
                    .map_err(InvokeError::Downstream)?;

                let container = Resource::new_empty(resource_address, resource_type);
                let bucket_id = system_api
                    .node_create(HeapRENode::Bucket(Bucket::new(container)))
                    .map_err(InvokeError::Downstream)?
                    .into();
                Ok(ScryptoValue::from_typed(&scrypto::resource::Bucket(
                    bucket_id,
                )))
            }
            ResourceManagerMethod::Mint => {
                let input: ResourceManagerMintInput = scrypto_decode(&args.raw)
                    .map_err(|e| InvokeError::Error(ResourceManagerError::InvalidRequestData(e)))?;

                let offset =
                    SubstateOffset::ResourceManager(ResourceManagerOffset::ResourceManager);
                let handle = system_api
                    .lock_substate(node_id, offset, true)
                    .map_err(InvokeError::Downstream)?;

                let (resource, non_fungibles) = {
                    let mut substate_mut = system_api
                        .get_mut(handle)
                        .map_err(InvokeError::Downstream)?;
                    let mut raw_mut = substate_mut.get_raw_mut();
                    let resource_manager = raw_mut.resource_manager();
                    let result = resource_manager.mint(input.mint_params, resource_address)?;
                    substate_mut.flush().map_err(InvokeError::Downstream)?;
                    result
                };

                let bucket_id = system_api
                    .node_create(HeapRENode::Bucket(Bucket::new(resource)))
                    .map_err(InvokeError::Downstream)?
                    .into();

                let substate_ref = system_api
                    .get_ref(handle)
                    .map_err(InvokeError::Downstream)?;
                let resource_manager = substate_ref.resource_manager();
                let non_fungible_store_id = resource_manager.non_fungible_store_id.clone();
                for (id, non_fungible) in non_fungibles {
                    let node_id = RENodeId::NonFungibleStore(non_fungible_store_id.unwrap());
                    let offset =
                        SubstateOffset::NonFungibleStore(NonFungibleStoreOffset::Entry(id.clone()));
                    let non_fungible_handle = system_api
                        .lock_substate(node_id, offset, true)
                        .map_err(InvokeError::Downstream)?;
                    let substate_ref = system_api
                        .get_ref(non_fungible_handle)
                        .map_err(InvokeError::Downstream)?;

                    let wrapper = substate_ref.non_fungible();
                    if wrapper.0.is_some() {
                        return Err(InvokeError::Error(
                            ResourceManagerError::NonFungibleAlreadyExists(
                                NonFungibleAddress::new(resource_address, id.clone()),
                            ),
                        ));
                    }

                    {
                        let mut substate_mut = system_api
                            .get_mut(non_fungible_handle)
                            .map_err(InvokeError::Downstream)?;
                        substate_mut
                            .overwrite(Substate::NonFungible(NonFungibleSubstate(Some(
                                non_fungible,
                            ))))
                            .map_err(InvokeError::Downstream)?;
                    }

                    system_api
                        .drop_lock(non_fungible_handle)
                        .map_err(InvokeError::Downstream)?;
                }

                system_api
                    .drop_lock(handle)
                    .map_err(InvokeError::Downstream)?;

                Ok(ScryptoValue::from_typed(&scrypto::resource::Bucket(
                    bucket_id,
                )))
            }
            ResourceManagerMethod::GetMetadata => {
                let _: ResourceManagerGetMetadataInput = scrypto_decode(&args.raw)
                    .map_err(|e| InvokeError::Error(ResourceManagerError::InvalidRequestData(e)))?;

                let offset =
                    SubstateOffset::ResourceManager(ResourceManagerOffset::ResourceManager);
                let scrypto_value = system_api
                    .read_substate(node_id, offset, |s| {
                        ScryptoValue::from_typed(&s.resource_manager().metadata)
                    })
                    .map_err(InvokeError::Downstream)?;
                Ok(scrypto_value)
            }
            ResourceManagerMethod::GetResourceType => {
                let _: ResourceManagerGetResourceTypeInput = scrypto_decode(&args.raw)
                    .map_err(|e| InvokeError::Error(ResourceManagerError::InvalidRequestData(e)))?;

                let offset =
                    SubstateOffset::ResourceManager(ResourceManagerOffset::ResourceManager);
                let resource_type = system_api
                    .read_substate(node_id, offset, |s| s.resource_manager().resource_type)
                    .map_err(InvokeError::Downstream)?;

                Ok(ScryptoValue::from_typed(&resource_type))
            }
            ResourceManagerMethod::GetTotalSupply => {
                let _: ResourceManagerGetTotalSupplyInput = scrypto_decode(&args.raw)
                    .map_err(|e| InvokeError::Error(ResourceManagerError::InvalidRequestData(e)))?;

                let offset =
                    SubstateOffset::ResourceManager(ResourceManagerOffset::ResourceManager);
                let total_supply = system_api
                    .read_substate(node_id, offset, |s| s.resource_manager().total_supply)
                    .map_err(InvokeError::Downstream)?;

                Ok(ScryptoValue::from_typed(&total_supply))
            }
            ResourceManagerMethod::UpdateMetadata => {
                let input: ResourceManagerUpdateMetadataInput = scrypto_decode(&args.raw)
                    .map_err(|e| InvokeError::Error(ResourceManagerError::InvalidRequestData(e)))?;

                let offset =
                    SubstateOffset::ResourceManager(ResourceManagerOffset::ResourceManager);
                let handle = system_api
                    .lock_substate(node_id, offset, true)
                    .map_err(InvokeError::Downstream)?;
                let mut substate_mut = system_api
                    .get_mut(handle)
                    .map_err(InvokeError::Downstream)?;
                let mut raw_mut = substate_mut.get_raw_mut();
                raw_mut.resource_manager().update_metadata(input.metadata)?;
                substate_mut.flush().map_err(InvokeError::Downstream)?;
                system_api
                    .drop_lock(handle)
                    .map_err(InvokeError::Downstream)?;

                Ok(ScryptoValue::from_typed(&()))
            }
            ResourceManagerMethod::UpdateNonFungibleData => {
                let input: ResourceManagerUpdateNonFungibleDataInput = scrypto_decode(&args.raw)
                    .map_err(|e| InvokeError::Error(ResourceManagerError::InvalidRequestData(e)))?;

                let offset =
                    SubstateOffset::ResourceManager(ResourceManagerOffset::ResourceManager);
                let handle = system_api
                    .lock_substate(node_id, offset, true)
                    .map_err(InvokeError::Downstream)?;
                let substate_ref = system_api
                    .get_ref(handle)
                    .map_err(InvokeError::Downstream)?;
                let resource_manager = substate_ref.resource_manager();
                let non_fungible_store_id = resource_manager
                    .non_fungible_store_id
                    .ok_or(InvokeError::Error(ResourceManagerError::NotNonFungible))?;

                let node_id = RENodeId::NonFungibleStore(non_fungible_store_id);
                let offset = SubstateOffset::NonFungibleStore(NonFungibleStoreOffset::Entry(
                    input.id.clone(),
                ));

                let lock_handle = system_api
                    .lock_substate(node_id, offset, true)
                    .map_err(InvokeError::Downstream)?;

                // Read current value
                let substate_ref = system_api
                    .get_ref(lock_handle)
                    .map_err(InvokeError::Downstream)?;
                let wrapper = substate_ref.non_fungible();

                // Write new value
                if let Some(mut non_fungible) = wrapper.0.clone() {
                    non_fungible.set_mutable_data(input.data);
                    let mut substate_mut = system_api
                        .get_mut(lock_handle)
                        .map_err(InvokeError::Downstream)?;
                    substate_mut
                        .overwrite(Substate::NonFungible(NonFungibleSubstate(Some(
                            non_fungible,
                        ))))
                        .map_err(InvokeError::Downstream)?;
                } else {
                    let non_fungible_address =
                        NonFungibleAddress::new(resource_address.clone(), input.id);
                    return Err(InvokeError::Error(
                        ResourceManagerError::NonFungibleNotFound(non_fungible_address.clone()),
                    ));
                }

                system_api
                    .drop_lock(lock_handle)
                    .map_err(InvokeError::Downstream)?;

                Ok(ScryptoValue::from_typed(&()))
            }
            ResourceManagerMethod::NonFungibleExists => {
                let input: ResourceManagerNonFungibleExistsInput = scrypto_decode(&args.raw)
                    .map_err(|e| InvokeError::Error(ResourceManagerError::InvalidRequestData(e)))?;
                let offset =
                    SubstateOffset::ResourceManager(ResourceManagerOffset::ResourceManager);
                let handle = system_api
                    .lock_substate(node_id, offset, true)
                    .map_err(InvokeError::Downstream)?;
                let substate_ref = system_api
                    .get_ref(handle)
                    .map_err(InvokeError::Downstream)?;
                let resource_manager = substate_ref.resource_manager();
                let non_fungible_store_id = resource_manager
                    .non_fungible_store_id
                    .ok_or(InvokeError::Error(ResourceManagerError::NotNonFungible))?;

                let node_id = RENodeId::NonFungibleStore(non_fungible_store_id);
                let offset =
                    SubstateOffset::NonFungibleStore(NonFungibleStoreOffset::Entry(input.id));
                let exists = system_api
                    .read_substate(node_id, offset, |s| s.non_fungible().0.is_some())
                    .map_err(InvokeError::Downstream)?;

                Ok(ScryptoValue::from_typed(&exists))
            }
            ResourceManagerMethod::GetNonFungible => {
                let input: ResourceManagerGetNonFungibleInput = scrypto_decode(&args.raw)
                    .map_err(|e| InvokeError::Error(ResourceManagerError::InvalidRequestData(e)))?;
                let offset =
                    SubstateOffset::ResourceManager(ResourceManagerOffset::ResourceManager);
                let handle = system_api
                    .lock_substate(node_id, offset, true)
                    .map_err(InvokeError::Downstream)?;
                let substate_ref = system_api
                    .get_ref(handle)
                    .map_err(InvokeError::Downstream)?;
                let resource_manager = substate_ref.resource_manager();
                let non_fungible_store_id = resource_manager
                    .non_fungible_store_id
                    .ok_or(InvokeError::Error(ResourceManagerError::NotNonFungible))?;

                let non_fungible_address =
                    NonFungibleAddress::new(resource_address.clone(), input.id.clone());

                let node_id = RENodeId::NonFungibleStore(non_fungible_store_id);
                let offset =
                    SubstateOffset::NonFungibleStore(NonFungibleStoreOffset::Entry(input.id));
                let scrypto_value = system_api
                    .read_substate(node_id, offset, |s| {
                        let wrapper = s.non_fungible();
                        if let Some(non_fungible) = wrapper.0.as_ref() {
                            let scrypto_value = ScryptoValue::from_typed(&[
                                non_fungible.immutable_data(),
                                non_fungible.mutable_data(),
                            ]);
                            Ok(scrypto_value)
                        } else {
                            Err(InvokeError::Error(
                                ResourceManagerError::NonFungibleNotFound(non_fungible_address),
                            ))
                        }
                    })
                    .map_err(InvokeError::Downstream)??;

                Ok(scrypto_value)
            }
        }?;

        Ok(rtn)
    }
}
