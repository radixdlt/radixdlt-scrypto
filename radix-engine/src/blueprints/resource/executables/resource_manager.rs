use crate::blueprints::resource::*;
use crate::errors::ApplicationError;
use crate::errors::InterpreterError;
use crate::errors::InvokeError;
use crate::errors::RuntimeError;
use crate::kernel::actor::ResolvedActor;
use crate::kernel::call_frame::CallFrameUpdate;
use crate::kernel::interpreters::deref_and_update;
use crate::kernel::kernel_api::{
    ExecutableInvocation, Executor, KernelNodeApi, KernelSubstateApi, LockFlags,
};
use crate::system::global::GlobalAddressSubstate;
use crate::system::node::RENodeInit;
use crate::system::node::RENodeModuleInit;
use crate::system::node_modules::auth::AccessRulesChainSubstate;
use crate::system::node_modules::metadata::MetadataSubstate;
use crate::types::*;
use crate::wasm::WasmEngine;
use native_sdk::resource::SysBucket;
use native_sdk::runtime::Runtime;
use radix_engine_interface::api::node_modules::auth::AuthZoneAssertAccessRuleInvocation;
use radix_engine_interface::api::types::*;
use radix_engine_interface::api::types::{
    GlobalAddress, NativeFn, NonFungibleStoreId, NonFungibleStoreOffset, RENodeId,
    ResourceManagerFn, ResourceManagerOffset, SubstateOffset,
};
use radix_engine_interface::api::ClientSubstateApi;
use radix_engine_interface::api::{ClientApi, ClientDerefApi};
use radix_engine_interface::api::{ClientNativeInvokeApi, ClientNodeApi};
use radix_engine_interface::blueprints::resource::AccessRule::{AllowAll, DenyAll};
use radix_engine_interface::blueprints::resource::VaultMethodAuthKey::{Deposit, Recall, Withdraw};
use radix_engine_interface::blueprints::resource::*;
use radix_engine_interface::data::model::Own;
use radix_engine_interface::data::ScryptoValue;
use radix_engine_interface::math::Decimal;
use radix_engine_interface::*;

/// Represents an error when accessing a bucket.
#[derive(Debug, Clone, PartialEq, Eq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub enum ResourceManagerError {
    InvalidAmount(Decimal, u8),
    MaxMintAmountExceeded,
    NonFungibleAlreadyExists(NonFungibleGlobalId),
    NonFungibleNotFound(NonFungibleGlobalId),
    NotNonFungible,
    MismatchingBucketResource,
    NonFungibleIdTypeDoesNotMatch(NonFungibleIdType, NonFungibleIdType),
    ResourceTypeDoesNotMatch,
    InvalidNonFungibleIdType,
}

impl ExecutableInvocation for ResourceManagerBurnBucketInvocation {
    type Exec = Self;

    fn resolve<D: ClientDerefApi<RuntimeError>>(
        self,
        _api: &mut D,
    ) -> Result<(ResolvedActor, CallFrameUpdate, Self::Exec), RuntimeError> {
        let call_frame_update = CallFrameUpdate::move_node(RENodeId::Bucket(self.bucket.0));
        let actor =
            ResolvedActor::function(NativeFn::ResourceManager(ResourceManagerFn::BurnBucket));
        Ok((actor, call_frame_update, self))
    }
}

impl Executor for ResourceManagerBurnBucketInvocation {
    type Output = ();

    fn execute<Y, W: WasmEngine>(self, env: &mut Y) -> Result<((), CallFrameUpdate), RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientNativeInvokeApi<RuntimeError>,
    {
        let bucket = Bucket(self.bucket.0);
        bucket.sys_burn(env)?;

        Ok(((), CallFrameUpdate::empty()))
    }
}

fn build_non_fungible_resource_manager_substate_with_initial_supply<Y>(
    resource_address: ResourceAddress,
    id_type: NonFungibleIdType,
    entries: BTreeMap<NonFungibleLocalId, (Vec<u8>, Vec<u8>)>,
    api: &mut Y,
) -> Result<(ResourceManagerSubstate, Bucket), RuntimeError>
where
    Y: KernelNodeApi + KernelSubstateApi,
{
    let nf_store_node_id = api.kernel_allocate_node_id(RENodeType::NonFungibleStore)?;
    api.kernel_create_node(
        nf_store_node_id,
        RENodeInit::NonFungibleStore(NonFungibleStore::new()),
        BTreeMap::new(),
    )?;
    let nf_store_id: NonFungibleStoreId = nf_store_node_id.into();

    let mut resource_manager = ResourceManagerSubstate::new(
        ResourceType::NonFungible { id_type },
        Some(nf_store_id),
        resource_address,
    );

    let bucket = {
        for (non_fungible_local_id, data) in &entries {
            if non_fungible_local_id.id_type() != id_type {
                return Err(RuntimeError::ApplicationError(
                    ApplicationError::ResourceManagerError(
                        ResourceManagerError::NonFungibleIdTypeDoesNotMatch(
                            non_fungible_local_id.id_type(),
                            id_type,
                        ),
                    ),
                ));
            }

            let offset = SubstateOffset::NonFungibleStore(NonFungibleStoreOffset::Entry(
                non_fungible_local_id.clone(),
            ));
            let non_fungible_handle = api.kernel_lock_substate(
                nf_store_node_id,
                NodeModuleId::SELF,
                offset,
                LockFlags::MUTABLE,
            )?;
            let mut substate_mut = api.kernel_get_substate_ref_mut(non_fungible_handle)?;
            let non_fungible_mut = substate_mut.non_fungible();
            *non_fungible_mut = NonFungibleSubstate(Some(
                NonFungible::new(data.0.clone(), data.1.clone()), // FIXME: verify data
            ));
            api.kernel_drop_lock(non_fungible_handle)?;
        }
        resource_manager.total_supply = entries.len().into();
        let ids = entries.into_keys().collect();
        let container = Resource::new_non_fungible(resource_address, ids, id_type);
        let node_id = api.kernel_allocate_node_id(RENodeType::Bucket)?;
        api.kernel_create_node(
            node_id,
            RENodeInit::Bucket(BucketSubstate::new(container)),
            BTreeMap::new(),
        )?;
        let bucket_id = node_id.into();
        Bucket(bucket_id)
    };

    Ok((resource_manager, bucket))
}

fn build_fungible_resource_manager_substate_with_initial_supply<Y>(
    resource_address: ResourceAddress,
    divisibility: u8,
    initial_supply: Decimal,
    api: &mut Y,
) -> Result<(ResourceManagerSubstate, Bucket), RuntimeError>
where
    Y: KernelNodeApi + KernelSubstateApi,
{
    let mut resource_manager = ResourceManagerSubstate::new(
        ResourceType::Fungible { divisibility },
        None,
        resource_address,
    );

    let bucket = {
        resource_manager.check_fungible_amount(initial_supply)?;
        // TODO: refactor this into mint function
        if initial_supply > dec!("1000000000000000000") {
            return Err(RuntimeError::ApplicationError(
                ApplicationError::ResourceManagerError(ResourceManagerError::MaxMintAmountExceeded),
            ));
        }
        resource_manager.total_supply = initial_supply;
        let container = Resource::new_fungible(resource_address, divisibility, initial_supply);
        let node_id = api.kernel_allocate_node_id(RENodeType::Bucket)?;
        api.kernel_create_node(
            node_id,
            RENodeInit::Bucket(BucketSubstate::new(container)),
            BTreeMap::new(),
        )?;
        let bucket_id = node_id.into();
        Bucket(bucket_id)
    };

    Ok((resource_manager, bucket))
}

