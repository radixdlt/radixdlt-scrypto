use crate::engine::{
    deref_and_update, ApplicationError, CallFrameUpdate, ExecutableInvocation, Invokable,
    LockFlags, MethodDeref, NativeExecutor, NativeProcedure, REActor, RENode, ResolvedFunction,
    ResolvedMethod, RuntimeError, SystemApi,
};
use crate::model::{AccessRulesSubstate, BucketSubstate, GlobalAddressSubstate, InvokeError, MetadataSubstate, NonFungible, NonFungibleSubstate, Resource, VaultRuntimeSubstate};
use crate::model::{NonFungibleStore, ResourceManagerSubstate};
use crate::types::*;
use radix_engine_interface::api::api::SysInvokableNative;
use radix_engine_interface::api::types::{
    GlobalAddress, NativeFunction, NativeMethod, NonFungibleStoreId, NonFungibleStoreOffset,
    RENodeId, ResourceManagerFunction, ResourceManagerMethod, ResourceManagerOffset,
    SubstateOffset,
};
use radix_engine_interface::dec;
use radix_engine_interface::math::Decimal;
use radix_engine_interface::model::AccessRule::{AllowAll, DenyAll};
use radix_engine_interface::model::VaultMethodAuthKey::{Deposit, Recall, Withdraw};
use radix_engine_interface::model::*;
use scrypto::resource::SysBucket;

/// Represents an error when accessing a bucket.
#[derive(Debug, Clone, PartialEq, Eq)]
#[scrypto(TypeId, Encode, Decode)]
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
    ResourceAddressAlreadySet,
}

impl ExecutableInvocation for ResourceManagerBucketBurnInvocation {
    type Exec = NativeExecutor<Self>;

    fn resolve<D: MethodDeref>(
        self,
        _deref: &mut D,
    ) -> Result<(REActor, CallFrameUpdate, Self::Exec), RuntimeError> {
        let call_frame_update = CallFrameUpdate::move_node(RENodeId::Bucket(self.bucket.0));
        let actor = REActor::Function(ResolvedFunction::Native(NativeFunction::ResourceManager(
            ResourceManagerFunction::BurnBucket,
        )));
        let executor = NativeExecutor(self);
        Ok((actor, call_frame_update, executor))
    }
}

impl NativeProcedure for ResourceManagerBucketBurnInvocation {
    type Output = ();

    fn main<Y>(self, env: &mut Y) -> Result<((), CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi + Invokable<ScryptoInvocation> + SysInvokableNative<RuntimeError>,
    {
        let bucket = Bucket(self.bucket.0);
        bucket.sys_burn(env)?;

        Ok(((), CallFrameUpdate::empty()))
    }
}

impl ExecutableInvocation for ResourceManagerCreateInvocation {
    type Exec = NativeExecutor<Self>;

    fn resolve<D: MethodDeref>(
        self,
        _deref: &mut D,
    ) -> Result<(REActor, CallFrameUpdate, Self::Exec), RuntimeError> {
        let call_frame_update = CallFrameUpdate::empty();
        let actor = REActor::Function(ResolvedFunction::Native(NativeFunction::ResourceManager(
            ResourceManagerFunction::Create,
        )));
        let executor = NativeExecutor(self);
        Ok((actor, call_frame_update, executor))
    }
}

impl NativeProcedure for ResourceManagerCreateInvocation {
    type Output = (ResourceAddress, Option<Bucket>);

