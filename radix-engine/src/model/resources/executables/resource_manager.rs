use crate::engine::{
    deref_and_update, ApplicationError, CallFrameUpdate, ExecutableInvocation, Executor, LockFlags,
    RENodeInit, ResolvedActor, ResolverApi, RuntimeError, SystemApi,
};
use crate::model::{
    AccessRulesChainSubstate, BucketSubstate, GlobalAddressSubstate, InvokeError, MetadataSubstate,
    NonFungible, NonFungibleSubstate, Resource, VaultRuntimeSubstate,
};
use crate::model::{NonFungibleStore, ResourceManagerSubstate};
use crate::types::*;
use crate::wasm::WasmEngine;
use native_sdk::resource::SysBucket;
use native_sdk::runtime::Runtime;
use radix_engine_interface::api::types::{
    GlobalAddress, NativeFn, NonFungibleStoreId, NonFungibleStoreOffset, RENodeId,
    ResourceManagerFn, ResourceManagerOffset, SubstateOffset,
};
use radix_engine_interface::api::{EngineApi, InvokableModel};
use radix_engine_interface::data::types::Own;
use radix_engine_interface::math::Decimal;
use radix_engine_interface::model::AccessRule::{AllowAll, DenyAll};
use radix_engine_interface::model::VaultMethodAuthKey::{Deposit, Recall, Withdraw};
use radix_engine_interface::model::*;
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

impl ExecutableInvocation for ResourceManagerBucketBurnInvocation {
    type Exec = Self;

    fn resolve<D: ResolverApi>(
        self,
        _api: &mut D,
    ) -> Result<(ResolvedActor, CallFrameUpdate, Self::Exec), RuntimeError> {
        let call_frame_update = CallFrameUpdate::move_node(RENodeId::Bucket(self.bucket.0));
        let actor =
            ResolvedActor::function(NativeFn::ResourceManager(ResourceManagerFn::BurnBucket));
        Ok((actor, call_frame_update, self))
    }
}

impl Executor for ResourceManagerBucketBurnInvocation {
    type Output = ();