fn build_substates(
    mut access_rules_map: BTreeMap<ResourceMethodAuthKey, (AccessRule, AccessRule)>,
) -> (AccessRulesChainSubstate, AccessRulesChainSubstate) {
    let (mint_access_rule, mint_mutability) = access_rules_map
        .remove(&Mint)
        .unwrap_or((DenyAll, rule!(deny_all)));
    let (burn_access_rule, burn_mutability) = access_rules_map
        .remove(&Burn)
        .unwrap_or((DenyAll, rule!(deny_all)));
    let (update_non_fungible_data_access_rule, update_non_fungible_data_mutability) =
        access_rules_map
            .remove(&UpdateNonFungibleData)
            .unwrap_or((AllowAll, rule!(deny_all)));
    let (update_metadata_access_rule, update_metadata_mutability) = access_rules_map
        .remove(&UpdateMetadata)
        .unwrap_or((DenyAll, rule!(deny_all)));

    let mut access_rules = AccessRules::new();
    access_rules.set_access_rule_and_mutability(
        AccessRuleKey::Native(NativeFn::Metadata(MetadataFn::Set)),
        update_metadata_access_rule,
        update_metadata_mutability,
    );
    access_rules.set_access_rule_and_mutability(
        AccessRuleKey::Native(NativeFn::Metadata(MetadataFn::Get)),
        AllowAll,
        DenyAll,
    );
    access_rules.set_group_access_rule_and_mutability(
        "mint".to_string(),
        mint_access_rule,
        mint_mutability,
    );
    access_rules.set_group_and_mutability(
        AccessRuleKey::Native(NativeFn::ResourceManager(
            ResourceManagerFn::MintNonFungible,
        )),
        "mint".to_string(),
        DenyAll,
    );
    access_rules.set_group_and_mutability(
        AccessRuleKey::Native(NativeFn::ResourceManager(
            ResourceManagerFn::MintUuidNonFungible,
        )),
        "mint".to_string(),
        DenyAll,
    );
    access_rules.set_group_and_mutability(
        AccessRuleKey::Native(NativeFn::ResourceManager(ResourceManagerFn::MintFungible)),
        "mint".to_string(),
        DenyAll,
    );

    access_rules.set_access_rule_and_mutability(
        AccessRuleKey::Native(NativeFn::ResourceManager(ResourceManagerFn::Burn)),
        burn_access_rule,
        burn_mutability,
    );
    access_rules.set_access_rule_and_mutability(
        AccessRuleKey::Native(NativeFn::ResourceManager(
            ResourceManagerFn::UpdateNonFungibleData,
        )),
        update_non_fungible_data_access_rule,
        update_non_fungible_data_mutability,
    );
    access_rules.set_access_rule_and_mutability(
        AccessRuleKey::Native(NativeFn::ResourceManager(ResourceManagerFn::CreateBucket)),
        AllowAll,
        DenyAll,
    );
    access_rules.set_access_rule_and_mutability(
        AccessRuleKey::Native(NativeFn::ResourceManager(
            ResourceManagerFn::GetResourceType,
        )),
        AllowAll,
        DenyAll,
    );
    access_rules.set_access_rule_and_mutability(
        AccessRuleKey::Native(NativeFn::ResourceManager(ResourceManagerFn::GetTotalSupply)),
        AllowAll,
        DenyAll,
    );
    access_rules.set_access_rule_and_mutability(
        AccessRuleKey::Native(NativeFn::ResourceManager(ResourceManagerFn::CreateVault)),
        AllowAll,
        DenyAll,
    );
    access_rules.set_access_rule_and_mutability(
        AccessRuleKey::Native(NativeFn::ResourceManager(
            ResourceManagerFn::NonFungibleExists,
        )),
        AllowAll,
        DenyAll,
    );
    access_rules.set_access_rule_and_mutability(
        AccessRuleKey::Native(NativeFn::ResourceManager(ResourceManagerFn::GetNonFungible)),
        AllowAll,
        DenyAll,
    );
    access_rules.set_access_rule_and_mutability(
        AccessRuleKey::Native(NativeFn::ResourceManager(
            ResourceManagerFn::UpdateVaultAuth,
        )),
        AllowAll, // Access verification occurs within method
        DenyAll,
    );
    access_rules.set_access_rule_and_mutability(
        AccessRuleKey::Native(NativeFn::ResourceManager(
            ResourceManagerFn::SetVaultAuthMutability,
        )),
        AllowAll, // Access verification occurs within method
        DenyAll,
    );

    let substate = AccessRulesChainSubstate {
        access_rules_chain: vec![access_rules],
    };

    let (deposit_access_rule, deposit_mutability) = access_rules_map
        .remove(&ResourceMethodAuthKey::Deposit)
        .unwrap_or((AllowAll, rule!(deny_all)));
    let (withdraw_access_rule, withdraw_mutability) = access_rules_map
        .remove(&ResourceMethodAuthKey::Withdraw)
        .unwrap_or((AllowAll, rule!(deny_all)));
    let (recall_access_rule, recall_mutability) = access_rules_map
        .remove(&ResourceMethodAuthKey::Recall)
        .unwrap_or((DenyAll, rule!(deny_all)));

    let mut vault_access_rules = AccessRules::new();
    vault_access_rules.set_group_access_rule_and_mutability(
        "withdraw".to_string(),
        withdraw_access_rule,
        withdraw_mutability,
    );
    vault_access_rules.set_group_access_rule_and_mutability(
        "recall".to_string(),
        recall_access_rule,
        recall_mutability,
    );
    vault_access_rules.set_group_and_mutability(
        AccessRuleKey::Native(NativeFn::Vault(VaultFn::Take)),
        "withdraw".to_string(),
        DenyAll,
    );
    vault_access_rules.set_group_and_mutability(
        AccessRuleKey::Native(NativeFn::Vault(VaultFn::TakeNonFungibles)),
        "withdraw".to_string(),
        DenyAll,
    );
    vault_access_rules.set_group_and_mutability(
        AccessRuleKey::Native(NativeFn::Vault(VaultFn::LockFee)),
        "withdraw".to_string(),
        DenyAll,
    );

    vault_access_rules.set_access_rule_and_mutability(
        AccessRuleKey::Native(NativeFn::Vault(VaultFn::Put)),
        deposit_access_rule,
        deposit_mutability,
    );
    vault_access_rules.set_access_rule_and_mutability(
        AccessRuleKey::Native(NativeFn::Vault(VaultFn::GetAmount)),
        AllowAll,
        DenyAll,
    );
    vault_access_rules.set_access_rule_and_mutability(
        AccessRuleKey::Native(NativeFn::Vault(VaultFn::GetResourceAddress)),
        AllowAll,
        DenyAll,
    );
    vault_access_rules.set_access_rule_and_mutability(
        AccessRuleKey::Native(NativeFn::Vault(VaultFn::GetNonFungibleLocalIds)),
        AllowAll,
        DenyAll,
    );
    vault_access_rules.set_access_rule_and_mutability(
        AccessRuleKey::Native(NativeFn::Vault(VaultFn::CreateProof)),
        AllowAll,
        DenyAll,
    );
    vault_access_rules.set_access_rule_and_mutability(
        AccessRuleKey::Native(NativeFn::Vault(VaultFn::CreateProofByAmount)),
        AllowAll,
        DenyAll,
    );
    vault_access_rules.set_access_rule_and_mutability(
        AccessRuleKey::Native(NativeFn::Vault(VaultFn::CreateProofByIds)),
        AllowAll,
        DenyAll,
    );

    let vault_substate = AccessRulesChainSubstate {
        access_rules_chain: vec![vault_access_rules],
    };

    (substate, vault_substate)
}

fn create_non_fungible_resource_manager<Y>(
    global_node_id: RENodeId,
    id_type: NonFungibleIdType,
    metadata: BTreeMap<String, String>,
    access_rules: BTreeMap<ResourceMethodAuthKey, (AccessRule, AccessRule)>,
    api: &mut Y,
) -> Result<ResourceAddress, RuntimeError>
where
    Y: KernelNodeApi + KernelSubstateApi,
{
    let resource_address: ResourceAddress = global_node_id.into();

    let nf_store_node_id = api.kernel_allocate_node_id(RENodeType::NonFungibleStore)?;
    api.kernel_create_node(
        nf_store_node_id,
        RENodeInit::NonFungibleStore(NonFungibleStore::new()),
        BTreeMap::new(),
    )?;
    let nf_store_id: NonFungibleStoreId = nf_store_node_id.into();
    let resource_manager_substate = ResourceManagerSubstate::new(
        ResourceType::NonFungible { id_type },
        Some(nf_store_id),
        resource_address,
    );
    let (access_rules_substate, vault_substate) = build_substates(access_rules);
    let metadata_substate = MetadataSubstate { metadata };

    let mut node_modules = BTreeMap::new();
    node_modules.insert(
        NodeModuleId::Metadata,
        RENodeModuleInit::Metadata(metadata_substate),
    );
    node_modules.insert(
        NodeModuleId::AccessRules,
        RENodeModuleInit::AccessRulesChain(access_rules_substate),
    );
    node_modules.insert(
        NodeModuleId::AccessRules1,
        RENodeModuleInit::AccessRulesChain(vault_substate),
    );

    let underlying_node_id = api.kernel_allocate_node_id(RENodeType::ResourceManager)?;
    api.kernel_create_node(
        underlying_node_id,
        RENodeInit::ResourceManager(resource_manager_substate),
        node_modules,
    )?;
    api.kernel_create_node(
        global_node_id,
        RENodeInit::Global(GlobalAddressSubstate::Resource(underlying_node_id.into())),
        BTreeMap::new(),
    )?;

    Ok(resource_address)
}

pub struct ResourceManagerNativePackage;