    fn main<Y>(
        mut self,
        api: &mut Y,
    ) -> Result<((ResourceAddress, Option<Bucket>), CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi + Invokable<ScryptoInvocation>,
    {
        let global_node_id = api.allocate_node_id(RENodeType::GlobalResourceManager)?;
        let resource_manager_substate = if matches!(self.resource_type, ResourceType::NonFungible) {
            let nf_store_node_id = api.allocate_node_id(RENodeType::NonFungibleStore)?;
            api.create_node(
                nf_store_node_id,
                RENode::NonFungibleStore(NonFungibleStore::new()),
            )?;
            let nf_store_id: NonFungibleStoreId = nf_store_node_id.into();

            let mut resource_manager = ResourceManagerSubstate::new(
                self.resource_type,
                Some(nf_store_id),
                global_node_id.into(),
            )
            .map_err(|e| match e {
                InvokeError::Error(e) => {
                    RuntimeError::ApplicationError(ApplicationError::ResourceManagerError(e))
                }
                InvokeError::Downstream(e) => e,
            })?;

            if let Some(mint_params) = &self.mint_params {
                if let MintParams::NonFungible { entries } = mint_params {
                    for (non_fungible_id, data) in entries {
                        let offset = SubstateOffset::NonFungibleStore(
                            NonFungibleStoreOffset::Entry(non_fungible_id.clone()),
                        );
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
                } else {
                    return Err(RuntimeError::ApplicationError(
                        ApplicationError::ResourceManagerError(
                            ResourceManagerError::ResourceTypeDoesNotMatch,
                        ),
                    ));
                }
            }

            resource_manager
        } else {
            let mut resource_manager = ResourceManagerSubstate::new(
                self.resource_type,
                None,
                global_node_id.into(),
            )
            .map_err(|e| match e {
                InvokeError::Error(e) => {
                    RuntimeError::ApplicationError(ApplicationError::ResourceManagerError(e))
                }
                InvokeError::Downstream(e) => e,
            })?;

            if let Some(mint_params) = &self.mint_params {
                if let MintParams::Fungible { amount } = mint_params {
                    resource_manager
                        .check_amount(*amount)
                        .map_err(|e| match e {
                            InvokeError::Error(e) => RuntimeError::ApplicationError(
                                ApplicationError::ResourceManagerError(e),
                            ),
                            InvokeError::Downstream(e) => e,
                        })?;
                    // TODO: refactor this into mint function
                    if *amount > dec!("1000000000000000000") {
                        return Err(RuntimeError::ApplicationError(
                            ApplicationError::ResourceManagerError(
                                ResourceManagerError::MaxMintAmountExceeded,
                            ),
                        ));
                    }
                    resource_manager.total_supply = amount.clone();
                } else {
                    return Err(RuntimeError::ApplicationError(
                        ApplicationError::ResourceManagerError(
                            ResourceManagerError::ResourceTypeDoesNotMatch,
                        ),
                    ));
                }
            }

            resource_manager
        };

        let (mint_access_rule, mint_mutability) =
            self.access_rules.remove(&Mint).unwrap_or((DenyAll, LOCKED));
        let (burn_access_rule, burn_mutability) =
            self.access_rules.remove(&Burn).unwrap_or((DenyAll, LOCKED));
        let (update_non_fungible_data_access_rule, update_non_fungible_data_mutability) = self
            .access_rules
            .remove(&UpdateNonFungibleData)
            .unwrap_or((AllowAll, LOCKED));

        let mut access_rules = AccessRules::new();
        access_rules.set_access_rule_and_mutability(
            AccessRuleKey::Native(NativeFn::Method(NativeMethod::ResourceManager(
                ResourceManagerMethod::Mint,
            ))),
            mint_access_rule,
            mint_mutability.into(),
        );
        access_rules.set_access_rule_and_mutability(
            AccessRuleKey::Native(NativeFn::Method(NativeMethod::ResourceManager(
                ResourceManagerMethod::Burn,
            ))),
            burn_access_rule,
            burn_mutability.into(),
        );
        access_rules.set_access_rule_and_mutability(
            AccessRuleKey::Native(NativeFn::Method(NativeMethod::Metadata(
                MetadataMethod::Set,
            ))),
            AllowAll,
            DenyAll,
        );
        access_rules.set_access_rule_and_mutability(
            AccessRuleKey::Native(NativeFn::Method(NativeMethod::ResourceManager(
                ResourceManagerMethod::UpdateNonFungibleData,
            ))),
            update_non_fungible_data_access_rule,
            update_non_fungible_data_mutability.into(),
        );
        access_rules.set_access_rule_and_mutability(
            AccessRuleKey::Native(NativeFn::Method(NativeMethod::Metadata(
                MetadataMethod::Get,
            ))),
            AllowAll,
            DenyAll,
        );
        access_rules.set_access_rule_and_mutability(
            AccessRuleKey::Native(NativeFn::Method(NativeMethod::ResourceManager(
                ResourceManagerMethod::CreateBucket,
            ))),
            AllowAll,
            DenyAll,
        );
        access_rules.set_access_rule_and_mutability(
            AccessRuleKey::Native(NativeFn::Method(NativeMethod::ResourceManager(
                ResourceManagerMethod::GetResourceType,
            ))),
            AllowAll,
            DenyAll,
        );
        access_rules.set_access_rule_and_mutability(
            AccessRuleKey::Native(NativeFn::Method(NativeMethod::ResourceManager(
                ResourceManagerMethod::GetTotalSupply,
            ))),
            AllowAll,
            DenyAll,
        );
        access_rules.set_access_rule_and_mutability(
            AccessRuleKey::Native(NativeFn::Method(NativeMethod::ResourceManager(
                ResourceManagerMethod::CreateVault,
            ))),
            AllowAll,
            DenyAll,
        );
        access_rules.set_access_rule_and_mutability(
            AccessRuleKey::Native(NativeFn::Method(NativeMethod::ResourceManager(
                ResourceManagerMethod::NonFungibleExists,
            ))),
            AllowAll,
            DenyAll,
        );
        access_rules.set_access_rule_and_mutability(
            AccessRuleKey::Native(NativeFn::Method(NativeMethod::ResourceManager(
                ResourceManagerMethod::GetNonFungible,
            ))),
            AllowAll,
            DenyAll,
        );
        access_rules.set_access_rule_and_mutability(
            AccessRuleKey::Native(NativeFn::Method(NativeMethod::ResourceManager(
                ResourceManagerMethod::UpdateVaultAuth,
            ))),
            AllowAll, // Access verification occurs within method
            DenyAll,
        );
        access_rules.set_access_rule_and_mutability(
            AccessRuleKey::Native(NativeFn::Method(NativeMethod::ResourceManager(
                ResourceManagerMethod::LockAuth,
            ))),
            AllowAll, // Access verification occurs within method
            DenyAll,
        );

        let access_rules_substate = AccessRulesSubstate {
            access_rules: vec![access_rules],
        };

        let (deposit_access_rule, deposit_mutability) = self
            .access_rules
            .remove(&ResourceMethodAuthKey::Deposit)
            .unwrap_or((AllowAll, LOCKED));
        let (withdraw_access_rule, withdraw_mutability) = self
            .access_rules
            .remove(&ResourceMethodAuthKey::Withdraw)
            .unwrap_or((AllowAll, LOCKED));
        let (recall_access_rule, recall_mutability) = self
            .access_rules
            .remove(&ResourceMethodAuthKey::Recall)
            .unwrap_or((DenyAll, LOCKED));

        let mut vault_access_rules = AccessRules::new();
        vault_access_rules.set_group_access_rule_and_mutability(
            "withdraw".to_string(),
            withdraw_access_rule,
            withdraw_mutability.into(),
        );
        vault_access_rules.set_group_access_rule_and_mutability(
            "recall".to_string(),
            recall_access_rule,
            recall_mutability.into(),
        );
        vault_access_rules.set_group_and_mutability(
            AccessRuleKey::Native(NativeFn::Method(NativeMethod::Vault(VaultMethod::Take))),
            "withdraw".to_string(),
            DenyAll,
        );
        vault_access_rules.set_group_and_mutability(
            AccessRuleKey::Native(NativeFn::Method(NativeMethod::Vault(
                VaultMethod::TakeNonFungibles,
            ))),
            "withdraw".to_string(),
            DenyAll,
        );
        vault_access_rules.set_group_and_mutability(
            AccessRuleKey::Native(NativeFn::Method(NativeMethod::Vault(VaultMethod::LockFee))),
            "withdraw".to_string(),
            DenyAll,
        );

        vault_access_rules.set_access_rule_and_mutability(
            AccessRuleKey::Native(NativeFn::Method(NativeMethod::Vault(VaultMethod::Put))),
            deposit_access_rule,
            deposit_mutability.into(),
        );
        vault_access_rules.set_access_rule_and_mutability(
            AccessRuleKey::Native(NativeFn::Method(NativeMethod::Vault(
                VaultMethod::GetAmount,
            ))),
            AllowAll,
            DenyAll,
        );
        vault_access_rules.set_access_rule_and_mutability(
            AccessRuleKey::Native(NativeFn::Method(NativeMethod::Vault(
                VaultMethod::GetResourceAddress,
            ))),
            AllowAll,
            DenyAll,
        );
        vault_access_rules.set_access_rule_and_mutability(
            AccessRuleKey::Native(NativeFn::Method(NativeMethod::Vault(
                VaultMethod::GetNonFungibleIds,
            ))),
            AllowAll,
            DenyAll,
        );
        vault_access_rules.set_access_rule_and_mutability(
            AccessRuleKey::Native(NativeFn::Method(NativeMethod::Vault(
                VaultMethod::CreateProof,
            ))),
            AllowAll,
            DenyAll,
        );
        vault_access_rules.set_access_rule_and_mutability(
            AccessRuleKey::Native(NativeFn::Method(NativeMethod::Vault(
                VaultMethod::CreateProofByAmount,
            ))),
            AllowAll,
            DenyAll,
        );
        vault_access_rules.set_access_rule_and_mutability(
            AccessRuleKey::Native(NativeFn::Method(NativeMethod::Vault(
                VaultMethod::CreateProofByIds,
            ))),
            AllowAll,
            DenyAll,
        );

        let vault_access_rules_substate = AccessRulesSubstate {
            access_rules: vec![vault_access_rules],
        };

        let metadata_substate = MetadataSubstate {
            metadata: self.metadata
        };

        let underlying_node_id = api.allocate_node_id(RENodeType::ResourceManager)?;
        api.create_node(
            underlying_node_id,
            RENode::ResourceManager(
                resource_manager_substate,
                metadata_substate,
                access_rules_substate,
                vault_access_rules_substate,
            ),
        )?;

        api.create_node(
            global_node_id,
            RENode::Global(GlobalAddressSubstate::Resource(underlying_node_id.into())),
        )?;
        let resource_address: ResourceAddress = global_node_id.into();

        // Mint
        let bucket = if let Some(mint_params) = self.mint_params {
            let container = match mint_params {
                MintParams::NonFungible { entries } => {
                    let ids = entries.into_keys().collect();
                    Resource::new_non_fungible(resource_address, ids)
                }
                MintParams::Fungible { amount } => Resource::new_fungible(
                    resource_address,
                    self.resource_type.divisibility(),
                    amount,
                ),
            };

            let node_id = api.allocate_node_id(RENodeType::Bucket)?;
            api.create_node(node_id, RENode::Bucket(BucketSubstate::new(container)))?;
            let bucket_id = node_id.into();
            Some(Bucket(bucket_id))
        } else {
            None
        };

        let mut nodes_to_move = vec![];
        if let Some(bucket) = &bucket {
            nodes_to_move.push(RENodeId::Bucket(bucket.0));
        }

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
    type Exec = NativeExecutor<ResourceManagerBurnExecutable>;

    fn resolve<D: MethodDeref>(
        self,
        deref: &mut D,
    ) -> Result<(REActor, CallFrameUpdate, Self::Exec), RuntimeError> {
        let mut call_frame_update = CallFrameUpdate::move_node(RENodeId::Bucket(self.bucket.0));
        let resolved_receiver = deref_and_update(
            RENodeId::Global(GlobalAddress::Resource(self.receiver)),
            &mut call_frame_update,
            deref,
        )?;
        let actor = REActor::Method(
            ResolvedMethod::Native(NativeMethod::ResourceManager(ResourceManagerMethod::Burn)),
            resolved_receiver,
        );
        let executor = NativeExecutor(ResourceManagerBurnExecutable(
            resolved_receiver.receiver,
            self.bucket,
        ));
        Ok((actor, call_frame_update, executor))
    }
}

impl NativeProcedure for ResourceManagerBurnExecutable {
    type Output = ();

    fn main<'a, Y>(self, system_api: &mut Y) -> Result<((), CallFrameUpdate), RuntimeError>
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
    type Exec = NativeExecutor<ResourceManagerUpdateVaultAuthExecutable>;

    fn resolve<D: MethodDeref>(
        self,
        deref: &mut D,
    ) -> Result<(REActor, CallFrameUpdate, Self::Exec), RuntimeError> {
        let mut call_frame_update = CallFrameUpdate::empty();
        let resolved_receiver = deref_and_update(
            RENodeId::Global(GlobalAddress::Resource(self.receiver)),
            &mut call_frame_update,
            deref,
        )?;
        let actor = REActor::Method(
            ResolvedMethod::Native(NativeMethod::ResourceManager(
                ResourceManagerMethod::UpdateVaultAuth,
            )),
            resolved_receiver,
        );
        let executor = NativeExecutor(ResourceManagerUpdateVaultAuthExecutable(
            resolved_receiver.receiver,
            self.method,
            self.access_rule,
        ));
        Ok((actor, call_frame_update, executor))
    }
}

// TODO: Figure out better place to do vault auth (or child node authorization)
impl NativeProcedure for ResourceManagerUpdateVaultAuthExecutable {
    type Output = ();

    fn main<'a, Y>(self, api: &mut Y) -> Result<((), CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi + SysInvokableNative<RuntimeError>,
    {
        let offset = SubstateOffset::VaultAccessRules(AccessRulesOffset::AccessRules);
        let handle = api.lock_substate(self.0, offset, LockFlags::MUTABLE)?;

        // TODO: Figure out how to move this access check into more appropriate place
        {
            let node_ids = api.get_visible_node_ids()?;
            let auth_zone_id = node_ids
                .into_iter()
                .find(|n| matches!(n, RENodeId::AuthZoneStack(..)))
                .expect("AuthZone does not exist");

            let substate_ref = api.get_ref(handle)?;
            let access_rules_substate = substate_ref.access_rules();

            let access_rule = match self.1 {
                Deposit => {
                    let key = AccessRuleKey::Native(NativeFn::Method(NativeMethod::Vault(
                        VaultMethod::Put,
                    )));
                    access_rules_substate.access_rules[0].get_mutability(&key)
                }
                Withdraw => access_rules_substate.access_rules[0].get_group_mutability("withdraw"),
                Recall => access_rules_substate.access_rules[0].get_group_mutability("recall"),
            }
            .clone();

            api.sys_invoke(AuthZoneAssertAccessRuleInvocation {
                receiver: auth_zone_id.into(),
                access_rule,
            })?;
        }

        let mut substate_mut = api.get_ref_mut(handle)?;
        let access_rules_substate = substate_mut.access_rules();

        match self.1 {
            VaultMethodAuthKey::Deposit => {
                let key =
                    AccessRuleKey::Native(NativeFn::Method(NativeMethod::Vault(VaultMethod::Put)));
                access_rules_substate.access_rules[0].set_method_access_rule(key, self.2);
            }
            VaultMethodAuthKey::Withdraw => {
                let group_key = "withdraw".to_string();
                access_rules_substate.access_rules[0].set_group_access_rule(group_key, self.2);
            }
            VaultMethodAuthKey::Recall => {
                let group_key = "recall".to_string();
                access_rules_substate.access_rules[0].set_group_access_rule(group_key, self.2);
            }
        }

        Ok(((), CallFrameUpdate::empty()))
    }
}

impl ExecutableInvocation for ResourceManagerSetVaultAuthMutabilityInvocation {
    type Exec = NativeExecutor<ResourceManagerLockVaultAuthExecutable>;

    fn resolve<D: MethodDeref>(
        self,
        deref: &mut D,
    ) -> Result<(REActor, CallFrameUpdate, Self::Exec), RuntimeError> {
        let mut call_frame_update = CallFrameUpdate::empty();
        let resolved_receiver = deref_and_update(
            RENodeId::Global(GlobalAddress::Resource(self.receiver)),
            &mut call_frame_update,
            deref,
        )?;
        let actor = REActor::Method(
            ResolvedMethod::Native(NativeMethod::ResourceManager(
                ResourceManagerMethod::LockAuth,
            )),
            resolved_receiver,
        );
        let executor = NativeExecutor(ResourceManagerLockVaultAuthExecutable(
            resolved_receiver.receiver,
            self.method,
            self.mutability,
        ));
        Ok((actor, call_frame_update, executor))
    }
}

pub struct ResourceManagerLockVaultAuthExecutable(RENodeId, VaultMethodAuthKey, AccessRule);

impl NativeProcedure for ResourceManagerLockVaultAuthExecutable {
    type Output = ();

    fn main<'a, Y>(self, api: &mut Y) -> Result<((), CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi + SysInvokableNative<RuntimeError>,
    {
        let offset = SubstateOffset::VaultAccessRules(AccessRulesOffset::AccessRules);
        let handle = api.lock_substate(self.0, offset, LockFlags::MUTABLE)?;

        // TODO: Figure out how to move this access check into more appropriate place
        {
            let node_ids = api.get_visible_node_ids()?;
            let auth_zone_id = node_ids
                .into_iter()
                .find(|n| matches!(n, RENodeId::AuthZoneStack(..)))
                .expect("AuthZone does not exist");

            let substate_ref = api.get_ref(handle)?;
            let access_rules_substate = substate_ref.access_rules();

            let access_rule = match self.1 {
                Deposit => {
                    let key = AccessRuleKey::Native(NativeFn::Method(NativeMethod::Vault(
                        VaultMethod::Put,
                    )));
                    access_rules_substate.access_rules[0].get_mutability(&key)
                }
                Withdraw => access_rules_substate.access_rules[0].get_group_mutability("withdraw"),
                Recall => access_rules_substate.access_rules[0].get_group_mutability("recall"),
            }
            .clone();

            api.sys_invoke(AuthZoneAssertAccessRuleInvocation {
                receiver: auth_zone_id.into(),
                access_rule,
            })?;
        }

        let mut substate_mut = api.get_ref_mut(handle)?;
        let access_rules_substate = substate_mut.access_rules();

        match self.1 {
            Deposit => {
                let key =
                    AccessRuleKey::Native(NativeFn::Method(NativeMethod::Vault(VaultMethod::Put)));
                access_rules_substate.access_rules[0].set_mutability(key, self.2);
            }
            Withdraw => {
                let group_key = "withdraw".to_string();
                access_rules_substate.access_rules[0].set_group_mutability(group_key, self.2);
            }
            Recall => {
                let group_key = "recall".to_string();
                access_rules_substate.access_rules[0].set_group_mutability(group_key, self.2);
            }
        }

        Ok(((), CallFrameUpdate::empty()))
    }
}

impl ExecutableInvocation for ResourceManagerCreateVaultInvocation {
    type Exec = NativeExecutor<ResourceManagerCreateVaultExecutable>;

    fn resolve<D: MethodDeref>(
        self,
        deref: &mut D,
    ) -> Result<(REActor, CallFrameUpdate, Self::Exec), RuntimeError> {
        let mut call_frame_update = CallFrameUpdate::empty();
        let resolved_receiver = deref_and_update(
            RENodeId::Global(GlobalAddress::Resource(self.receiver)),
            &mut call_frame_update,
            deref,
        )?;
        let actor = REActor::Method(
            ResolvedMethod::Native(NativeMethod::ResourceManager(
                ResourceManagerMethod::CreateVault,
            )),
            resolved_receiver,
        );
        let executor = NativeExecutor(ResourceManagerCreateVaultExecutable(
            resolved_receiver.receiver,
        ));
        Ok((actor, call_frame_update, executor))
    }
}

pub struct ResourceManagerCreateVaultExecutable(RENodeId);

impl NativeProcedure for ResourceManagerCreateVaultExecutable {
    type Output = Vault;

    fn main<'a, Y>(self, api: &mut Y) -> Result<(Vault, CallFrameUpdate), RuntimeError>
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
        api.create_node(node_id, RENode::Vault(VaultRuntimeSubstate::new(resource)))?;
        let vault_id = node_id.into();

        Ok((
            Vault(vault_id),
            CallFrameUpdate::move_node(RENodeId::Vault(vault_id)),
        ))
    }
}

impl ExecutableInvocation for ResourceManagerCreateBucketInvocation {
    type Exec = NativeExecutor<ResourceManagerCreateBucketExecutable>;

    fn resolve<D: MethodDeref>(
        self,
        deref: &mut D,
    ) -> Result<(REActor, CallFrameUpdate, Self::Exec), RuntimeError> {
        let mut call_frame_update = CallFrameUpdate::empty();
        let resolved_receiver = deref_and_update(
            RENodeId::Global(GlobalAddress::Resource(self.receiver)),
            &mut call_frame_update,
            deref,
        )?;
        let actor = REActor::Method(
            ResolvedMethod::Native(NativeMethod::ResourceManager(
                ResourceManagerMethod::CreateBucket,
            )),
            resolved_receiver,
        );
        let executor = NativeExecutor(ResourceManagerCreateBucketExecutable(
            resolved_receiver.receiver,
        ));
        Ok((actor, call_frame_update, executor))
    }
}

pub struct ResourceManagerCreateBucketExecutable(RENodeId);

impl NativeProcedure for ResourceManagerCreateBucketExecutable {
    type Output = Bucket;

    fn main<'a, Y>(self, api: &mut Y) -> Result<(Bucket, CallFrameUpdate), RuntimeError>
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
        api.create_node(node_id, RENode::Bucket(BucketSubstate::new(container)))?;
        let bucket_id = node_id.into();

        Ok((
            Bucket(bucket_id),
            CallFrameUpdate::move_node(RENodeId::Bucket(bucket_id)),
        ))
    }
}

impl ExecutableInvocation for ResourceManagerMintInvocation {
    type Exec = NativeExecutor<ResourceManagerMintExecutable>;

