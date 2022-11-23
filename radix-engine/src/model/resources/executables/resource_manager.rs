use crate::engine::{
    deref_and_update, ApplicationError, CallFrameUpdate, ExecutableInvocation, Invokable,
    LockFlags, MethodDeref, NativeInvocation, NativeInvocationInfo, NativeProgram, REActor, RENode,
    ResolvedFunction, ResolvedMethod, ResolvedReceiver, RuntimeError, SystemApi, TypedExecutor,
};
use crate::model::{
    BucketSubstate, GlobalAddressSubstate, InvokeError, NonFungible, NonFungibleSubstate, Resource,
    VaultRuntimeSubstate,
};
use crate::model::{MethodAccessRuleMethod, NonFungibleStore, ResourceManagerSubstate};
use crate::types::*;
use radix_engine_interface::api::api::{Invocation, SysInvokableNative, SysInvokableNative2};
use radix_engine_interface::api::types::{
    GlobalAddress, NativeFunction, NativeMethod, NonFungibleStoreId, NonFungibleStoreOffset,
    RENodeId, ResourceManagerFunction, ResourceManagerMethod, ResourceManagerOffset,
    SubstateOffset,
};
use radix_engine_interface::data::IndexedScryptoValue;
use radix_engine_interface::dec;
use radix_engine_interface::math::Decimal;
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
    type Exec = TypedExecutor<Self>;

    fn prepare<D: MethodDeref>(
        self,
        _deref: &mut D,
    ) -> Result<(REActor, CallFrameUpdate, Self::Exec), RuntimeError> {
        let input = IndexedScryptoValue::from_typed(&self);
        let call_frame_update = CallFrameUpdate::move_node(RENodeId::Bucket(self.bucket.0));
        let actor = REActor::Function(ResolvedFunction::Native(NativeFunction::ResourceManager(
            ResourceManagerFunction::BurnBucket,
        )));
        let executor = TypedExecutor(self, input);
        Ok((actor, call_frame_update, executor))
    }
}

impl NativeProgram for ResourceManagerBucketBurnInvocation {
    type Output = ();

    fn main<Y>(self, env: &mut Y) -> Result<((), CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi
            + Invokable<ScryptoInvocation>
            + SysInvokableNative<RuntimeError>
            + SysInvokableNative2<RuntimeError>,
    {
        let bucket = Bucket(self.bucket.0);
        bucket.sys_burn(env)?;

        Ok(((), CallFrameUpdate::empty()))
    }
}

impl ExecutableInvocation for ResourceManagerCreateInvocation {
    type Exec = TypedExecutor<Self>;

    fn prepare<D: MethodDeref>(
        self,
        _deref: &mut D,
    ) -> Result<(REActor, CallFrameUpdate, Self::Exec), RuntimeError> {
        let input = IndexedScryptoValue::from_typed(&self);
        let call_frame_update = CallFrameUpdate::empty();
        let actor = REActor::Function(ResolvedFunction::Native(NativeFunction::ResourceManager(
            ResourceManagerFunction::Create,
        )));
        let executor = TypedExecutor(self, input);
        Ok((actor, call_frame_update, executor))
    }
}

impl NativeProgram for ResourceManagerCreateInvocation {
    type Output = (ResourceAddress, Option<Bucket>);