impl ResourceManagerNativePackage {
    pub fn invoke_export<Y>(
        export_name: &str,
        input: ScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi
            + KernelSubstateApi
            + ClientSubstateApi<RuntimeError>
            + ClientApi<RuntimeError>
            + ClientNativeInvokeApi<RuntimeError>,
    {
        match export_name {
            RESOURCE_MANAGER_CREATE_NON_FUNGIBLE_IDENT => Self::create_non_fungible(input, api),
            RESOURCE_MANAGER_CREATE_NON_FUNGIBLE_WITH_ADDRESS_IDENT => {
                Self::create_non_fungible_with_address(input, api)
            }
            RESOURCE_MANAGER_CREATE_NON_FUNGIBLE_WITH_INITIAL_SUPPLY_IDENT => {
                Self::create_non_fungible_with_initial_supply(input, api)
            }
            RESOURCE_MANAGER_CREATE_UUID_NON_FUNGIBLE_WITH_INITIAL_SUPPLY => {
                Self::create_uuid_non_fungible_with_initial_supply(input, api)
            }
            RESOURCE_MANAGER_CREATE_FUNGIBLE_IDENT => Self::create_fungible(input, api),
            RESOURCE_MANAGER_CREATE_FUNGIBLE_WITH_INITIAL_SUPPLY_IDENT => {
                Self::create_fungible_with_initial_supply(input, api)
            }
            RESOURCE_MANAGER_CREATE_FUNGIBLE_WITH_INITIAL_SUPPLY_AND_ADDRESS_IDENT => {
                Self::create_fungible_with_initial_supply_and_address(input, api)
            }
            _ => Err(RuntimeError::InterpreterError(
                InterpreterError::InvalidInvocation,
            )),
        }
    }

    fn create_non_fungible<Y>(
        input: ScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi
            + KernelSubstateApi
            + ClientSubstateApi<RuntimeError>
            + ClientApi<RuntimeError>
            + ClientNativeInvokeApi<RuntimeError>,
    {
        // TODO: Remove decode/encode mess
        let input: ResourceManagerCreateNonFungibleInput =
            scrypto_decode(&scrypto_encode(&input).unwrap())
                .map_err(|_| RuntimeError::InterpreterError(InterpreterError::InvalidInvocation))?;

        let global_node_id = api.kernel_allocate_node_id(RENodeType::GlobalResourceManager)?;
        let address = create_non_fungible_resource_manager(
            global_node_id,
            input.id_type,
            input.metadata,
            input.access_rules,
            api,
        )?;
        Ok(IndexedScryptoValue::from_typed(&address))
    }

    fn create_non_fungible_with_address<Y>(
        input: ScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi
            + KernelSubstateApi
            + ClientSubstateApi<RuntimeError>
            + ClientApi<RuntimeError>
            + ClientNativeInvokeApi<RuntimeError>,
    {
        // TODO: Remove decode/encode mess
        let input: ResourceManagerCreateNonFungibleWithAddressInput =
            scrypto_decode(&scrypto_encode(&input).unwrap())
                .map_err(|_| RuntimeError::InterpreterError(InterpreterError::InvalidInvocation))?;

        // If address isn't user frame allocated or pre_allocated then
        // using this node_id will fail on create_node below
        let global_node_id = RENodeId::Global(GlobalAddress::Resource(ResourceAddress::Normal(
            input.resource_address,
        )));
        let address = create_non_fungible_resource_manager(
            global_node_id,
            input.id_type,
            input.metadata,
            input.access_rules,
            api,
        )?;

        Ok(IndexedScryptoValue::from_typed(&address))
    }

    fn create_non_fungible_with_initial_supply<Y>(
        input: ScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi
            + KernelSubstateApi
            + ClientSubstateApi<RuntimeError>
            + ClientApi<RuntimeError>
            + ClientNativeInvokeApi<RuntimeError>,
    {
        // TODO: Remove decode/encode mess
        let input: ResourceManagerCreateNonFungibleWithInitialSupplyInput =
            scrypto_decode(&scrypto_encode(&input).unwrap())
                .map_err(|_| RuntimeError::InterpreterError(InterpreterError::InvalidInvocation))?;

        let global_node_id = api.kernel_allocate_node_id(RENodeType::GlobalResourceManager)?;
        let resource_address: ResourceAddress = global_node_id.into();

        // TODO: Do this check in a better way (e.g. via type check)
        if input.id_type == NonFungibleIdType::UUID {
            return Err(RuntimeError::ApplicationError(
                ApplicationError::ResourceManagerError(
                    ResourceManagerError::InvalidNonFungibleIdType,
                ),
            ));
        }

        let (resource_manager_substate, bucket) =
            build_non_fungible_resource_manager_substate_with_initial_supply(
                resource_address,
                input.id_type,
                input.entries,
                api,
            )?;
        let (access_rules_substate, vault_substate) = build_substates(input.access_rules);
        let metadata_substate = MetadataSubstate {
            metadata: input.metadata,
        };

        let mut node_modules = BTreeMap::new();
        node_modules.insert(
            NodeModuleId::Metadata,
            RENodeModuleInit::Metadata(metadata_substate),
        );
        node_modules.insert(
            NodeModuleId::AccessRules,
            RENodeModuleInit::AccessRulesChain(access_rules_substate),
        );
        node_modules.insert(
            NodeModuleId::AccessRules1,
            RENodeModuleInit::AccessRulesChain(vault_substate),
        );

        let underlying_node_id = api.kernel_allocate_node_id(RENodeType::ResourceManager)?;
        api.kernel_create_node(
            underlying_node_id,
            RENodeInit::ResourceManager(resource_manager_substate),
            node_modules,
        )?;

        api.kernel_create_node(
            global_node_id,
            RENodeInit::Global(GlobalAddressSubstate::Resource(underlying_node_id.into())),
            BTreeMap::new(),
        )?;

        Ok(IndexedScryptoValue::from_typed(&(resource_address, bucket)))
    }

    fn create_uuid_non_fungible_with_initial_supply<Y>(
        input: ScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi
            + KernelSubstateApi
            + ClientSubstateApi<RuntimeError>
            + ClientApi<RuntimeError>
            + ClientNativeInvokeApi<RuntimeError>,
    {
        // TODO: Remove decode/encode mess
        let input: ResourceManagerCreateUuidNonFungibleWithInitialSupplyInput =
            scrypto_decode(&scrypto_encode(&input).unwrap())
                .map_err(|_| RuntimeError::InterpreterError(InterpreterError::InvalidInvocation))?;

        let global_node_id = api.kernel_allocate_node_id(RENodeType::GlobalResourceManager)?;
        let resource_address: ResourceAddress = global_node_id.into();

        let mut entries = BTreeMap::new();
        for entry in input.entries {
            let uuid = Runtime::generate_uuid(api)?;
            entries.insert(NonFungibleLocalId::UUID(uuid), entry);
        }

        let (resource_manager_substate, bucket) =
            build_non_fungible_resource_manager_substate_with_initial_supply(
                resource_address,
                NonFungibleIdType::UUID,
                entries,
                api,
            )?;
        let (access_rules_substate, vault_substate) = build_substates(input.access_rules);
        let metadata_substate = MetadataSubstate {
            metadata: input.metadata,
        };

        let mut node_modules = BTreeMap::new();
        node_modules.insert(
            NodeModuleId::Metadata,
            RENodeModuleInit::Metadata(metadata_substate),
        );
        node_modules.insert(
            NodeModuleId::AccessRules,
            RENodeModuleInit::AccessRulesChain(access_rules_substate),
        );
        node_modules.insert(
            NodeModuleId::AccessRules1,
            RENodeModuleInit::AccessRulesChain(vault_substate),
        );

        let underlying_node_id = api.kernel_allocate_node_id(RENodeType::ResourceManager)?;
        api.kernel_create_node(
            underlying_node_id,
            RENodeInit::ResourceManager(resource_manager_substate),
            node_modules,
        )?;

        api.kernel_create_node(
            global_node_id,
            RENodeInit::Global(GlobalAddressSubstate::Resource(underlying_node_id.into())),
            BTreeMap::new(),
        )?;

        Ok(IndexedScryptoValue::from_typed(&(resource_address, bucket)))
    }

    fn create_fungible<Y>(
        input: ScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi
            + KernelSubstateApi
            + ClientSubstateApi<RuntimeError>
            + ClientApi<RuntimeError>
            + ClientNativeInvokeApi<RuntimeError>,
    {
        // TODO: Remove decode/encode mess
        let input: ResourceManagerCreateFungibleInput =
            scrypto_decode(&scrypto_encode(&input).unwrap())
                .map_err(|_| RuntimeError::InterpreterError(InterpreterError::InvalidInvocation))?;

        let global_node_id = api.kernel_allocate_node_id(RENodeType::GlobalResourceManager)?;
        let address = create_fungible_resource_manager(
            global_node_id,
            input.divisibility,
            input.metadata,
            input.access_rules,
            api,
        )?;
        Ok(IndexedScryptoValue::from_typed(&address))
    }

    fn create_fungible_with_initial_supply<Y>(
        input: ScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi
            + KernelSubstateApi
            + ClientSubstateApi<RuntimeError>
            + ClientNativeInvokeApi<RuntimeError>,
    {
        // TODO: Remove decode/encode mess
        let input: ResourceManagerCreateFungibleWithInitialSupplyInput =
            scrypto_decode(&scrypto_encode(&input).unwrap())
                .map_err(|_| RuntimeError::InterpreterError(InterpreterError::InvalidInvocation))?;

        let global_node_id = api.kernel_allocate_node_id(RENodeType::GlobalResourceManager)?;
        let resource_address: ResourceAddress = global_node_id.into();

        let (resource_manager_substate, bucket) =
            build_fungible_resource_manager_substate_with_initial_supply(
                resource_address,
                input.divisibility,
                input.initial_supply,
                api,
            )?;
        let (access_rules_substate, vault_substate) = build_substates(input.access_rules);
        let metadata_substate = MetadataSubstate {
            metadata: input.metadata,
        };

        let mut node_modules = BTreeMap::new();
        node_modules.insert(
            NodeModuleId::Metadata,
            RENodeModuleInit::Metadata(metadata_substate),
        );
        node_modules.insert(
            NodeModuleId::AccessRules,
            RENodeModuleInit::AccessRulesChain(access_rules_substate),
        );
        node_modules.insert(
            NodeModuleId::AccessRules1,
            RENodeModuleInit::AccessRulesChain(vault_substate),
        );

        let underlying_node_id = api.kernel_allocate_node_id(RENodeType::ResourceManager)?;
        api.kernel_create_node(
            underlying_node_id,
            RENodeInit::ResourceManager(resource_manager_substate),
            node_modules,
        )?;

        api.kernel_create_node(
            global_node_id,
            RENodeInit::Global(GlobalAddressSubstate::Resource(underlying_node_id.into())),
            BTreeMap::new(),
        )?;

        Ok(IndexedScryptoValue::from_typed(&(resource_address, bucket)))
    }

    fn create_fungible_with_initial_supply_and_address<Y>(
        input: ScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi
            + KernelSubstateApi
            + ClientSubstateApi<RuntimeError>
            + ClientApi<RuntimeError>
            + ClientNativeInvokeApi<RuntimeError>,
    {
        // TODO: Remove decode/encode mess
        let input: ResourceManagerCreateFungibleWithInitialSupplyAndAddressInput =
            scrypto_decode(&scrypto_encode(&input).unwrap())
                .map_err(|_| RuntimeError::InterpreterError(InterpreterError::InvalidInvocation))?;

        let global_node_id = RENodeId::Global(GlobalAddress::Resource(ResourceAddress::Normal(
            input.resource_address,
        )));
        let resource_address: ResourceAddress = global_node_id.into();

        let (resource_manager_substate, bucket) =
            build_fungible_resource_manager_substate_with_initial_supply(
                resource_address,
                input.divisibility,
                input.initial_supply,
                api,
            )?;
        let (access_rules_substate, vault_substate) = build_substates(input.access_rules);
        let metadata_substate = MetadataSubstate {
            metadata: input.metadata,
        };

        let mut node_modules = BTreeMap::new();
        node_modules.insert(
            NodeModuleId::Metadata,
            RENodeModuleInit::Metadata(metadata_substate),
        );
        node_modules.insert(
            NodeModuleId::AccessRules,
            RENodeModuleInit::AccessRulesChain(access_rules_substate),
        );
        node_modules.insert(
            NodeModuleId::AccessRules1,
            RENodeModuleInit::AccessRulesChain(vault_substate),
        );

        let underlying_node_id = api.kernel_allocate_node_id(RENodeType::ResourceManager)?;
        api.kernel_create_node(
            underlying_node_id,
            RENodeInit::ResourceManager(resource_manager_substate),
            node_modules,
        )?;

        api.kernel_create_node(
            global_node_id,
            RENodeInit::Global(GlobalAddressSubstate::Resource(underlying_node_id.into())),
            BTreeMap::new(),
        )?;

        Ok(IndexedScryptoValue::from_typed(&(resource_address, bucket)))
    }
}

fn create_fungible_resource_manager<Y>(
    global_node_id: RENodeId,
    divisibility: u8,
    metadata: BTreeMap<String, String>,
    access_rules: BTreeMap<ResourceMethodAuthKey, (AccessRule, AccessRule)>,
    api: &mut Y,
) -> Result<ResourceAddress, RuntimeError>
where
    Y: KernelNodeApi + KernelSubstateApi,
{
    let resource_address: ResourceAddress = global_node_id.into();

    let resource_manager_substate = ResourceManagerSubstate::new(
        ResourceType::Fungible { divisibility },
        None,
        resource_address,
    );
    let (access_rules_substate, vault_substate) = build_substates(access_rules);
    let metadata_substate = MetadataSubstate { metadata };

    let mut node_modules = BTreeMap::new();
    node_modules.insert(
        NodeModuleId::Metadata,
        RENodeModuleInit::Metadata(metadata_substate),
    );
    node_modules.insert(
        NodeModuleId::AccessRules,
        RENodeModuleInit::AccessRulesChain(access_rules_substate),
    );
    node_modules.insert(
        NodeModuleId::AccessRules1,
        RENodeModuleInit::AccessRulesChain(vault_substate),
    );

    let underlying_node_id = api.kernel_allocate_node_id(RENodeType::ResourceManager)?;
    api.kernel_create_node(
        underlying_node_id,
        RENodeInit::ResourceManager(resource_manager_substate),
        node_modules,
    )?;
    api.kernel_create_node(
        global_node_id,
        RENodeInit::Global(GlobalAddressSubstate::Resource(underlying_node_id.into())),
        BTreeMap::new(),
    )?;

    Ok(resource_address)
}

pub struct ResourceManagerBurnExecutable(RENodeId, Bucket);

impl ExecutableInvocation for ResourceManagerBurnInvocation {
    type Exec = ResourceManagerBurnExecutable;

    fn resolve<D: ClientDerefApi<RuntimeError>>(
        self,
        api: &mut D,
    ) -> Result<(ResolvedActor, CallFrameUpdate, Self::Exec), RuntimeError> {
        let mut call_frame_update = CallFrameUpdate::move_node(RENodeId::Bucket(self.bucket.0));
        let resolved_receiver = deref_and_update(
            RENodeId::Global(GlobalAddress::Resource(self.receiver)),
            &mut call_frame_update,
            api,
        )?;
        let actor = ResolvedActor::method(
            NativeFn::ResourceManager(ResourceManagerFn::Burn),
            resolved_receiver,
        );
        let executor = ResourceManagerBurnExecutable(resolved_receiver.receiver, self.bucket);
        Ok((actor, call_frame_update, executor))
    }
}

impl Executor for ResourceManagerBurnExecutable {
    type Output = ();

    fn execute<'a, Y, W: WasmEngine>(
        self,
        api: &mut Y,
    ) -> Result<((), CallFrameUpdate), RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi,
    {
        let offset = SubstateOffset::ResourceManager(ResourceManagerOffset::ResourceManager);
        let resman_handle =
            api.kernel_lock_substate(self.0, NodeModuleId::SELF, offset, LockFlags::MUTABLE)?;

        let bucket: BucketSubstate = api.kernel_drop_node(RENodeId::Bucket(self.1 .0))?.into();

        // Check if resource matches
        // TODO: Move this check into actor check
        {
            let substate_ref = api.kernel_get_substate_ref(resman_handle)?;
            let resource_manager = substate_ref.resource_manager();
            if bucket.resource_address() != resource_manager.resource_address {
                return Err(RuntimeError::ApplicationError(
                    ApplicationError::ResourceManagerError(
                        ResourceManagerError::MismatchingBucketResource,
                    ),
                ));
            }
        }
        // Update total supply
        // TODO: there might be better for maintaining total supply, especially for non-fungibles
        // where we can leverage capabilities of key-value map.

        // Update total supply
        {
            let mut substate_mut = api.kernel_get_substate_ref_mut(resman_handle)?;
            let resource_manager = substate_mut.resource_manager();
            resource_manager.total_supply -= bucket.total_amount();
        }

        // Burn non-fungible
        let substate_ref = api.kernel_get_substate_ref(resman_handle)?;
        let resource_manager = substate_ref.resource_manager();
        if let Some(nf_store_id) = resource_manager.nf_store_id {
            let node_id = RENodeId::NonFungibleStore(nf_store_id);

            for id in bucket
                .total_ids()
                .expect("Failed to list non-fungible IDs on non-fungible Bucket")
            {
                let offset = SubstateOffset::NonFungibleStore(NonFungibleStoreOffset::Entry(id));
                let non_fungible_handle = api.kernel_lock_substate(
                    node_id,
                    NodeModuleId::SELF,
                    offset,
                    LockFlags::MUTABLE,
                )?;
                let mut substate_mut = api.kernel_get_substate_ref_mut(non_fungible_handle)?;
                let non_fungible_mut = substate_mut.non_fungible();

                *non_fungible_mut = NonFungibleSubstate(None);
                api.kernel_drop_lock(non_fungible_handle)?;
            }
        }

        Ok(((), CallFrameUpdate::empty()))
    }
}

pub struct ResourceManagerUpdateVaultAuthExecutable(RENodeId, VaultMethodAuthKey, AccessRule);

impl ExecutableInvocation for ResourceManagerUpdateVaultAuthInvocation {
    type Exec = ResourceManagerUpdateVaultAuthExecutable;

    fn resolve<D: ClientDerefApi<RuntimeError>>(
        self,
        api: &mut D,
    ) -> Result<(ResolvedActor, CallFrameUpdate, Self::Exec), RuntimeError> {
        let mut call_frame_update = CallFrameUpdate::empty();
        let resolved_receiver = deref_and_update(
            RENodeId::Global(GlobalAddress::Resource(self.receiver)),
            &mut call_frame_update,
            api,
        )?;
        let actor = ResolvedActor::method(
            NativeFn::ResourceManager(ResourceManagerFn::UpdateVaultAuth),
            resolved_receiver,
        );
        let executor = ResourceManagerUpdateVaultAuthExecutable(
            resolved_receiver.receiver,
            self.method,
            self.access_rule,
        );
        Ok((actor, call_frame_update, executor))
    }
}

// TODO: Figure out better place to do vault auth (or child node authorization)
impl Executor for ResourceManagerUpdateVaultAuthExecutable {
    type Output = ();

    fn execute<'a, Y, W: WasmEngine>(
        self,
        api: &mut Y,
    ) -> Result<((), CallFrameUpdate), RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientNativeInvokeApi<RuntimeError>,
    {
        let handle = api.kernel_lock_substate(
            self.0,
            NodeModuleId::AccessRules1,
            SubstateOffset::AccessRulesChain(AccessRulesChainOffset::AccessRulesChain),
            LockFlags::MUTABLE,
        )?;

        // TODO: Figure out how to move this access check into more appropriate place
        {
            let substate_ref = api.kernel_get_substate_ref(handle)?;
            let substate = substate_ref.access_rules_chain();

            let access_rule = match self.1 {
                Deposit => {
                    let key = AccessRuleKey::Native(NativeFn::Vault(VaultFn::Put));
                    substate.access_rules_chain[0].get_mutability(&key)
                }
                Withdraw => substate.access_rules_chain[0].get_group_mutability("withdraw"),
                Recall => substate.access_rules_chain[0].get_group_mutability("recall"),
            }
            .clone();

            api.call_native(AuthZoneAssertAccessRuleInvocation {
                receiver: RENodeId::AuthZoneStack.into(),
                access_rule,
            })?;
        }

        let mut substate_mut = api.kernel_get_substate_ref_mut(handle)?;
        let substate = substate_mut.access_rules_chain();

        match self.1 {
            VaultMethodAuthKey::Deposit => {
                let key = AccessRuleKey::Native(NativeFn::Vault(VaultFn::Put));
                substate.access_rules_chain[0].set_method_access_rule(key, self.2);
            }
            VaultMethodAuthKey::Withdraw => {
                let group_key = "withdraw".to_string();
                substate.access_rules_chain[0].set_group_access_rule(group_key, self.2);
            }
            VaultMethodAuthKey::Recall => {
                let group_key = "recall".to_string();
                substate.access_rules_chain[0].set_group_access_rule(group_key, self.2);
            }
        }

        Ok(((), CallFrameUpdate::empty()))
    }
}

impl ExecutableInvocation for ResourceManagerSetVaultAuthMutabilityInvocation {
    type Exec = ResourceManagerLockVaultAuthExecutable;

    fn resolve<D: ClientDerefApi<RuntimeError>>(
        self,
        api: &mut D,
    ) -> Result<(ResolvedActor, CallFrameUpdate, Self::Exec), RuntimeError> {
        let mut call_frame_update = CallFrameUpdate::empty();
        let resolved_receiver = deref_and_update(
            RENodeId::Global(GlobalAddress::Resource(self.receiver)),
            &mut call_frame_update,
            api,
        )?;
        let actor = ResolvedActor::method(
            NativeFn::ResourceManager(ResourceManagerFn::SetVaultAuthMutability),
            resolved_receiver,
        );
        let executor = ResourceManagerLockVaultAuthExecutable(
            resolved_receiver.receiver,
            self.method,
            self.mutability,
        );
        Ok((actor, call_frame_update, executor))
    }
}

pub struct ResourceManagerLockVaultAuthExecutable(RENodeId, VaultMethodAuthKey, AccessRule);

impl Executor for ResourceManagerLockVaultAuthExecutable {
    type Output = ();

    fn execute<'a, Y, W: WasmEngine>(
        self,
        api: &mut Y,
    ) -> Result<((), CallFrameUpdate), RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientNativeInvokeApi<RuntimeError>,
    {
        let handle = api.kernel_lock_substate(
            self.0,
            NodeModuleId::AccessRules1,
            SubstateOffset::AccessRulesChain(AccessRulesChainOffset::AccessRulesChain),
            LockFlags::MUTABLE,
        )?;

        // TODO: Figure out how to move this access check into more appropriate place
        {
            let substate_ref = api.kernel_get_substate_ref(handle)?;
            let substate = substate_ref.access_rules_chain();

            let access_rule = match self.1 {
                Deposit => {
                    let key = AccessRuleKey::Native(NativeFn::Vault(VaultFn::Put));
                    substate.access_rules_chain[0].get_mutability(&key)
                }
                Withdraw => substate.access_rules_chain[0].get_group_mutability("withdraw"),
                Recall => substate.access_rules_chain[0].get_group_mutability("recall"),
            }
            .clone();

            api.call_native(AuthZoneAssertAccessRuleInvocation {
                receiver: RENodeId::AuthZoneStack.into(),
                access_rule,
            })?;
        }

        let mut substate_mut = api.kernel_get_substate_ref_mut(handle)?;
        let substate = substate_mut.access_rules_chain();

        match self.1 {
            Deposit => {
                let key = AccessRuleKey::Native(NativeFn::Vault(VaultFn::Put));
                substate.access_rules_chain[0].set_mutability(key, self.2);
            }
            Withdraw => {
                let group_key = "withdraw".to_string();
                substate.access_rules_chain[0].set_group_mutability(group_key, self.2);
            }
            Recall => {
                let group_key = "recall".to_string();
                substate.access_rules_chain[0].set_group_mutability(group_key, self.2);
            }
        }

        Ok(((), CallFrameUpdate::empty()))
    }
}

impl ExecutableInvocation for ResourceManagerCreateVaultInvocation {
    type Exec = ResourceManagerCreateVaultExecutable;

    fn resolve<D: ClientDerefApi<RuntimeError>>(
        self,
        api: &mut D,
    ) -> Result<(ResolvedActor, CallFrameUpdate, Self::Exec), RuntimeError> {
        let mut call_frame_update = CallFrameUpdate::empty();
        let resolved_receiver = deref_and_update(
            RENodeId::Global(GlobalAddress::Resource(self.receiver)),
            &mut call_frame_update,
            api,
        )?;
        let actor = ResolvedActor::method(
            NativeFn::ResourceManager(ResourceManagerFn::CreateVault),
            resolved_receiver,
        );
        let executor = ResourceManagerCreateVaultExecutable(resolved_receiver.receiver);
        Ok((actor, call_frame_update, executor))
    }
}

pub struct ResourceManagerCreateVaultExecutable(RENodeId);

impl Executor for ResourceManagerCreateVaultExecutable {
    type Output = Own;

    fn execute<'a, Y, W: WasmEngine>(
        self,
        api: &mut Y,
    ) -> Result<(Own, CallFrameUpdate), RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi,
    {
        let offset = SubstateOffset::ResourceManager(ResourceManagerOffset::ResourceManager);
        let resman_handle =
            api.kernel_lock_substate(self.0, NodeModuleId::SELF, offset, LockFlags::MUTABLE)?;

        let substate_ref = api.kernel_get_substate_ref(resman_handle)?;
        let resource_manager = substate_ref.resource_manager();
        let resource = Resource::new_empty(
            resource_manager.resource_address,
            resource_manager.resource_type,
        );

        let node_id = api.kernel_allocate_node_id(RENodeType::Vault)?;
        api.kernel_create_node(
            node_id,
            RENodeInit::Vault(VaultRuntimeSubstate::new(resource)),
            BTreeMap::new(),
        )?;
        let vault_id = node_id.into();

        Ok((
            Own::Vault(vault_id),
            CallFrameUpdate::move_node(RENodeId::Vault(vault_id)),
        ))
    }
}

impl ExecutableInvocation for ResourceManagerCreateBucketInvocation {
    type Exec = ResourceManagerCreateBucketExecutable;

    fn resolve<D: ClientDerefApi<RuntimeError>>(
        self,
        api: &mut D,
    ) -> Result<(ResolvedActor, CallFrameUpdate, Self::Exec), RuntimeError> {
        let mut call_frame_update = CallFrameUpdate::empty();
        let resolved_receiver = deref_and_update(
            RENodeId::Global(GlobalAddress::Resource(self.receiver)),
            &mut call_frame_update,
            api,
        )?;
        let actor = ResolvedActor::method(
            NativeFn::ResourceManager(ResourceManagerFn::CreateBucket),
            resolved_receiver,
        );
        let executor = ResourceManagerCreateBucketExecutable(resolved_receiver.receiver);
        Ok((actor, call_frame_update, executor))
    }
}

pub struct ResourceManagerCreateBucketExecutable(RENodeId);

impl Executor for ResourceManagerCreateBucketExecutable {
    type Output = Bucket;

    fn execute<'a, Y, W: WasmEngine>(
        self,
        api: &mut Y,
    ) -> Result<(Bucket, CallFrameUpdate), RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi,
    {
        let offset = SubstateOffset::ResourceManager(ResourceManagerOffset::ResourceManager);
        let resman_handle =
            api.kernel_lock_substate(self.0, NodeModuleId::SELF, offset, LockFlags::MUTABLE)?;

        let substate_ref = api.kernel_get_substate_ref(resman_handle)?;
        let resource_manager = substate_ref.resource_manager();
        let container = Resource::new_empty(
            resource_manager.resource_address,
            resource_manager.resource_type,
        );

        let node_id = api.kernel_allocate_node_id(RENodeType::Bucket)?;
        api.kernel_create_node(
            node_id,
            RENodeInit::Bucket(BucketSubstate::new(container)),
            BTreeMap::new(),
        )?;
        let bucket_id = node_id.into();

        Ok((
            Bucket(bucket_id),
            CallFrameUpdate::move_node(RENodeId::Bucket(bucket_id)),
        ))
    }
}

pub struct ResourceManagerMintNonFungibleExecutable(
    RENodeId,
    BTreeMap<NonFungibleLocalId, (Vec<u8>, Vec<u8>)>,
);

impl ExecutableInvocation for ResourceManagerMintNonFungibleInvocation {
    type Exec = ResourceManagerMintNonFungibleExecutable;

    fn resolve<D: ClientDerefApi<RuntimeError>>(
        self,
        api: &mut D,
    ) -> Result<(ResolvedActor, CallFrameUpdate, Self::Exec), RuntimeError> {
        let mut call_frame_update = CallFrameUpdate::empty();
        let resolved_receiver = deref_and_update(
            RENodeId::Global(GlobalAddress::Resource(self.receiver)),
            &mut call_frame_update,
            api,
        )?;
        let actor = ResolvedActor::method(
            NativeFn::ResourceManager(ResourceManagerFn::MintNonFungible),
            resolved_receiver,
        );
        let executor =
            ResourceManagerMintNonFungibleExecutable(resolved_receiver.receiver, self.entries);
        Ok((actor, call_frame_update, executor))
    }
}

impl Executor for ResourceManagerMintNonFungibleExecutable {
    type Output = Bucket;

    fn execute<'a, Y, W: WasmEngine>(
        self,
        api: &mut Y,
    ) -> Result<(Bucket, CallFrameUpdate), RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi,
    {
        let offset = SubstateOffset::ResourceManager(ResourceManagerOffset::ResourceManager);
        let resman_handle =
            api.kernel_lock_substate(self.0, NodeModuleId::SELF, offset, LockFlags::MUTABLE)?;

        let (resource, non_fungibles) = {
            let mut substate_mut = api.kernel_get_substate_ref_mut(resman_handle)?;
            let resource_manager = substate_mut.resource_manager();

            let id_type = match resource_manager.resource_type {
                ResourceType::NonFungible { id_type } => id_type,
                _ => {
                    return Err(RuntimeError::ApplicationError(
                        ApplicationError::ResourceManagerError(
                            ResourceManagerError::ResourceTypeDoesNotMatch,
                        ),
                    ))
                }
            };

            if id_type == NonFungibleIdType::UUID {
                return Err(RuntimeError::ApplicationError(
                    ApplicationError::ResourceManagerError(
                        ResourceManagerError::InvalidNonFungibleIdType,
                    ),
                ));
            }

            let amount: Decimal = self.1.len().into();
            resource_manager.total_supply += amount;
            // Allocate non-fungibles
            let mut ids = BTreeSet::new();
            let mut non_fungibles = BTreeMap::new();
            for (id, data) in self.1 {
                if id.id_type() != id_type {
                    return Err(RuntimeError::ApplicationError(
                        ApplicationError::ResourceManagerError(
                            ResourceManagerError::NonFungibleIdTypeDoesNotMatch(
                                id.id_type(),
                                id_type,
                            ),
                        ),
                    ));
                }

                let non_fungible = NonFungible::new(data.0, data.1);
                ids.insert(id.clone());
                non_fungibles.insert(id, non_fungible);
            }

            (
                Resource::new_non_fungible(resource_manager.resource_address, ids, id_type),
                non_fungibles,
            )
        };

        let node_id = api.kernel_allocate_node_id(RENodeType::Bucket)?;
        api.kernel_create_node(
            node_id,
            RENodeInit::Bucket(BucketSubstate::new(resource)),
            BTreeMap::new(),
        )?;
        let bucket_id = node_id.into();

        let (nf_store_id, resource_address) = {
            let substate_ref = api.kernel_get_substate_ref(resman_handle)?;
            let resource_manager = substate_ref.resource_manager();
            (
                resource_manager.nf_store_id.clone(),
                resource_manager.resource_address,
            )
        };

        for (id, non_fungible) in non_fungibles {
            let node_id = RENodeId::NonFungibleStore(nf_store_id.unwrap());
            let offset =
                SubstateOffset::NonFungibleStore(NonFungibleStoreOffset::Entry(id.clone()));
            let non_fungible_handle =
                api.kernel_lock_substate(node_id, NodeModuleId::SELF, offset, LockFlags::MUTABLE)?;

            {
                let mut substate_mut = api.kernel_get_substate_ref_mut(non_fungible_handle)?;
                let non_fungible_mut = substate_mut.non_fungible();

                if non_fungible_mut.0.is_some() {
                    return Err(RuntimeError::ApplicationError(
                        ApplicationError::ResourceManagerError(
                            ResourceManagerError::NonFungibleAlreadyExists(
                                NonFungibleGlobalId::new(resource_address, id),
                            ),
                        ),
                    ));
                }

                *non_fungible_mut = NonFungibleSubstate(Some(non_fungible));
            }

            api.kernel_drop_lock(non_fungible_handle)?;
        }

        Ok((
            Bucket(bucket_id),
            CallFrameUpdate::move_node(RENodeId::Bucket(bucket_id)),
        ))
    }
}

pub struct ResourceManagerMintUuidNonFungibleExecutable(RENodeId, Vec<(Vec<u8>, Vec<u8>)>);

impl ExecutableInvocation for ResourceManagerMintUuidNonFungibleInvocation {
    type Exec = ResourceManagerMintUuidNonFungibleExecutable;

    fn resolve<D: ClientDerefApi<RuntimeError>>(
        self,
        api: &mut D,
    ) -> Result<(ResolvedActor, CallFrameUpdate, Self::Exec), RuntimeError> {
        let mut call_frame_update = CallFrameUpdate::empty();
        let resolved_receiver = deref_and_update(
            RENodeId::Global(GlobalAddress::Resource(self.receiver)),
            &mut call_frame_update,
            api,
        )?;
        let actor = ResolvedActor::method(
            NativeFn::ResourceManager(ResourceManagerFn::MintUuidNonFungible),
            resolved_receiver,
        );
        let executor =
            ResourceManagerMintUuidNonFungibleExecutable(resolved_receiver.receiver, self.entries);
        Ok((actor, call_frame_update, executor))
    }
}

impl Executor for ResourceManagerMintUuidNonFungibleExecutable {
    type Output = Bucket;

    fn execute<Y, W: WasmEngine>(
        self,
        api: &mut Y,
    ) -> Result<(Bucket, CallFrameUpdate), RuntimeError>
    where
        Y: KernelNodeApi
            + KernelSubstateApi
            + ClientNodeApi<RuntimeError>
            + ClientSubstateApi<RuntimeError>
            + ClientNativeInvokeApi<RuntimeError>,
    {
        let offset = SubstateOffset::ResourceManager(ResourceManagerOffset::ResourceManager);
        let resman_handle =
            api.kernel_lock_substate(self.0, NodeModuleId::SELF, offset, LockFlags::MUTABLE)?;

        let bucket_id = {
            let mut substate_mut = api.kernel_get_substate_ref_mut(resman_handle)?;
            let resource_manager = substate_mut.resource_manager();
            let resource_address = resource_manager.resource_address;
            let id_type = match resource_manager.resource_type {
                ResourceType::NonFungible { id_type } => id_type,
                _ => {
                    return Err(RuntimeError::ApplicationError(
                        ApplicationError::ResourceManagerError(
                            ResourceManagerError::ResourceTypeDoesNotMatch,
                        ),
                    ))
                }
            };
            let nf_store_id = resource_manager.nf_store_id.unwrap();

            if id_type != NonFungibleIdType::UUID {
                return Err(RuntimeError::ApplicationError(
                    ApplicationError::ResourceManagerError(
                        ResourceManagerError::InvalidNonFungibleIdType,
                    ),
                ));
            }

            let amount: Decimal = self.1.len().into();
            resource_manager.total_supply += amount;
            // Allocate non-fungibles
            let mut ids = BTreeSet::new();
            for data in self.1 {
                // TODO: Is this enough bits to prevent hash collisions?
                // TODO: Possibly use an always incrementing timestamp
                let uuid = Runtime::generate_uuid(api)?;
                let id = NonFungibleLocalId::UUID(uuid);
                ids.insert(id.clone());

                {
                    let node_id = RENodeId::NonFungibleStore(nf_store_id);
                    let offset =
                        SubstateOffset::NonFungibleStore(NonFungibleStoreOffset::Entry(id));
                    let non_fungible_handle = api.kernel_lock_substate(
                        node_id,
                        NodeModuleId::SELF,
                        offset,
                        LockFlags::MUTABLE,
                    )?;
                    let non_fungible = NonFungible::new(data.0, data.1);
                    let mut substate_mut = api.kernel_get_substate_ref_mut(non_fungible_handle)?;
                    let non_fungible_mut = substate_mut.non_fungible();
                    *non_fungible_mut = NonFungibleSubstate(Some(non_fungible));
                    api.kernel_drop_lock(non_fungible_handle)?;
                }
            }

            let node_id = api.kernel_allocate_node_id(RENodeType::Bucket)?;
            api.kernel_create_node(
                node_id,
                RENodeInit::Bucket(BucketSubstate::new(Resource::new_non_fungible(
                    resource_address,
                    ids,
                    id_type,
                ))),
                BTreeMap::new(),
            )?;
            let bucket_id: BucketId = node_id.into();
            bucket_id
        };

        Ok((
            Bucket(bucket_id),
            CallFrameUpdate::move_node(RENodeId::Bucket(bucket_id)),
        ))
    }
}

pub struct ResourceManagerMintFungibleExecutable(RENodeId, Decimal);

impl ExecutableInvocation for ResourceManagerMintFungibleInvocation {
    type Exec = ResourceManagerMintFungibleExecutable;

    fn resolve<D: ClientDerefApi<RuntimeError>>(
        self,
        api: &mut D,
    ) -> Result<(ResolvedActor, CallFrameUpdate, Self::Exec), RuntimeError> {
        let mut call_frame_update = CallFrameUpdate::empty();
        let resolved_receiver = deref_and_update(
            RENodeId::Global(GlobalAddress::Resource(self.receiver)),
            &mut call_frame_update,
            api,
        )?;
        let actor = ResolvedActor::method(
            NativeFn::ResourceManager(ResourceManagerFn::MintFungible),
            resolved_receiver,
        );
        let executor =
            ResourceManagerMintFungibleExecutable(resolved_receiver.receiver, self.amount);
        Ok((actor, call_frame_update, executor))
    }
}

impl Executor for ResourceManagerMintFungibleExecutable {
    type Output = Bucket;

    fn execute<'a, Y, W: WasmEngine>(
        self,
        api: &mut Y,
    ) -> Result<(Bucket, CallFrameUpdate), RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi,
    {
        let offset = SubstateOffset::ResourceManager(ResourceManagerOffset::ResourceManager);
        let resman_handle =
            api.kernel_lock_substate(self.0, NodeModuleId::SELF, offset, LockFlags::MUTABLE)?;

        let resource = {
            let mut substate_mut = api.kernel_get_substate_ref_mut(resman_handle)?;
            let resource_manager = substate_mut.resource_manager();
            let result =
                resource_manager.mint_fungible(self.1, resource_manager.resource_address)?;
            result
        };

        let node_id = api.kernel_allocate_node_id(RENodeType::Bucket)?;
        api.kernel_create_node(
            node_id,
            RENodeInit::Bucket(BucketSubstate::new(resource)),
            BTreeMap::new(),
        )?;
        let bucket_id = node_id.into();

        Ok((
            Bucket(bucket_id),
            CallFrameUpdate::move_node(RENodeId::Bucket(bucket_id)),
        ))
    }
}

impl ExecutableInvocation for ResourceManagerGetResourceTypeInvocation {
    type Exec = ResourceManagerGetResourceTypeExecutable;