    fn resolve<D: MethodDeref>(
        self,
        deref: &mut D,
    ) -> Result<(REActor, CallFrameUpdate, Self::Exec), RuntimeError> {
        let mut call_frame_update = CallFrameUpdate::empty();
        let resolved_receiver = deref_and_update(
            RENodeId::Global(GlobalAddress::Resource(self.receiver)),
            &mut call_frame_update,
            deref,
        )?;
        let actor = REActor::Method(
            ResolvedMethod::Native(NativeMethod::ResourceManager(ResourceManagerMethod::Mint)),
            resolved_receiver,
        );
        let executor = NativeExecutor(ResourceManagerMintExecutable(
            resolved_receiver.receiver,
            self.mint_params,
        ));
        Ok((actor, call_frame_update, executor))
    }
}

pub struct ResourceManagerMintExecutable(RENodeId, MintParams);

impl NativeProcedure for ResourceManagerMintExecutable {
    type Output = Bucket;

    fn main<'a, Y>(self, api: &mut Y) -> Result<(Bucket, CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi,
    {
        let offset = SubstateOffset::ResourceManager(ResourceManagerOffset::ResourceManager);
        let resman_handle = api.lock_substate(self.0, offset, LockFlags::MUTABLE)?;

        let (resource, non_fungibles) = {
            let mut substate_mut = api.get_ref_mut(resman_handle)?;
            let resource_manager = substate_mut.resource_manager();
            let result = resource_manager
                .mint(self.1, resource_manager.resource_address)
                .map_err(|e| match e {
                    InvokeError::Error(e) => {
                        RuntimeError::ApplicationError(ApplicationError::ResourceManagerError(e))
                    }
                    InvokeError::Downstream(runtime_error) => runtime_error,
                })?;
            result
        };

        let node_id = api.allocate_node_id(RENodeType::Bucket)?;
        api.create_node(node_id, RENode::Bucket(BucketSubstate::new(resource)))?;
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
                                NonFungibleAddress::new(resource_address, id),
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

impl ExecutableInvocation for ResourceManagerGetResourceTypeInvocation {
    type Exec = NativeExecutor<ResourceManagerGetResourceTypeExecutable>;

    fn resolve<D: MethodDeref>(
        self,
        deref: &mut D,
    ) -> Result<(REActor, CallFrameUpdate, Self::Exec), RuntimeError> {
        let mut call_frame_update = CallFrameUpdate::empty();
        let resolved_receiver = deref_and_update(
            RENodeId::Global(GlobalAddress::Resource(self.receiver)),
            &mut call_frame_update,
            deref,
        )?;
        let actor = REActor::Method(
            ResolvedMethod::Native(NativeMethod::ResourceManager(
                ResourceManagerMethod::GetResourceType,
            )),
            resolved_receiver,
        );
        let executor = NativeExecutor(ResourceManagerGetResourceTypeExecutable(
            resolved_receiver.receiver,
        ));
        Ok((actor, call_frame_update, executor))
    }
}

pub struct ResourceManagerGetResourceTypeExecutable(RENodeId);

impl NativeProcedure for ResourceManagerGetResourceTypeExecutable {
    type Output = ResourceType;

    fn main<'a, Y>(
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
    type Exec = NativeExecutor<ResourceManagerGetTotalSupplyExecutable>;

    fn resolve<D: MethodDeref>(
        self,
        deref: &mut D,
    ) -> Result<(REActor, CallFrameUpdate, Self::Exec), RuntimeError> {
        let mut call_frame_update = CallFrameUpdate::empty();
        let resolved_receiver = deref_and_update(
            RENodeId::Global(GlobalAddress::Resource(self.receiver)),
            &mut call_frame_update,
            deref,
        )?;
        let actor = REActor::Method(
            ResolvedMethod::Native(NativeMethod::ResourceManager(
                ResourceManagerMethod::GetTotalSupply,
            )),
            resolved_receiver,
        );
        let executor = NativeExecutor(ResourceManagerGetTotalSupplyExecutable(
            resolved_receiver.receiver,
        ));
        Ok((actor, call_frame_update, executor))
    }
}

pub struct ResourceManagerGetTotalSupplyExecutable(RENodeId);

impl NativeProcedure for ResourceManagerGetTotalSupplyExecutable {
    type Output = Decimal;

    fn main<'a, Y>(self, system_api: &mut Y) -> Result<(Decimal, CallFrameUpdate), RuntimeError>
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
    type Exec = NativeExecutor<ResourceManagerUpdateNonFungibleDataExecutable>;

    fn resolve<D: MethodDeref>(
        self,
        deref: &mut D,
    ) -> Result<(REActor, CallFrameUpdate, Self::Exec), RuntimeError> {
        let mut call_frame_update = CallFrameUpdate::empty();
        let resolved_receiver = deref_and_update(
            RENodeId::Global(GlobalAddress::Resource(self.receiver)),
            &mut call_frame_update,
            deref,
        )?;
        let actor = REActor::Method(
            ResolvedMethod::Native(NativeMethod::ResourceManager(
                ResourceManagerMethod::UpdateNonFungibleData,
            )),
            resolved_receiver,
        );
        let executor = NativeExecutor(ResourceManagerUpdateNonFungibleDataExecutable(
            resolved_receiver.receiver,
            self.id,
            self.data,
        ));
        Ok((actor, call_frame_update, executor))
    }
}

pub struct ResourceManagerUpdateNonFungibleDataExecutable(RENodeId, NonFungibleId, Vec<u8>);

impl NativeProcedure for ResourceManagerUpdateNonFungibleDataExecutable {
    type Output = ();

    fn main<'a, Y>(self, system_api: &mut Y) -> Result<((), CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi,
    {
        let offset = SubstateOffset::ResourceManager(ResourceManagerOffset::ResourceManager);
        let resman_handle = system_api.lock_substate(self.0, offset, LockFlags::MUTABLE)?;

        let substate_ref = system_api.get_ref(resman_handle)?;
        let resource_manager = substate_ref.resource_manager();
        let nf_store_id = resource_manager
            .nf_store_id
            .ok_or(InvokeError::Error(ResourceManagerError::NotNonFungible))
            .map_err(|e| match e {
                InvokeError::Error(e) => {
                    RuntimeError::ApplicationError(ApplicationError::ResourceManagerError(e))
                }
                InvokeError::Downstream(runtime_error) => runtime_error,
            })?;
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
            let non_fungible_address = NonFungibleAddress::new(resource_address, self.1);
            return Err(RuntimeError::ApplicationError(
                ApplicationError::ResourceManagerError(ResourceManagerError::NonFungibleNotFound(
                    non_fungible_address,
                )),
            ));
        }

        system_api.drop_lock(non_fungible_handle)?;

        Ok(((), CallFrameUpdate::empty()))
    }
}

impl ExecutableInvocation for ResourceManagerNonFungibleExistsInvocation {
    type Exec = NativeExecutor<ResourceManagerNonFungibleExistsExecutable>;