    fn main<Y>(
        self,
        system_api: &mut Y,
    ) -> Result<((ResourceAddress, Option<Bucket>), CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi
            + Invokable<ScryptoInvocation>
            + Invokable<ResourceManagerSetResourceAddressInvocation>,
    {
        let node_id = if matches!(self.resource_type, ResourceType::NonFungible) {
            let nf_store_node_id =
                system_api.create_node(RENode::NonFungibleStore(NonFungibleStore::new()))?;
            let nf_store_id: NonFungibleStoreId = nf_store_node_id.into();

            let mut resource_manager = ResourceManagerSubstate::new(
                self.resource_type,
                self.metadata,
                self.access_rules,
                Some(nf_store_id),
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
                        let non_fungible_handle = system_api.lock_substate(
                            nf_store_node_id,
                            offset,
                            LockFlags::MUTABLE,
                        )?;
                        let mut substate_mut = system_api.get_ref_mut(non_fungible_handle)?;
                        let non_fungible_mut = substate_mut.non_fungible();
                        *non_fungible_mut = NonFungibleSubstate(Some(
                            NonFungible::new(data.0.clone(), data.1.clone()), // FIXME: verify data
                        ));
                        system_api.drop_lock(non_fungible_handle)?;
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
            system_api.create_node(RENode::ResourceManager(resource_manager))?
        } else {
            let mut resource_manager = ResourceManagerSubstate::new(
                self.resource_type,
                self.metadata,
                self.access_rules,
                None,
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
            system_api.create_node(RENode::ResourceManager(resource_manager))?
        };
        let global_node_id = system_api.create_node(RENode::Global(
            GlobalAddressSubstate::Resource(node_id.into()),
        ))?;
        let resource_address: ResourceAddress = global_node_id.into();

        // FIXME this is temporary workaround for the resource address resolution problem
        system_api.invoke(ResourceManagerSetResourceAddressInvocation {
            receiver: resource_address,
        })?;

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
            let bucket_id = system_api
                .create_node(RENode::Bucket(BucketSubstate::new(container)))?
                .into();
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
    type Exec = TypedExecutor<ResourceManagerBurnExecutable>;

    fn prepare<D: MethodDeref>(
        self,
        deref: &mut D,
    ) -> Result<(REActor, CallFrameUpdate, Self::Exec), RuntimeError> {
        let input = IndexedScryptoValue::from_typed(&self);
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
        let executor = TypedExecutor(
            ResourceManagerBurnExecutable(resolved_receiver.receiver, self.bucket),
            input,
        );
        Ok((actor, call_frame_update, executor))
    }
}

impl NativeProgram for ResourceManagerBurnExecutable {
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
            if Some(bucket.resource_address()) != resource_manager.resource_address {
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

pub struct ResourceManagerUpdateAuthExecutable(RENodeId, ResourceMethodAuthKey, AccessRule);

impl ExecutableInvocation for ResourceManagerUpdateAuthInvocation {
    type Exec = TypedExecutor<ResourceManagerUpdateAuthExecutable>;

    fn prepare<D: MethodDeref>(
        self,
        deref: &mut D,
    ) -> Result<(REActor, CallFrameUpdate, Self::Exec), RuntimeError> {
        let input = IndexedScryptoValue::from_typed(&self);
        let mut call_frame_update = CallFrameUpdate::empty();
        let resolved_receiver = deref_and_update(
            RENodeId::Global(GlobalAddress::Resource(self.receiver)),
            &mut call_frame_update,
            deref,
        )?;
        let actor = REActor::Method(
            ResolvedMethod::Native(NativeMethod::ResourceManager(
                ResourceManagerMethod::UpdateAuth,
            )),
            resolved_receiver,
        );
        let executor = TypedExecutor(
            ResourceManagerUpdateAuthExecutable(
                resolved_receiver.receiver,
                self.method,
                self.access_rule,
            ),
            input,
        );
        Ok((actor, call_frame_update, executor))
    }
}

impl NativeProgram for ResourceManagerUpdateAuthExecutable {
    type Output = ();

    fn main<'a, Y>(self, system_api: &mut Y) -> Result<((), CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi,
    {
        let offset = SubstateOffset::ResourceManager(ResourceManagerOffset::ResourceManager);
        let resman_handle = system_api.lock_substate(self.0, offset, LockFlags::MUTABLE)?;

        let mut substate_mut = system_api.get_ref_mut(resman_handle)?;
        let method_entry = substate_mut
            .resource_manager()
            .authorization
            .get_mut(&self.1)
            .expect(&format!("Authorization for {:?} not specified", self.1));
        method_entry
            .main(MethodAccessRuleMethod::Update(self.2))
            .map_err(|e| match e {
                InvokeError::Error(e) => {
                    RuntimeError::ApplicationError(ApplicationError::ResourceManagerError(e))
                }
                InvokeError::Downstream(runtime_error) => runtime_error,
            })?;

        Ok(((), CallFrameUpdate::empty()))
    }
}

impl ExecutableInvocation for ResourceManagerLockAuthInvocation {
    type Exec = TypedExecutor<ResourceManagerLockAuthExecutable>;

    fn prepare<D: MethodDeref>(
        self,
        deref: &mut D,
    ) -> Result<(REActor, CallFrameUpdate, Self::Exec), RuntimeError> {
        let input = IndexedScryptoValue::from_typed(&self);
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
        let executor = TypedExecutor(
            ResourceManagerLockAuthExecutable(resolved_receiver.receiver, self.method),
            input,
        );
        Ok((actor, call_frame_update, executor))
    }
}

pub struct ResourceManagerLockAuthExecutable(RENodeId, ResourceMethodAuthKey);

impl NativeProgram for ResourceManagerLockAuthExecutable {
    type Output = ();

    fn main<'a, Y>(self, system_api: &mut Y) -> Result<((), CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi,
    {
        let offset = SubstateOffset::ResourceManager(ResourceManagerOffset::ResourceManager);
        let resman_handle = system_api.lock_substate(self.0, offset, LockFlags::MUTABLE)?;

        let mut substate_mut = system_api.get_ref_mut(resman_handle)?;
        let method_entry = substate_mut
            .resource_manager()
            .authorization
            .get_mut(&self.1)
            .expect(&format!("Authorization for {:?} not specified", self.1));
        method_entry
            .main(MethodAccessRuleMethod::Lock())
            .map_err(|e| match e {
                InvokeError::Error(e) => {
                    RuntimeError::ApplicationError(ApplicationError::ResourceManagerError(e))
                }
                InvokeError::Downstream(runtime_error) => runtime_error,
            })?;

        Ok(((), CallFrameUpdate::empty()))
    }
}

impl ExecutableInvocation for ResourceManagerCreateVaultInvocation {
    type Exec = TypedExecutor<ResourceManagerCreateVaultExecutable>;

    fn prepare<D: MethodDeref>(
        self,
        deref: &mut D,
    ) -> Result<(REActor, CallFrameUpdate, Self::Exec), RuntimeError> {
        let input = IndexedScryptoValue::from_typed(&self);
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
        let executor = TypedExecutor(
            ResourceManagerCreateVaultExecutable(resolved_receiver.receiver),
            input,
        );
        Ok((actor, call_frame_update, executor))
    }
}

pub struct ResourceManagerCreateVaultExecutable(RENodeId);

impl NativeProgram for ResourceManagerCreateVaultExecutable {
    type Output = Vault;

    fn main<'a, Y>(self, system_api: &mut Y) -> Result<(Vault, CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi,
    {
        let offset = SubstateOffset::ResourceManager(ResourceManagerOffset::ResourceManager);
        let resman_handle = system_api.lock_substate(self.0, offset, LockFlags::MUTABLE)?;

        let substate_ref = system_api.get_ref(resman_handle)?;
        let resource_manager = substate_ref.resource_manager();
        let resource = Resource::new_empty(
            resource_manager.resource_address.unwrap(),
            resource_manager.resource_type,
        );
        let vault_id = system_api
            .create_node(RENode::Vault(VaultRuntimeSubstate::new(resource)))?
            .into();

        Ok((
            Vault(vault_id),
            CallFrameUpdate::move_node(RENodeId::Vault(vault_id)),
        ))
    }
}

impl ExecutableInvocation for ResourceManagerCreateBucketInvocation {
    type Exec = TypedExecutor<ResourceManagerCreateBucketExecutable>;

    fn prepare<D: MethodDeref>(
        self,
        deref: &mut D,
    ) -> Result<(REActor, CallFrameUpdate, Self::Exec), RuntimeError> {
        let input = IndexedScryptoValue::from_typed(&self);
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
        let executor = TypedExecutor(
            ResourceManagerCreateBucketExecutable(resolved_receiver.receiver),
            input,
        );
        Ok((actor, call_frame_update, executor))
    }
}

pub struct ResourceManagerCreateBucketExecutable(RENodeId);

impl NativeProgram for ResourceManagerCreateBucketExecutable {
    type Output = Bucket;

    fn main<'a, Y>(self, system_api: &mut Y) -> Result<(Bucket, CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi,
    {
        let offset = SubstateOffset::ResourceManager(ResourceManagerOffset::ResourceManager);
        let resman_handle = system_api.lock_substate(self.0, offset, LockFlags::MUTABLE)?;

        let substate_ref = system_api.get_ref(resman_handle)?;
        let resource_manager = substate_ref.resource_manager();
        let container = Resource::new_empty(
            resource_manager.resource_address.unwrap(),
            resource_manager.resource_type,
        );
        let bucket_id = system_api
            .create_node(RENode::Bucket(BucketSubstate::new(container)))?
            .into();

        Ok((
            Bucket(bucket_id),
            CallFrameUpdate::move_node(RENodeId::Bucket(bucket_id)),
        ))
    }
}

impl ExecutableInvocation for ResourceManagerMintInvocation {
    type Exec = TypedExecutor<ResourceManagerMintExecutable>;

    fn prepare<D: MethodDeref>(
        self,
        deref: &mut D,
    ) -> Result<(REActor, CallFrameUpdate, Self::Exec), RuntimeError> {
        let input = IndexedScryptoValue::from_typed(&self);
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
        let executor = TypedExecutor(
            ResourceManagerMintExecutable(resolved_receiver.receiver, self.mint_params),
            input,
        );
        Ok((actor, call_frame_update, executor))
    }
}

pub struct ResourceManagerMintExecutable(RENodeId, MintParams);

impl NativeProgram for ResourceManagerMintExecutable {
    type Output = Bucket;

    fn main<'a, Y>(self, system_api: &mut Y) -> Result<(Bucket, CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi,
    {
        let offset = SubstateOffset::ResourceManager(ResourceManagerOffset::ResourceManager);
        let resman_handle = system_api.lock_substate(self.0, offset, LockFlags::MUTABLE)?;

        let (resource, non_fungibles) = {
            let mut substate_mut = system_api.get_ref_mut(resman_handle)?;
            let resource_manager = substate_mut.resource_manager();
            let result = resource_manager
                .mint(self.1, resource_manager.resource_address.unwrap())
                .map_err(|e| match e {
                    InvokeError::Error(e) => {
                        RuntimeError::ApplicationError(ApplicationError::ResourceManagerError(e))
                    }
                    InvokeError::Downstream(runtime_error) => runtime_error,
                })?;
            result
        };

        let bucket_id = system_api
            .create_node(RENode::Bucket(BucketSubstate::new(resource)))?
            .into();

        let (nf_store_id, resource_address) = {
            let substate_ref = system_api.get_ref(resman_handle)?;
            let resource_manager = substate_ref.resource_manager();
            (
                resource_manager.nf_store_id.clone(),
                resource_manager.resource_address.unwrap(),
            )
        };

        for (id, non_fungible) in non_fungibles {
            let node_id = RENodeId::NonFungibleStore(nf_store_id.unwrap());
            let offset =
                SubstateOffset::NonFungibleStore(NonFungibleStoreOffset::Entry(id.clone()));
            let non_fungible_handle =
                system_api.lock_substate(node_id, offset, LockFlags::MUTABLE)?;

            {
                let mut substate_mut = system_api.get_ref_mut(non_fungible_handle)?;
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

            system_api.drop_lock(non_fungible_handle)?;
        }

        Ok((
            Bucket(bucket_id),
            CallFrameUpdate::move_node(RENodeId::Bucket(bucket_id)),
        ))
    }
}

impl ExecutableInvocation for ResourceManagerGetMetadataInvocation {
    type Exec = TypedExecutor<ResourceManagerGetMetadataExecutable>;

    fn prepare<D: MethodDeref>(
        self,
        deref: &mut D,
    ) -> Result<(REActor, CallFrameUpdate, Self::Exec), RuntimeError> {
        let input = IndexedScryptoValue::from_typed(&self);
        let mut call_frame_update = CallFrameUpdate::empty();
        let resolved_receiver = deref_and_update(
            RENodeId::Global(GlobalAddress::Resource(self.receiver)),
            &mut call_frame_update,
            deref,
        )?;
        let actor = REActor::Method(
            ResolvedMethod::Native(NativeMethod::ResourceManager(
                ResourceManagerMethod::GetMetadata,
            )),
            resolved_receiver,
        );
        let executor = TypedExecutor(
            ResourceManagerGetMetadataExecutable(resolved_receiver.receiver),
            input,
        );
        Ok((actor, call_frame_update, executor))
    }
}

pub struct ResourceManagerGetMetadataExecutable(RENodeId);

impl NativeProgram for ResourceManagerGetMetadataExecutable {
    type Output = HashMap<String, String>;

    fn main<'a, Y>(
        self,
        system_api: &mut Y,
    ) -> Result<(HashMap<String, String>, CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi,
    {
        let offset = SubstateOffset::ResourceManager(ResourceManagerOffset::ResourceManager);
        let resman_handle = system_api.lock_substate(self.0, offset, LockFlags::read_only())?;

        let substate_ref = system_api.get_ref(resman_handle)?;
        let metadata = &substate_ref.resource_manager().metadata;

        Ok((metadata.clone(), CallFrameUpdate::empty()))
    }
}

impl ExecutableInvocation for ResourceManagerGetResourceTypeInvocation {
    type Exec = TypedExecutor<ResourceManagerGetResourceTypeExecutable>;

    fn prepare<D: MethodDeref>(
        self,
        deref: &mut D,
    ) -> Result<(REActor, CallFrameUpdate, Self::Exec), RuntimeError> {
        let input = IndexedScryptoValue::from_typed(&self);
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
        let executor = TypedExecutor(
            ResourceManagerGetResourceTypeExecutable(resolved_receiver.receiver),
            input,
        );
        Ok((actor, call_frame_update, executor))
    }
}

pub struct ResourceManagerGetResourceTypeExecutable(RENodeId);

impl NativeProgram for ResourceManagerGetResourceTypeExecutable {
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
    type Exec = TypedExecutor<ResourceManagerGetTotalSupplyExecutable>;

    fn prepare<D: MethodDeref>(
        self,
        deref: &mut D,
    ) -> Result<(REActor, CallFrameUpdate, Self::Exec), RuntimeError> {
        let input = IndexedScryptoValue::from_typed(&self);
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
        let executor = TypedExecutor(
            ResourceManagerGetTotalSupplyExecutable(resolved_receiver.receiver),
            input,
        );
        Ok((actor, call_frame_update, executor))
    }
}

pub struct ResourceManagerGetTotalSupplyExecutable(RENodeId);

impl NativeProgram for ResourceManagerGetTotalSupplyExecutable {
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

impl ExecutableInvocation for ResourceManagerUpdateMetadataInvocation {
    type Exec = TypedExecutor<ResourceManagerUpdateMetadataExecutable>;

    fn prepare<D: MethodDeref>(
        self,
        deref: &mut D,
    ) -> Result<(REActor, CallFrameUpdate, Self::Exec), RuntimeError> {
        let input = IndexedScryptoValue::from_typed(&self);
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
        let executor = TypedExecutor(
            ResourceManagerUpdateMetadataExecutable(resolved_receiver.receiver, self.metadata),
            input,
        );
        Ok((actor, call_frame_update, executor))
    }
}

pub struct ResourceManagerUpdateMetadataExecutable(RENodeId, HashMap<String, String>);

impl NativeProgram for ResourceManagerUpdateMetadataExecutable {
    type Output = ();

    fn main<'a, Y>(self, system_api: &mut Y) -> Result<((), CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi,
    {
        let offset = SubstateOffset::ResourceManager(ResourceManagerOffset::ResourceManager);
        let resman_handle = system_api.lock_substate(self.0, offset, LockFlags::MUTABLE)?;

        let mut substate_mut = system_api.get_ref_mut(resman_handle)?;
        substate_mut
            .resource_manager()
            .update_metadata(self.1)
            .map_err(|e| match e {
                InvokeError::Error(e) => {
                    RuntimeError::ApplicationError(ApplicationError::ResourceManagerError(e))
                }
                InvokeError::Downstream(runtime_error) => runtime_error,
            })?;

        Ok(((), CallFrameUpdate::empty()))
    }
}

impl ExecutableInvocation for ResourceManagerUpdateNonFungibleDataInvocation {
    type Exec = TypedExecutor<ResourceManagerUpdateNonFungibleDataExecutable>;

    fn prepare<D: MethodDeref>(
        self,
        deref: &mut D,
    ) -> Result<(REActor, CallFrameUpdate, Self::Exec), RuntimeError> {
        let input = IndexedScryptoValue::from_typed(&self);
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
        let executor = TypedExecutor(
            ResourceManagerUpdateNonFungibleDataExecutable(
                resolved_receiver.receiver,
                self.id,
                self.data,
            ),
            input,
        );
        Ok((actor, call_frame_update, executor))
    }
}

pub struct ResourceManagerUpdateNonFungibleDataExecutable(RENodeId, NonFungibleId, Vec<u8>);

impl NativeProgram for ResourceManagerUpdateNonFungibleDataExecutable {
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
        let resource_address = resource_manager.resource_address.unwrap();

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
    type Exec = TypedExecutor<ResourceManagerNonFungibleExistsExecutable>;

    fn prepare<D: MethodDeref>(
        self,
        deref: &mut D,
    ) -> Result<(REActor, CallFrameUpdate, Self::Exec), RuntimeError> {
        let input = IndexedScryptoValue::from_typed(&self);
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
        let executor = TypedExecutor(
            ResourceManagerNonFungibleExistsExecutable(resolved_receiver.receiver, self.id),
            input,
        );
        Ok((actor, call_frame_update, executor))
    }
}

pub struct ResourceManagerNonFungibleExistsExecutable(RENodeId, NonFungibleId);

impl NativeProgram for ResourceManagerNonFungibleExistsExecutable {
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
    type Exec = TypedExecutor<ResourceManagerGetNonFungibleExecutable>;

    fn prepare<D: MethodDeref>(
        self,
        deref: &mut D,
    ) -> Result<(REActor, CallFrameUpdate, Self::Exec), RuntimeError> {
        let input = IndexedScryptoValue::from_typed(&self);
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
        let executor = TypedExecutor(
            ResourceManagerGetNonFungibleExecutable(resolved_receiver.receiver, self.id),
            input,
        );
        Ok((actor, call_frame_update, executor))
    }
}

pub struct ResourceManagerGetNonFungibleExecutable(RENodeId, NonFungibleId);

impl NativeProgram for ResourceManagerGetNonFungibleExecutable {
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
            NonFungibleAddress::new(resource_manager.resource_address.unwrap(), self.1.clone());

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

#[derive(Debug)]
#[scrypto(TypeId, Encode, Decode)]
pub struct ResourceManagerSetResourceAddressInvocation {
    pub receiver: ResourceAddress,
}

impl Invocation for ResourceManagerSetResourceAddressInvocation {
    type Output = ();
}

impl NativeInvocation for ResourceManagerSetResourceAddressInvocation {
    fn info(&self) -> NativeInvocationInfo {
        NativeInvocationInfo::Method(
            NativeMethod::ResourceManager(ResourceManagerMethod::SetResourceAddress),
            RENodeId::Global(GlobalAddress::Resource(self.receiver)),
            CallFrameUpdate::empty(),
        )
    }

    fn execute<Y>(input: Self, system_api: &mut Y) -> Result<((), CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi,
    {
        // TODO: Remove this hack and get resolved receiver in a better way
        let node_id = match system_api.get_actor() {
            REActor::Method(_, ResolvedReceiver { receiver, .. }) => *receiver,
            _ => panic!("Unexpected"),
        };
        let offset = SubstateOffset::ResourceManager(ResourceManagerOffset::ResourceManager);
        let resman_handle = system_api.lock_substate(node_id, offset, LockFlags::MUTABLE)?;

        let mut substate_mut = system_api.get_ref_mut(resman_handle)?;
        substate_mut
            .resource_manager()
            .set_resource_address(input.receiver)
            .map_err(|e| match e {
                InvokeError::Error(e) => {
                    RuntimeError::ApplicationError(ApplicationError::ResourceManagerError(e))
                }
                InvokeError::Downstream(runtime_error) => runtime_error,
            })?;

        Ok(((), CallFrameUpdate::empty()))
    }
}