    fn resolve<D: ClientDerefApi<RuntimeError>>(
        self,
        api: &mut D,
    ) -> Result<(ResolvedActor, CallFrameUpdate, Self::Exec), RuntimeError> {
        let mut call_frame_update = CallFrameUpdate::empty();
        let resolved_receiver = deref_and_update(
            RENodeId::Global(GlobalAddress::Resource(self.receiver)),
            &mut call_frame_update,
            api,
        )?;
        let actor = ResolvedActor::method(
            NativeFn::ResourceManager(ResourceManagerFn::GetResourceType),
            resolved_receiver,
        );
        let executor = ResourceManagerGetResourceTypeExecutable(resolved_receiver.receiver);
        Ok((actor, call_frame_update, executor))
    }
}

pub struct ResourceManagerGetResourceTypeExecutable(RENodeId);

impl Executor for ResourceManagerGetResourceTypeExecutable {
    type Output = ResourceType;

    fn execute<'a, Y, W: WasmEngine>(
        self,
        api: &mut Y,
    ) -> Result<(ResourceType, CallFrameUpdate), RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi,
    {
        let offset = SubstateOffset::ResourceManager(ResourceManagerOffset::ResourceManager);
        let resman_handle =
            api.kernel_lock_substate(self.0, NodeModuleId::SELF, offset, LockFlags::read_only())?;

        let substate_ref = api.kernel_get_substate_ref(resman_handle)?;
        let resource_type = substate_ref.resource_manager().resource_type;

        Ok((resource_type, CallFrameUpdate::empty()))
    }
}