    fn execute<Y, W: WasmEngine>(self, env: &mut Y) -> Result<((), CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi + InvokableModel<RuntimeError>,
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
    Y: SystemApi,
{
    let nf_store_node_id = api.allocate_node_id(RENodeType::NonFungibleStore)?;
    api.create_node(
        nf_store_node_id,
        RENodeInit::NonFungibleStore(NonFungibleStore::new()),
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
            let non_fungible_handle =
                api.lock_substate(nf_store_node_id, offset, LockFlags::MUTABLE)?;
            let mut substate_mut = api.get_ref_mut(non_fungible_handle)?;
            let non_fungible_mut = substate_mut.non_fungible();
            *non_fungible_mut = NonFungibleSubstate(Some(
                NonFungible::new(data.0.clone(), data.1.clone()), // FIXME: verify data
            ));
            api.drop_lock(non_fungible_handle)?;
        }
        resource_manager.total_supply = entries.len().into();
        let ids = entries.into_keys().collect();
        let container = Resource::new_non_fungible(resource_address, ids, id_type);
        let node_id = api.allocate_node_id(RENodeType::Bucket)?;
        api.create_node(node_id, RENodeInit::Bucket(BucketSubstate::new(container)))?;
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
    Y: SystemApi,
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
        let node_id = api.allocate_node_id(RENodeType::Bucket)?;
        api.create_node(node_id, RENodeInit::Bucket(BucketSubstate::new(container)))?;
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
        AccessRuleKey::Native(NativeFn::ResourceManager(ResourceManagerFn::LockAuth)),
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

impl ExecutableInvocation for ResourceManagerCreateNonFungibleInvocation {
    type Exec = Self;

    fn resolve<D: ResolverApi>(
        self,
        _api: &mut D,
    ) -> Result<(ResolvedActor, CallFrameUpdate, Self::Exec), RuntimeError> {
        let call_frame_update = CallFrameUpdate::empty();
        let actor = ResolvedActor::function(NativeFn::ResourceManager(
            ResourceManagerFn::CreateNonFungible,
        ));
        Ok((actor, call_frame_update, self))
    }
}

impl Executor for ResourceManagerCreateNonFungibleInvocation {
    type Output = ResourceAddress;

    fn execute<Y, W: WasmEngine>(
        self,
        api: &mut Y,
    ) -> Result<(ResourceAddress, CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi,
    {
        let global_node_id = if let Some(address) = self.resource_address {
            // If address isn't user frame allocated or pre_allocated then
            // using this node_id will fail on create_node below
            RENodeId::Global(GlobalAddress::Resource(ResourceAddress::Normal(address)))
        } else {
            api.allocate_node_id(RENodeType::GlobalResourceManager)?
        };
        let resource_address: ResourceAddress = global_node_id.into();

        let nf_store_node_id = api.allocate_node_id(RENodeType::NonFungibleStore)?;
        api.create_node(
            nf_store_node_id,
            RENodeInit::NonFungibleStore(NonFungibleStore::new()),
        )?;
        let nf_store_id: NonFungibleStoreId = nf_store_node_id.into();
        let resource_manager_substate = ResourceManagerSubstate::new(
            ResourceType::NonFungible {
                id_type: self.id_type,
            },
            Some(nf_store_id),
            resource_address,
        );
        let (substate, vault_substate) = build_substates(self.access_rules);
        let metadata_substate = MetadataSubstate {
            metadata: self.metadata,
        };

        let underlying_node_id = api.allocate_node_id(RENodeType::ResourceManager)?;
        api.create_node(
            underlying_node_id,
            RENodeInit::ResourceManager(
                resource_manager_substate,
                metadata_substate,
                substate,
                vault_substate,
            ),
        )?;
        api.create_node(
            global_node_id,
            RENodeInit::Global(GlobalAddressSubstate::Resource(underlying_node_id.into())),
        )?;

        let update =
            CallFrameUpdate::copy_ref(RENodeId::Global(GlobalAddress::Resource(resource_address)));

        Ok((resource_address, update))
    }
}

impl ExecutableInvocation for ResourceManagerCreateFungibleInvocation {
    type Exec = Self;

    fn resolve<D: ResolverApi>(
        self,
        _api: &mut D,
    ) -> Result<(ResolvedActor, CallFrameUpdate, Self::Exec), RuntimeError> {
        let call_frame_update = CallFrameUpdate::empty();
        let actor =
            ResolvedActor::function(NativeFn::ResourceManager(ResourceManagerFn::CreateFungible));
        Ok((actor, call_frame_update, self))
    }
}

impl Executor for ResourceManagerCreateFungibleInvocation {
    type Output = ResourceAddress;

    fn execute<Y, W: WasmEngine>(
        self,
        api: &mut Y,
    ) -> Result<(ResourceAddress, CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi,
    {
        let global_node_id = api.allocate_node_id(RENodeType::GlobalResourceManager)?;
        let resource_address: ResourceAddress = global_node_id.into();

        let resource_manager_substate = ResourceManagerSubstate::new(
            ResourceType::Fungible {
                divisibility: self.divisibility,
            },
            None,
            resource_address,
        );
        let (substate, vault_substate) = build_substates(self.access_rules);
        let metadata_substate = MetadataSubstate {
            metadata: self.metadata,
        };

        let underlying_node_id = api.allocate_node_id(RENodeType::ResourceManager)?;
        api.create_node(
            underlying_node_id,
            RENodeInit::ResourceManager(
                resource_manager_substate,
                metadata_substate,
                substate,
                vault_substate,
            ),
        )?;
        api.create_node(
            global_node_id,
            RENodeInit::Global(GlobalAddressSubstate::Resource(underlying_node_id.into())),
        )?;

        let update =
            CallFrameUpdate::copy_ref(RENodeId::Global(GlobalAddress::Resource(resource_address)));

        Ok((resource_address, update))
    }
}

impl ExecutableInvocation for ResourceManagerCreateNonFungibleWithInitialSupplyInvocation {
    type Exec = Self;

    fn resolve<D: ResolverApi>(
        self,
        _api: &mut D,
    ) -> Result<(ResolvedActor, CallFrameUpdate, Self::Exec), RuntimeError> {
        let call_frame_update = CallFrameUpdate::empty();
        let actor = ResolvedActor::function(NativeFn::ResourceManager(
            ResourceManagerFn::CreateNonFungibleWithInitialSupply,
        ));
        Ok((actor, call_frame_update, self))
    }
}

impl Executor for ResourceManagerCreateNonFungibleWithInitialSupplyInvocation {
    type Output = (ResourceAddress, Bucket);

    fn execute<Y, W: WasmEngine>(
        self,
        api: &mut Y,
    ) -> Result<((ResourceAddress, Bucket), CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi,
    {
        let global_node_id = api.allocate_node_id(RENodeType::GlobalResourceManager)?;
        let resource_address: ResourceAddress = global_node_id.into();

        // TODO: Do this check in a better way (e.g. via type check)
        if self.id_type == NonFungibleIdType::UUID {
            return Err(RuntimeError::ApplicationError(
                ApplicationError::ResourceManagerError(
                    ResourceManagerError::InvalidNonFungibleIdType,
                ),
            ));
        }

        let (resource_manager_substate, bucket) =
            build_non_fungible_resource_manager_substate_with_initial_supply(
                resource_address,
                self.id_type,
                self.entries,
                api,
            )?;
        let (substate, vault_substate) = build_substates(self.access_rules);
        let metadata_substate = MetadataSubstate {
            metadata: self.metadata,
        };

        let underlying_node_id = api.allocate_node_id(RENodeType::ResourceManager)?;
        api.create_node(
            underlying_node_id,
            RENodeInit::ResourceManager(
                resource_manager_substate,
                metadata_substate,
                substate,
                vault_substate,
            ),
        )?;

        api.create_node(
            global_node_id,
            RENodeInit::Global(GlobalAddressSubstate::Resource(underlying_node_id.into())),
        )?;

        let mut nodes_to_move = vec![];
        nodes_to_move.push(RENodeId::Bucket(bucket.0));

        let mut node_refs_to_copy = HashSet::new();
        node_refs_to_copy.insert(RENodeId::Global(GlobalAddress::Resource(resource_address)));

        Ok((
            (resource_address, bucket),
            CallFrameUpdate {
                nodes_to_move,
                node_refs_to_copy,
            },
        ))
    }
}

impl ExecutableInvocation for ResourceManagerCreateUuidNonFungibleWithInitialSupplyInvocation {
    type Exec = Self;

    fn resolve<D: ResolverApi>(
        self,
        _api: &mut D,
    ) -> Result<(ResolvedActor, CallFrameUpdate, Self::Exec), RuntimeError> {
        let call_frame_update = CallFrameUpdate::empty();
        let actor = ResolvedActor::function(NativeFn::ResourceManager(
            ResourceManagerFn::CreateUuidNonFungibleWithInitialSupply,
        ));
        Ok((actor, call_frame_update, self))
    }
}

impl Executor for ResourceManagerCreateUuidNonFungibleWithInitialSupplyInvocation {
    type Output = (ResourceAddress, Bucket);

    fn execute<Y, W: WasmEngine>(
        self,
        api: &mut Y,
    ) -> Result<((ResourceAddress, Bucket), CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi + EngineApi<RuntimeError> + InvokableModel<RuntimeError>,
    {
        let global_node_id = api.allocate_node_id(RENodeType::GlobalResourceManager)?;
        let resource_address: ResourceAddress = global_node_id.into();

        let mut entries = BTreeMap::new();
        for entry in self.entries {
            let uuid = Runtime::generate_uuid(api)?;
            entries.insert(NonFungibleLocalId::uuid(uuid).unwrap(), entry);
        }

        let (resource_manager_substate, bucket) =
            build_non_fungible_resource_manager_substate_with_initial_supply(
                resource_address,
                NonFungibleIdType::UUID,
                entries,
                api,
            )?;
        let (substate, vault_substate) = build_substates(self.access_rules);
        let metadata_substate = MetadataSubstate {
            metadata: self.metadata,
        };

        let underlying_node_id = api.allocate_node_id(RENodeType::ResourceManager)?;
        api.create_node(
            underlying_node_id,
            RENodeInit::ResourceManager(
                resource_manager_substate,
                metadata_substate,
                substate,
                vault_substate,
            ),
        )?;

        api.create_node(
            global_node_id,
            RENodeInit::Global(GlobalAddressSubstate::Resource(underlying_node_id.into())),
        )?;

        let mut nodes_to_move = vec![];
        nodes_to_move.push(RENodeId::Bucket(bucket.0));

        let mut node_refs_to_copy = HashSet::new();
        node_refs_to_copy.insert(RENodeId::Global(GlobalAddress::Resource(resource_address)));

        Ok((
            (resource_address, bucket),
            CallFrameUpdate {
                nodes_to_move,
                node_refs_to_copy,
            },
        ))
    }
}

impl ExecutableInvocation for ResourceManagerCreateFungibleWithInitialSupplyInvocation {
    type Exec = Self;

    fn resolve<D: ResolverApi>(
        self,
        _api: &mut D,
    ) -> Result<(ResolvedActor, CallFrameUpdate, Self::Exec), RuntimeError> {
        let call_frame_update = CallFrameUpdate::empty();
        let actor = ResolvedActor::function(NativeFn::ResourceManager(
            ResourceManagerFn::CreateFungibleWithInitialSupply,
        ));
        Ok((actor, call_frame_update, self))
    }
}

impl Executor for ResourceManagerCreateFungibleWithInitialSupplyInvocation {
    type Output = (ResourceAddress, Bucket);

    fn execute<Y, W: WasmEngine>(
        self,
        api: &mut Y,
    ) -> Result<((ResourceAddress, Bucket), CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi,
    {
        let global_node_id = if let Some(address) = self.resource_address {
            RENodeId::Global(GlobalAddress::Resource(ResourceAddress::Normal(address)))
        } else {
            api.allocate_node_id(RENodeType::GlobalResourceManager)?
        };

        let resource_address: ResourceAddress = global_node_id.into();

        let (resource_manager_substate, bucket) =
            build_fungible_resource_manager_substate_with_initial_supply(
                resource_address,
                self.divisibility,
                self.initial_supply,
                api,
            )?;
        let (substate, vault_substate) = build_substates(self.access_rules);
        let metadata_substate = MetadataSubstate {
            metadata: self.metadata,
        };

        let underlying_node_id = api.allocate_node_id(RENodeType::ResourceManager)?;
        api.create_node(
            underlying_node_id,
            RENodeInit::ResourceManager(
                resource_manager_substate,
                metadata_substate,
                substate,
                vault_substate,
            ),
        )?;

        api.create_node(
            global_node_id,
            RENodeInit::Global(GlobalAddressSubstate::Resource(underlying_node_id.into())),
        )?;

        let mut nodes_to_move = vec![];
        nodes_to_move.push(RENodeId::Bucket(bucket.0));

        let mut node_refs_to_copy = HashSet::new();
        node_refs_to_copy.insert(RENodeId::Global(GlobalAddress::Resource(resource_address)));

        Ok((
            (resource_address, bucket),
            CallFrameUpdate {
                nodes_to_move,
                node_refs_to_copy,
            },
        ))
    }
}

pub struct ResourceManagerBurnExecutable(RENodeId, Bucket);

impl ExecutableInvocation for ResourceManagerBurnInvocation {
    type Exec = ResourceManagerBurnExecutable;

    fn resolve<D: ResolverApi>(
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
        system_api: &mut Y,
    ) -> Result<((), CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi,
    {
        let offset = SubstateOffset::ResourceManager(ResourceManagerOffset::ResourceManager);
        let resman_handle = system_api.lock_substate(self.0, offset, LockFlags::MUTABLE)?;

        let bucket: BucketSubstate = system_api.drop_node(RENodeId::Bucket(self.1 .0))?.into();

        // Check if resource matches
        // TODO: Move this check into actor check
        {
            let substate_ref = system_api.get_ref(resman_handle)?;
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
            let mut substate_mut = system_api.get_ref_mut(resman_handle)?;
            let resource_manager = substate_mut.resource_manager();
            resource_manager.total_supply -= bucket.total_amount();
        }

        // Burn non-fungible
        let substate_ref = system_api.get_ref(resman_handle)?;
        let resource_manager = substate_ref.resource_manager();
        if let Some(nf_store_id) = resource_manager.nf_store_id {
            let node_id = RENodeId::NonFungibleStore(nf_store_id);

            for id in bucket
                .total_ids()
                .expect("Failed to list non-fungible IDs on non-fungible Bucket")
            {
                let offset = SubstateOffset::NonFungibleStore(NonFungibleStoreOffset::Entry(id));
                let non_fungible_handle =
                    system_api.lock_substate(node_id, offset, LockFlags::MUTABLE)?;
                let mut substate_mut = system_api.get_ref_mut(non_fungible_handle)?;
                let non_fungible_mut = substate_mut.non_fungible();

                *non_fungible_mut = NonFungibleSubstate(None);
                system_api.drop_lock(non_fungible_handle)?;
            }
        }

        Ok(((), CallFrameUpdate::empty()))
    }
}

pub struct ResourceManagerUpdateVaultAuthExecutable(RENodeId, VaultMethodAuthKey, AccessRule);

impl ExecutableInvocation for ResourceManagerUpdateVaultAuthInvocation {
    type Exec = ResourceManagerUpdateVaultAuthExecutable;

    fn resolve<D: ResolverApi>(
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
        Y: SystemApi + InvokableModel<RuntimeError>,
    {
        let offset =
            SubstateOffset::VaultAccessRulesChain(AccessRulesChainOffset::AccessRulesChain);
        let handle = api.lock_substate(self.0, offset, LockFlags::MUTABLE)?;

        // TODO: Figure out how to move this access check into more appropriate place
        {
            let node_ids = api.get_visible_nodes()?;
            let auth_zone_id = node_ids
                .into_iter()
                .find(|n| matches!(n, RENodeId::AuthZoneStack(..)))
                .expect("AuthZone does not exist");

            let substate_ref = api.get_ref(handle)?;
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

            api.invoke(AuthZoneAssertAccessRuleInvocation {
                receiver: auth_zone_id.into(),
                access_rule,
            })?;
        }

        let mut substate_mut = api.get_ref_mut(handle)?;
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

    fn resolve<D: ResolverApi>(
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
            NativeFn::ResourceManager(ResourceManagerFn::LockAuth),
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
        Y: SystemApi + InvokableModel<RuntimeError>,
    {
        let offset =
            SubstateOffset::VaultAccessRulesChain(AccessRulesChainOffset::AccessRulesChain);
        let handle = api.lock_substate(self.0, offset, LockFlags::MUTABLE)?;

        // TODO: Figure out how to move this access check into more appropriate place
        {
            let node_ids = api.get_visible_nodes()?;
            let auth_zone_id = node_ids
                .into_iter()
                .find(|n| matches!(n, RENodeId::AuthZoneStack(..)))
                .expect("AuthZone does not exist");

            let substate_ref = api.get_ref(handle)?;
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

            api.invoke(AuthZoneAssertAccessRuleInvocation {
                receiver: auth_zone_id.into(),
                access_rule,
            })?;
        }

        let mut substate_mut = api.get_ref_mut(handle)?;
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

    fn resolve<D: ResolverApi>(
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
        Y: SystemApi,
    {
        let offset = SubstateOffset::ResourceManager(ResourceManagerOffset::ResourceManager);
        let resman_handle = api.lock_substate(self.0, offset, LockFlags::MUTABLE)?;

        let substate_ref = api.get_ref(resman_handle)?;
        let resource_manager = substate_ref.resource_manager();
        let resource = Resource::new_empty(
            resource_manager.resource_address,
            resource_manager.resource_type,
        );

        let node_id = api.allocate_node_id(RENodeType::Vault)?;
        api.create_node(
            node_id,
            RENodeInit::Vault(VaultRuntimeSubstate::new(resource)),
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

    fn resolve<D: ResolverApi>(
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
        Y: SystemApi,
    {
        let offset = SubstateOffset::ResourceManager(ResourceManagerOffset::ResourceManager);
        let resman_handle = api.lock_substate(self.0, offset, LockFlags::MUTABLE)?;

        let substate_ref = api.get_ref(resman_handle)?;
        let resource_manager = substate_ref.resource_manager();
        let container = Resource::new_empty(
            resource_manager.resource_address,
            resource_manager.resource_type,
        );

        let node_id = api.allocate_node_id(RENodeType::Bucket)?;
        api.create_node(node_id, RENodeInit::Bucket(BucketSubstate::new(container)))?;
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

    fn resolve<D: ResolverApi>(
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
        Y: SystemApi,
    {
        let offset = SubstateOffset::ResourceManager(ResourceManagerOffset::ResourceManager);
        let resman_handle = api.lock_substate(self.0, offset, LockFlags::MUTABLE)?;

        let (resource, non_fungibles) = {
            let mut substate_mut = api.get_ref_mut(resman_handle)?;
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

        let node_id = api.allocate_node_id(RENodeType::Bucket)?;
        api.create_node(node_id, RENodeInit::Bucket(BucketSubstate::new(resource)))?;
        let bucket_id = node_id.into();

        let (nf_store_id, resource_address) = {
            let substate_ref = api.get_ref(resman_handle)?;
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
            let non_fungible_handle = api.lock_substate(node_id, offset, LockFlags::MUTABLE)?;

            {
                let mut substate_mut = api.get_ref_mut(non_fungible_handle)?;
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

            api.drop_lock(non_fungible_handle)?;
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

    fn resolve<D: ResolverApi>(
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
        Y: SystemApi + EngineApi<RuntimeError> + InvokableModel<RuntimeError>,
    {
        let offset = SubstateOffset::ResourceManager(ResourceManagerOffset::ResourceManager);
        let resman_handle = api.lock_substate(self.0, offset, LockFlags::MUTABLE)?;

        let bucket_id = {
            let mut substate_mut = api.get_ref_mut(resman_handle)?;
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
                let id = NonFungibleLocalId::uuid(uuid).unwrap();
                ids.insert(id.clone());

                {
                    let node_id = RENodeId::NonFungibleStore(nf_store_id);
                    let offset =
                        SubstateOffset::NonFungibleStore(NonFungibleStoreOffset::Entry(id));
                    let non_fungible_handle =
                        api.lock_substate(node_id, offset, LockFlags::MUTABLE)?;
                    let non_fungible = NonFungible::new(data.0, data.1);
                    let mut substate_mut = api.get_ref_mut(non_fungible_handle)?;
                    let non_fungible_mut = substate_mut.non_fungible();
                    *non_fungible_mut = NonFungibleSubstate(Some(non_fungible));
                    api.drop_lock(non_fungible_handle)?;
                }
            }

            let node_id = api.allocate_node_id(RENodeType::Bucket)?;
            api.create_node(
                node_id,
                RENodeInit::Bucket(BucketSubstate::new(Resource::new_non_fungible(
                    resource_address,
                    ids,
                    id_type,
                ))),
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

    fn resolve<D: ResolverApi>(
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
        Y: SystemApi,
    {
        let offset = SubstateOffset::ResourceManager(ResourceManagerOffset::ResourceManager);
        let resman_handle = api.lock_substate(self.0, offset, LockFlags::MUTABLE)?;

        let resource = {
            let mut substate_mut = api.get_ref_mut(resman_handle)?;
            let resource_manager = substate_mut.resource_manager();
            let result =
                resource_manager.mint_fungible(self.1, resource_manager.resource_address)?;
            result
        };

        let node_id = api.allocate_node_id(RENodeType::Bucket)?;
        api.create_node(node_id, RENodeInit::Bucket(BucketSubstate::new(resource)))?;
        let bucket_id = node_id.into();

        Ok((
            Bucket(bucket_id),
            CallFrameUpdate::move_node(RENodeId::Bucket(bucket_id)),
        ))
    }
}

impl ExecutableInvocation for ResourceManagerGetResourceTypeInvocation {
    type Exec = ResourceManagerGetResourceTypeExecutable;

    fn resolve<D: ResolverApi>(
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
        system_api: &mut Y,
    ) -> Result<(ResourceType, CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi,
    {
        let offset = SubstateOffset::ResourceManager(ResourceManagerOffset::ResourceManager);
        let resman_handle = system_api.lock_substate(self.0, offset, LockFlags::read_only())?;

        let substate_ref = system_api.get_ref(resman_handle)?;
        let resource_type = substate_ref.resource_manager().resource_type;

        Ok((resource_type, CallFrameUpdate::empty()))
    }
}

impl ExecutableInvocation for ResourceManagerGetTotalSupplyInvocation {
    type Exec = ResourceManagerGetTotalSupplyExecutable;

    fn resolve<D: ResolverApi>(
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
        system_api: &mut Y,
    ) -> Result<(Decimal, CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi,
    {
        let offset = SubstateOffset::ResourceManager(ResourceManagerOffset::ResourceManager);
        let resman_handle = system_api.lock_substate(self.0, offset, LockFlags::read_only())?;
        let substate_ref = system_api.get_ref(resman_handle)?;
        let total_supply = substate_ref.resource_manager().total_supply;

        Ok((total_supply, CallFrameUpdate::empty()))
    }
}

impl ExecutableInvocation for ResourceManagerUpdateNonFungibleDataInvocation {
    type Exec = ResourceManagerUpdateNonFungibleDataExecutable;

    fn resolve<D: ResolverApi>(
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
        system_api: &mut Y,
    ) -> Result<((), CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi,
    {
        let offset = SubstateOffset::ResourceManager(ResourceManagerOffset::ResourceManager);
        let resman_handle = system_api.lock_substate(self.0, offset, LockFlags::MUTABLE)?;

        let substate_ref = system_api.get_ref(resman_handle)?;
        let resource_manager = substate_ref.resource_manager();
        let nf_store_id = resource_manager
            .nf_store_id
            .ok_or(InvokeError::SelfError(ResourceManagerError::NotNonFungible))?;
        let resource_address = resource_manager.resource_address;

        let node_id = RENodeId::NonFungibleStore(nf_store_id);
        let offset =
            SubstateOffset::NonFungibleStore(NonFungibleStoreOffset::Entry(self.1.clone()));

        let non_fungible_handle = system_api.lock_substate(node_id, offset, LockFlags::MUTABLE)?;
        let mut substate_mut = system_api.get_ref_mut(non_fungible_handle)?;
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

        system_api.drop_lock(non_fungible_handle)?;

        Ok(((), CallFrameUpdate::empty()))
    }
}

impl ExecutableInvocation for ResourceManagerNonFungibleExistsInvocation {
    type Exec = ResourceManagerNonFungibleExistsExecutable;

    fn resolve<D: ResolverApi>(
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
        system_api: &mut Y,
    ) -> Result<(bool, CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi,
    {
        let offset = SubstateOffset::ResourceManager(ResourceManagerOffset::ResourceManager);
        let resman_handle = system_api.lock_substate(self.0, offset, LockFlags::read_only())?;

        let substate_ref = system_api.get_ref(resman_handle)?;
        let resource_manager = substate_ref.resource_manager();
        let nf_store_id = resource_manager
            .nf_store_id
            .ok_or(InvokeError::SelfError(ResourceManagerError::NotNonFungible))?;

        let node_id = RENodeId::NonFungibleStore(nf_store_id);
        let offset = SubstateOffset::NonFungibleStore(NonFungibleStoreOffset::Entry(self.1));
        let non_fungible_handle =
            system_api.lock_substate(node_id, offset, LockFlags::read_only())?;
        let substate = system_api.get_ref(non_fungible_handle)?;
        let exists = substate.non_fungible().0.is_some();

        Ok((exists, CallFrameUpdate::empty()))
    }
}

impl ExecutableInvocation for ResourceManagerGetNonFungibleInvocation {
    type Exec = ResourceManagerGetNonFungibleExecutable;

    fn resolve<D: ResolverApi>(
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
        system_api: &mut Y,
    ) -> Result<([Vec<u8>; 2], CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi,
    {
        let offset = SubstateOffset::ResourceManager(ResourceManagerOffset::ResourceManager);
        let resman_handle = system_api.lock_substate(self.0, offset, LockFlags::read_only())?;

        let substate_ref = system_api.get_ref(resman_handle)?;
        let resource_manager = substate_ref.resource_manager();
        let nf_store_id = resource_manager
            .nf_store_id
            .ok_or(InvokeError::SelfError(ResourceManagerError::NotNonFungible))?;

        let non_fungible_global_id =
            NonFungibleGlobalId::new(resource_manager.resource_address, self.1.clone());

        let node_id = RENodeId::NonFungibleStore(nf_store_id);
        let offset = SubstateOffset::NonFungibleStore(NonFungibleStoreOffset::Entry(self.1));
        let non_fungible_handle =
            system_api.lock_substate(node_id, offset, LockFlags::read_only())?;
        let non_fungible_ref = system_api.get_ref(non_fungible_handle)?;
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