    fn resolve<D: MethodDeref>(
        self,
        deref: &mut D,
    ) -> Result<(REActor, CallFrameUpdate, Self::Exec), RuntimeError> {
        let mut call_frame_update = CallFrameUpdate::empty();
        let resolved_receiver = deref_and_update(
            RENodeId::Global(GlobalAddress::Resource(self.receiver)),
            &mut call_frame_update,
            deref,
        )?;
        let actor = REActor::Method(
            ResolvedMethod::Native(NativeMethod::ResourceManager(
                ResourceManagerMethod::NonFungibleExists,
            )),
            resolved_receiver,
        );
        let executor = NativeExecutor(ResourceManagerNonFungibleExistsExecutable(
            resolved_receiver.receiver,
            self.id,
        ));
        Ok((actor, call_frame_update, executor))
    }
}

pub struct ResourceManagerNonFungibleExistsExecutable(RENodeId, NonFungibleId);

impl NativeProcedure for ResourceManagerNonFungibleExistsExecutable {
    type Output = bool;

    fn main<'a, Y>(self, system_api: &mut Y) -> Result<(bool, CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi,
    {
        let offset = SubstateOffset::ResourceManager(ResourceManagerOffset::ResourceManager);
        let resman_handle = system_api.lock_substate(self.0, offset, LockFlags::read_only())?;

        let substate_ref = system_api.get_ref(resman_handle)?;
        let resource_manager = substate_ref.resource_manager();
        let nf_store_id = resource_manager
            .nf_store_id
            .ok_or(InvokeError::Error(ResourceManagerError::NotNonFungible))
            .map_err(|e| match e {
                InvokeError::Error(e) => {
                    RuntimeError::ApplicationError(ApplicationError::ResourceManagerError(e))
                }
                InvokeError::Downstream(runtime_error) => runtime_error,
            })?;

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
    type Exec = NativeExecutor<ResourceManagerGetNonFungibleExecutable>;

    fn resolve<D: MethodDeref>(
        self,
        deref: &mut D,
    ) -> Result<(REActor, CallFrameUpdate, Self::Exec), RuntimeError> {
        let mut call_frame_update = CallFrameUpdate::empty();
        let resolved_receiver = deref_and_update(
            RENodeId::Global(GlobalAddress::Resource(self.receiver)),
            &mut call_frame_update,
            deref,
        )?;
        let actor = REActor::Method(
            ResolvedMethod::Native(NativeMethod::ResourceManager(
                ResourceManagerMethod::GetNonFungible,
            )),
            resolved_receiver,
        );
        let executor = NativeExecutor(ResourceManagerGetNonFungibleExecutable(
            resolved_receiver.receiver,
            self.id,
        ));
        Ok((actor, call_frame_update, executor))
    }
}

pub struct ResourceManagerGetNonFungibleExecutable(RENodeId, NonFungibleId);

impl NativeProcedure for ResourceManagerGetNonFungibleExecutable {
    type Output = [Vec<u8>; 2];

    fn main<Y>(self, system_api: &mut Y) -> Result<([Vec<u8>; 2], CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi,
    {
        let offset = SubstateOffset::ResourceManager(ResourceManagerOffset::ResourceManager);
        let resman_handle = system_api.lock_substate(self.0, offset, LockFlags::read_only())?;

        let substate_ref = system_api.get_ref(resman_handle)?;
        let resource_manager = substate_ref.resource_manager();
        let nf_store_id = resource_manager
            .nf_store_id
            .ok_or(InvokeError::Error(ResourceManagerError::NotNonFungible))
            .map_err(|e| match e {
                InvokeError::Error(e) => {
                    RuntimeError::ApplicationError(ApplicationError::ResourceManagerError(e))
                }
                InvokeError::Downstream(runtime_error) => runtime_error,
            })?;

        let non_fungible_address =
            NonFungibleAddress::new(resource_manager.resource_address, self.1.clone());

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
                    non_fungible_address,
                )),
            ));
        }
    }
}