impl ExecutableInvocation for ResourceManagerGetTotalSupplyInvocation {
    type Exec = ResourceManagerGetTotalSupplyExecutable;

    fn resolve<D: ClientDerefApi<RuntimeError>>(
        self,
        api: &mut D,
    ) -> Result<(ResolvedActor, CallFrameUpdate, Self::Exec), RuntimeError> {
        let mut call_frame_update = CallFrameUpdate::empty();
        let resolved_receiver = deref_and_update(
            RENodeId::Global(GlobalAddress::Resource(self.receiver)),
            &mut call_frame_update,
            api,
        )?;
        let actor = ResolvedActor::method(
            NativeFn::ResourceManager(ResourceManagerFn::GetTotalSupply),
            resolved_receiver,
        );
        let executor = ResourceManagerGetTotalSupplyExecutable(resolved_receiver.receiver);
        Ok((actor, call_frame_update, executor))
    }
}

pub struct ResourceManagerGetTotalSupplyExecutable(RENodeId);

impl Executor for ResourceManagerGetTotalSupplyExecutable {
    type Output = Decimal;

    fn execute<'a, Y, W: WasmEngine>(
        self,
        api: &mut Y,
    ) -> Result<(Decimal, CallFrameUpdate), RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi,
    {
        let offset = SubstateOffset::ResourceManager(ResourceManagerOffset::ResourceManager);
        let resman_handle =
            api.kernel_lock_substate(self.0, NodeModuleId::SELF, offset, LockFlags::read_only())?;
        let substate_ref = api.kernel_get_substate_ref(resman_handle)?;
        let total_supply = substate_ref.resource_manager().total_supply;

        Ok((total_supply, CallFrameUpdate::empty()))
    }
}

impl ExecutableInvocation for ResourceManagerUpdateNonFungibleDataInvocation {
    type Exec = ResourceManagerUpdateNonFungibleDataExecutable;

    fn resolve<D: ClientDerefApi<RuntimeError>>(
        self,
        api: &mut D,
    ) -> Result<(ResolvedActor, CallFrameUpdate, Self::Exec), RuntimeError> {
        let mut call_frame_update = CallFrameUpdate::empty();
        let resolved_receiver = deref_and_update(
            RENodeId::Global(GlobalAddress::Resource(self.receiver)),
            &mut call_frame_update,
            api,
        )?;
        let actor = ResolvedActor::method(
            NativeFn::ResourceManager(ResourceManagerFn::UpdateNonFungibleData),
            resolved_receiver,
        );
        let executor = ResourceManagerUpdateNonFungibleDataExecutable(
            resolved_receiver.receiver,
            self.id,
            self.data,
        );
        Ok((actor, call_frame_update, executor))
    }
}

pub struct ResourceManagerUpdateNonFungibleDataExecutable(RENodeId, NonFungibleLocalId, Vec<u8>);

impl Executor for ResourceManagerUpdateNonFungibleDataExecutable {
    type Output = ();

    fn execute<'a, Y, W: WasmEngine>(
        self,
        api: &mut Y,
    ) -> Result<((), CallFrameUpdate), RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi,
    {
        let offset = SubstateOffset::ResourceManager(ResourceManagerOffset::ResourceManager);
        let resman_handle =
            api.kernel_lock_substate(self.0, NodeModuleId::SELF, offset, LockFlags::MUTABLE)?;

        let substate_ref = api.kernel_get_substate_ref(resman_handle)?;
        let resource_manager = substate_ref.resource_manager();
        let nf_store_id = resource_manager
            .nf_store_id
            .ok_or(InvokeError::SelfError(ResourceManagerError::NotNonFungible))?;
        let resource_address = resource_manager.resource_address;

        let node_id = RENodeId::NonFungibleStore(nf_store_id);
        let offset =
            SubstateOffset::NonFungibleStore(NonFungibleStoreOffset::Entry(self.1.clone()));

        let non_fungible_handle =
            api.kernel_lock_substate(node_id, NodeModuleId::SELF, offset, LockFlags::MUTABLE)?;
        let mut substate_mut = api.kernel_get_substate_ref_mut(non_fungible_handle)?;
        let non_fungible_mut = substate_mut.non_fungible();
        if let Some(ref mut non_fungible) = non_fungible_mut.0 {
            non_fungible.set_mutable_data(self.2);
        } else {
            let non_fungible_global_id = NonFungibleGlobalId::new(resource_address, self.1);
            return Err(RuntimeError::ApplicationError(
                ApplicationError::ResourceManagerError(ResourceManagerError::NonFungibleNotFound(
                    non_fungible_global_id,
                )),
            ));
        }

        api.kernel_drop_lock(non_fungible_handle)?;

        Ok(((), CallFrameUpdate::empty()))
    }
}

impl ExecutableInvocation for ResourceManagerNonFungibleExistsInvocation {
    type Exec = ResourceManagerNonFungibleExistsExecutable;

    fn resolve<D: ClientDerefApi<RuntimeError>>(
        self,
        api: &mut D,
    ) -> Result<(ResolvedActor, CallFrameUpdate, Self::Exec), RuntimeError> {
        let mut call_frame_update = CallFrameUpdate::empty();
        let resolved_receiver = deref_and_update(
            RENodeId::Global(GlobalAddress::Resource(self.receiver)),
            &mut call_frame_update,
            api,
        )?;
        let actor = ResolvedActor::method(
            NativeFn::ResourceManager(ResourceManagerFn::NonFungibleExists),
            resolved_receiver,
        );
        let executor =
            ResourceManagerNonFungibleExistsExecutable(resolved_receiver.receiver, self.id);
        Ok((actor, call_frame_update, executor))
    }
}

pub struct ResourceManagerNonFungibleExistsExecutable(RENodeId, NonFungibleLocalId);

impl Executor for ResourceManagerNonFungibleExistsExecutable {
    type Output = bool;

    fn execute<'a, Y, W: WasmEngine>(
        self,
        api: &mut Y,
    ) -> Result<(bool, CallFrameUpdate), RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi,
    {
        let offset = SubstateOffset::ResourceManager(ResourceManagerOffset::ResourceManager);
        let resman_handle =
            api.kernel_lock_substate(self.0, NodeModuleId::SELF, offset, LockFlags::read_only())?;

        let substate_ref = api.kernel_get_substate_ref(resman_handle)?;
        let resource_manager = substate_ref.resource_manager();
        let nf_store_id = resource_manager
            .nf_store_id
            .ok_or(InvokeError::SelfError(ResourceManagerError::NotNonFungible))?;

        let node_id = RENodeId::NonFungibleStore(nf_store_id);
        let offset = SubstateOffset::NonFungibleStore(NonFungibleStoreOffset::Entry(self.1));
        let non_fungible_handle =
            api.kernel_lock_substate(node_id, NodeModuleId::SELF, offset, LockFlags::read_only())?;
        let substate = api.kernel_get_substate_ref(non_fungible_handle)?;
        let exists = substate.non_fungible().0.is_some();

        Ok((exists, CallFrameUpdate::empty()))
    }
}

impl ExecutableInvocation for ResourceManagerGetNonFungibleInvocation {
    type Exec = ResourceManagerGetNonFungibleExecutable;

    fn resolve<D: ClientDerefApi<RuntimeError>>(
        self,
        api: &mut D,
    ) -> Result<(ResolvedActor, CallFrameUpdate, Self::Exec), RuntimeError> {
        let mut call_frame_update = CallFrameUpdate::empty();
        let resolved_receiver = deref_and_update(
            RENodeId::Global(GlobalAddress::Resource(self.receiver)),
            &mut call_frame_update,
            api,
        )?;
        let actor = ResolvedActor::method(
            NativeFn::ResourceManager(ResourceManagerFn::GetNonFungible),
            resolved_receiver,
        );
        let executor = ResourceManagerGetNonFungibleExecutable(resolved_receiver.receiver, self.id);
        Ok((actor, call_frame_update, executor))
    }
}

pub struct ResourceManagerGetNonFungibleExecutable(RENodeId, NonFungibleLocalId);

impl Executor for ResourceManagerGetNonFungibleExecutable {
    type Output = [Vec<u8>; 2];

    fn execute<Y, W: WasmEngine>(
        self,
        api: &mut Y,
    ) -> Result<([Vec<u8>; 2], CallFrameUpdate), RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi,
    {
        let offset = SubstateOffset::ResourceManager(ResourceManagerOffset::ResourceManager);
        let resman_handle =
            api.kernel_lock_substate(self.0, NodeModuleId::SELF, offset, LockFlags::read_only())?;

        let substate_ref = api.kernel_get_substate_ref(resman_handle)?;
        let resource_manager = substate_ref.resource_manager();
        let nf_store_id = resource_manager
            .nf_store_id
            .ok_or(InvokeError::SelfError(ResourceManagerError::NotNonFungible))?;

        let non_fungible_global_id =
            NonFungibleGlobalId::new(resource_manager.resource_address, self.1.clone());

        let node_id = RENodeId::NonFungibleStore(nf_store_id);
        let offset = SubstateOffset::NonFungibleStore(NonFungibleStoreOffset::Entry(self.1));
        let non_fungible_handle =
            api.kernel_lock_substate(node_id, NodeModuleId::SELF, offset, LockFlags::read_only())?;
        let non_fungible_ref = api.kernel_get_substate_ref(non_fungible_handle)?;
        let wrapper = non_fungible_ref.non_fungible();
        if let Some(non_fungible) = wrapper.0.as_ref() {
            Ok((
                [non_fungible.immutable_data(), non_fungible.mutable_data()],
                CallFrameUpdate::empty(),
            ))
        } else {
            return Err(RuntimeError::ApplicationError(
                ApplicationError::ResourceManagerError(ResourceManagerError::NonFungibleNotFound(
                    non_fungible_global_id,
                )),
            ));
        }
    }
}
