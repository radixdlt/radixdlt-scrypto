use crate::engine::{
    ApplicationError, CallFrameUpdate, Invokable, InvokableNative, LockFlags, NativeExecutable,
    NativeInvocation, NativeInvocationInfo, REActor, RENode, ResolvedReceiver, RuntimeError,
    SystemApi,
};
use crate::fee::FeeReserve;
use crate::model::{
    BucketSubstate, GlobalAddressSubstate, InvokeError, NonFungible, NonFungibleSubstate, Resource,
    ResourceMethodRule::{Protected, Public},
    VaultRuntimeSubstate,
};
use crate::model::{
    MethodAccessRule, MethodAccessRuleMethod, NonFungibleStore, ResourceManagerSubstate,
    ResourceMethodRule,
};
use crate::types::AccessRule::*;
use crate::types::ResourceMethodAuthKey::*;
use crate::types::*;
use scrypto::resource::ResourceManagerBucketBurnInput;

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
    ResourceAddressAlreadySet,
}

impl NativeExecutable for ResourceManagerBucketBurnInput {
    type Output = ();

    fn execute<'s, 'a, Y, R>(
        invocation: Self,
        system_api: &mut Y,
    ) -> Result<((), CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi<'s, R>
            + Invokable<ScryptoInvocation>
            + InvokableNative<'a>
            + Invokable<NativeMethodInvocation>,
        R: FeeReserve,
    {
        let node_id = RENodeId::Bucket(invocation.bucket.0);
        let offset = SubstateOffset::Bucket(BucketOffset::Bucket);

        let bucket_handle = system_api.lock_substate(node_id, offset, LockFlags::read_only())?;
        let substate_ref = system_api.get_ref(bucket_handle)?;
        let resource_address = substate_ref.bucket().resource_address();

        system_api.drop_lock(bucket_handle)?;
        system_api.invoke(ResourceManagerBurnInput {
            resource_address,
            bucket: invocation.bucket,
        })?;

        Ok(((), CallFrameUpdate::empty()))
    }
}

impl NativeInvocation for ResourceManagerBucketBurnInput {
    fn info(&self) -> NativeInvocationInfo {
        let bucket = RENodeId::Bucket(self.bucket.0);
        let mut node_refs_to_copy = HashSet::new();

        node_refs_to_copy.insert(RENodeId::Global(GlobalAddress::Resource(RADIX_TOKEN)));
        node_refs_to_copy.insert(RENodeId::Global(GlobalAddress::System(EPOCH_MANAGER)));
        node_refs_to_copy.insert(RENodeId::Global(GlobalAddress::Resource(
            ECDSA_SECP256K1_TOKEN,
        )));
        node_refs_to_copy.insert(RENodeId::Global(GlobalAddress::Resource(
            EDDSA_ED25519_TOKEN,
        )));
        node_refs_to_copy.insert(RENodeId::Global(GlobalAddress::Package(ACCOUNT_PACKAGE)));

        NativeInvocationInfo::Function(
            NativeFunction::ResourceManager(ResourceManagerFunction::BurnBucket),
            CallFrameUpdate {
                nodes_to_move: vec![bucket],
                node_refs_to_copy,
            },
        )
    }
}

impl NativeExecutable for ResourceManagerCreateInput {
    type Output = (ResourceAddress, Option<scrypto::resource::Bucket>);

    fn execute<'s, 'a, Y, R>(
        invocation: Self,
        system_api: &mut Y,
    ) -> Result<
        (
            (ResourceAddress, Option<scrypto::resource::Bucket>),
            CallFrameUpdate,
        ),
        RuntimeError,
    >
    where
        Y: SystemApi<'s, R>
            + Invokable<ScryptoInvocation>
            + InvokableNative<'a>
            + Invokable<NativeMethodInvocation>,
        R: FeeReserve,
    {
        let node_id = if matches!(invocation.resource_type, ResourceType::NonFungible) {
            let nf_store_node_id =
                system_api.create_node(RENode::NonFungibleStore(NonFungibleStore::new()))?;
            let nf_store_id: NonFungibleStoreId = nf_store_node_id.into();

            let mut resource_manager = ResourceManager::new(
                invocation.resource_type,
                invocation.metadata,
                invocation.access_rules,
                Some(nf_store_id),
            )
            .map_err(|e| match e {
                InvokeError::Error(e) => {
                    RuntimeError::ApplicationError(ApplicationError::ResourceManagerError(e))
                }
                InvokeError::Downstream(e) => e,
            })?;

            if let Some(mint_params) = &invocation.mint_params {
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
            let mut resource_manager = ResourceManager::new(
                invocation.resource_type,
                invocation.metadata,
                invocation.access_rules,
                None,
            )
            .map_err(|e| match e {
                InvokeError::Error(e) => {
                    RuntimeError::ApplicationError(ApplicationError::ResourceManagerError(e))
                }
                InvokeError::Downstream(e) => e,
            })?;

            if let Some(mint_params) = &invocation.mint_params {
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
        system_api.invoke(NativeMethodInvocation(
            NativeMethod::ResourceManager(ResourceManagerMethod::SetResourceAddress),
            RENodeId::Global(GlobalAddress::Resource(resource_address)),
            ScryptoValue::from_typed(&ResourceManagerSetResourceAddressInput {
                address: resource_address,
            }),
        ))?;

        // Mint
        let bucket = if let Some(mint_params) = invocation.mint_params {
            let container = match mint_params {
                MintParams::NonFungible { entries } => {
                    let ids = entries.into_keys().collect();
                    Resource::new_non_fungible(resource_address, ids)
                }
                MintParams::Fungible { amount } => Resource::new_fungible(
                    resource_address,
                    invocation.resource_type.divisibility(),
                    amount,
                ),
            };
            let bucket_id = system_api
                .create_node(RENode::Bucket(BucketSubstate::new(container)))?
                .into();
            Some(scrypto::resource::Bucket(bucket_id))
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

impl NativeInvocation for ResourceManagerCreateInput {
    fn info(&self) -> NativeInvocationInfo {
        let mut node_refs_to_copy = HashSet::new();

        node_refs_to_copy.insert(RENodeId::Global(GlobalAddress::Resource(RADIX_TOKEN)));
        node_refs_to_copy.insert(RENodeId::Global(GlobalAddress::System(EPOCH_MANAGER)));
        node_refs_to_copy.insert(RENodeId::Global(GlobalAddress::Resource(
            ECDSA_SECP256K1_TOKEN,
        )));
        node_refs_to_copy.insert(RENodeId::Global(GlobalAddress::Resource(
            EDDSA_ED25519_TOKEN,
        )));
        node_refs_to_copy.insert(RENodeId::Global(GlobalAddress::Package(ACCOUNT_PACKAGE)));

        NativeInvocationInfo::Function(
            NativeFunction::ResourceManager(ResourceManagerFunction::Create),
            CallFrameUpdate {
                nodes_to_move: vec![],
                node_refs_to_copy,
            },
        )
    }
}

impl NativeExecutable for ResourceManagerBurnInput {
    type Output = ();

    fn execute<'s, 'a, Y, R>(
        input: Self,
        system_api: &mut Y,
    ) -> Result<((), CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi<'s, R> + InvokableNative<'a>,
        R: FeeReserve,
    {
        // TODO: Remove this hack and get resolved receiver in a better way
        let node_id = match system_api.get_actor() {
            REActor::Method(_, ResolvedReceiver { receiver, .. }) => *receiver,
            _ => panic!("Unexpected"),
        };
        let offset = SubstateOffset::ResourceManager(ResourceManagerOffset::ResourceManager);
        let resman_handle = system_api.lock_substate(node_id, offset, LockFlags::MUTABLE)?;

        let bucket: BucketSubstate = system_api
            .drop_node(RENodeId::Bucket(input.bucket.0))?
            .into();

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

impl NativeInvocation for ResourceManagerBurnInput {
    fn info(&self) -> NativeInvocationInfo {
        NativeInvocationInfo::Method(
            NativeMethod::ResourceManager(ResourceManagerMethod::Burn),
            RENodeId::Global(GlobalAddress::Resource(self.resource_address)),
            CallFrameUpdate::move_node(RENodeId::Bucket(self.bucket.0)),
        )
    }
}

impl NativeExecutable for ResourceManagerUpdateAuthInput {
    type Output = ();

    fn execute<'s, 'a, Y, R>(
        input: Self,
        system_api: &mut Y,
    ) -> Result<((), CallFrameUpdate), RuntimeError>
        where
            Y: SystemApi<'s, R> + InvokableNative<'a>,
            R: FeeReserve,
    {
        // TODO: Remove this hack and get resolved receiver in a better way
        let node_id = match system_api.get_actor() {
            REActor::Method(_, ResolvedReceiver { receiver, .. }) => *receiver,
            _ => panic!("Unexpected"),
        };
        let offset = SubstateOffset::ResourceManager(ResourceManagerOffset::ResourceManager);
        let resman_handle = system_api.lock_substate(node_id, offset, LockFlags::MUTABLE)?;


        let mut substate_mut = system_api.get_ref_mut(resman_handle)?;
        let method_entry = substate_mut
            .resource_manager()
            .authorization
            .get_mut(&input.method)
            .expect(&format!(
                "Authorization for {:?} not specified",
                input.method
            ));
        method_entry.main(MethodAccessRuleMethod::Update(input.access_rule))
            .map_err(|e| {
                match e {
                    InvokeError::Error(e) => RuntimeError::ApplicationError(ApplicationError::ResourceManagerError(e)),
                    InvokeError::Downstream(runtime_error) => runtime_error,
                }
            })?;

        Ok(((), CallFrameUpdate::empty()))
    }
}

impl NativeInvocation for ResourceManagerUpdateAuthInput {
    fn info(&self) -> NativeInvocationInfo {
        NativeInvocationInfo::Method(
            NativeMethod::ResourceManager(ResourceManagerMethod::UpdateAuth),
            RENodeId::Global(GlobalAddress::Resource(self.resource_address)),
            CallFrameUpdate::empty(),
        )
    }
}

impl NativeExecutable for ResourceManagerLockAuthInput {
    type Output = ();

    fn execute<'s, 'a, Y, R>(
        input: Self,
        system_api: &mut Y,
    ) -> Result<((), CallFrameUpdate), RuntimeError>
        where
            Y: SystemApi<'s, R> + InvokableNative<'a>,
            R: FeeReserve,
    {
        // TODO: Remove this hack and get resolved receiver in a better way
        let node_id = match system_api.get_actor() {
            REActor::Method(_, ResolvedReceiver { receiver, .. }) => *receiver,
            _ => panic!("Unexpected"),
        };
        let offset = SubstateOffset::ResourceManager(ResourceManagerOffset::ResourceManager);
        let resman_handle = system_api.lock_substate(node_id, offset, LockFlags::MUTABLE)?;

        let mut substate_mut = system_api.get_ref_mut(resman_handle)?;
        let method_entry = substate_mut
            .resource_manager()
            .authorization
            .get_mut(&input.method)
            .expect(&format!(
                "Authorization for {:?} not specified",
                input.method
            ));
        method_entry.main(MethodAccessRuleMethod::Lock()).map_err(|e| {
            match e {
                InvokeError::Error(e) => RuntimeError::ApplicationError(ApplicationError::ResourceManagerError(e)),
                InvokeError::Downstream(runtime_error) => runtime_error,
            }
        })?;

        Ok(((), CallFrameUpdate::empty()))
    }
}

impl NativeInvocation for ResourceManagerLockAuthInput {
    fn info(&self) -> NativeInvocationInfo {
        NativeInvocationInfo::Method(
            NativeMethod::ResourceManager(ResourceManagerMethod::LockAuth),
            RENodeId::Global(GlobalAddress::Resource(self.resource_address)),
            CallFrameUpdate::empty(),
        )
    }
}

impl NativeExecutable for ResourceManagerCreateVaultInput {
    type Output = scrypto::resource::Vault;

    fn execute<'s, 'a, Y, R>(
        _input: Self,
        system_api: &mut Y,
    ) -> Result<(scrypto::resource::Vault, CallFrameUpdate), RuntimeError>
        where
            Y: SystemApi<'s, R> + InvokableNative<'a>,
            R: FeeReserve,
    {
        // TODO: Remove this hack and get resolved receiver in a better way
        let node_id = match system_api.get_actor() {
            REActor::Method(_, ResolvedReceiver { receiver, .. }) => *receiver,
            _ => panic!("Unexpected"),
        };
        let offset = SubstateOffset::ResourceManager(ResourceManagerOffset::ResourceManager);
        let resman_handle = system_api.lock_substate(node_id, offset, LockFlags::MUTABLE)?;

        let substate_ref = system_api.get_ref(resman_handle)?;
        let resource_manager = substate_ref.resource_manager();
        let resource = Resource::new_empty(
            resource_manager.resource_address.unwrap(),
            resource_manager.resource_type,
        );
        let vault_id = system_api
            .create_node(RENode::Vault(VaultRuntimeSubstate::new(resource)))?
            .into();

        Ok((scrypto::resource::Vault(vault_id), CallFrameUpdate::move_node(RENodeId::Vault(vault_id))))
    }
}

impl NativeInvocation for ResourceManagerCreateVaultInput {
    fn info(&self) -> NativeInvocationInfo {
        NativeInvocationInfo::Method(
            NativeMethod::ResourceManager(ResourceManagerMethod::CreateVault),
            RENodeId::Global(GlobalAddress::Resource(self.resource_address)),
            CallFrameUpdate::empty(),
        )
    }
}

impl NativeExecutable for ResourceManagerCreateBucketInput {
    type Output = scrypto::resource::Bucket;

    fn execute<'s, 'a, Y, R>(
        _input: Self,
        system_api: &mut Y,
    ) -> Result<(scrypto::resource::Bucket, CallFrameUpdate), RuntimeError>
        where
            Y: SystemApi<'s, R> + InvokableNative<'a>,
            R: FeeReserve,
    {
        // TODO: Remove this hack and get resolved receiver in a better way
        let node_id = match system_api.get_actor() {
            REActor::Method(_, ResolvedReceiver { receiver, .. }) => *receiver,
            _ => panic!("Unexpected"),
        };
        let offset = SubstateOffset::ResourceManager(ResourceManagerOffset::ResourceManager);
        let resman_handle = system_api.lock_substate(node_id, offset, LockFlags::MUTABLE)?;

        let substate_ref = system_api.get_ref(resman_handle)?;
        let resource_manager = substate_ref.resource_manager();
        let container = Resource::new_empty(
            resource_manager.resource_address.unwrap(),
            resource_manager.resource_type,
        );
        let bucket_id = system_api
            .create_node(RENode::Bucket(BucketSubstate::new(container)))?
            .into();

        Ok((scrypto::resource::Bucket(bucket_id), CallFrameUpdate::move_node(RENodeId::Bucket(bucket_id))))
    }
}

impl NativeInvocation for ResourceManagerCreateBucketInput {
    fn info(&self) -> NativeInvocationInfo {
        NativeInvocationInfo::Method(
            NativeMethod::ResourceManager(ResourceManagerMethod::CreateBucket),
            RENodeId::Global(GlobalAddress::Resource(self.resource_address)),
            CallFrameUpdate::empty(),
        )
    }
}

impl NativeExecutable for ResourceManagerMintInput {
    type Output = scrypto::resource::Bucket;

    fn execute<'s, 'a, Y, R>(
        input: Self,
        system_api: &mut Y,
    ) -> Result<(scrypto::resource::Bucket, CallFrameUpdate), RuntimeError>
        where
            Y: SystemApi<'s, R> + InvokableNative<'a>,
            R: FeeReserve,
    {
        // TODO: Remove this hack and get resolved receiver in a better way
        let node_id = match system_api.get_actor() {
            REActor::Method(_, ResolvedReceiver { receiver, .. }) => *receiver,
            _ => panic!("Unexpected"),
        };
        let offset = SubstateOffset::ResourceManager(ResourceManagerOffset::ResourceManager);
        let resman_handle = system_api.lock_substate(node_id, offset, LockFlags::MUTABLE)?;

        let (resource, non_fungibles) = {
            let mut substate_mut = system_api.get_ref_mut(resman_handle)?;
            let resource_manager = substate_mut.resource_manager();
            let result = resource_manager.mint(
                input.mint_params,
                resource_manager.resource_address.unwrap(),
            ).map_err(|e| {
                match e {
                    InvokeError::Error(e) => RuntimeError::ApplicationError(ApplicationError::ResourceManagerError(e)),
                    InvokeError::Downstream(runtime_error) => runtime_error,
                }
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
                    return Err(RuntimeError::ApplicationError(ApplicationError::ResourceManagerError(
                        ResourceManagerError::NonFungibleAlreadyExists(
                            NonFungibleAddress::new(resource_address, id),
                        ),
                    )));
                }

                *non_fungible_mut = NonFungibleSubstate(Some(non_fungible));
            }

            system_api.drop_lock(non_fungible_handle)?;
        }

        Ok((scrypto::resource::Bucket(bucket_id), CallFrameUpdate::move_node(RENodeId::Bucket(bucket_id))))
    }
}

impl NativeInvocation for ResourceManagerMintInput {
    fn info(&self) -> NativeInvocationInfo {
        NativeInvocationInfo::Method(
            NativeMethod::ResourceManager(ResourceManagerMethod::Mint),
            RENodeId::Global(GlobalAddress::Resource(self.resource_address)),
            CallFrameUpdate::empty(),
        )
    }
}

impl NativeExecutable for ResourceManagerGetMetadataInput {
    type Output = HashMap<String, String>;

    fn execute<'s, 'a, Y, R>(
        _input: Self,
        system_api: &mut Y,
    ) -> Result<(HashMap<String, String>, CallFrameUpdate), RuntimeError>
        where
            Y: SystemApi<'s, R> + InvokableNative<'a>,
            R: FeeReserve,
    {
        // TODO: Remove this hack and get resolved receiver in a better way
        let node_id = match system_api.get_actor() {
            REActor::Method(_, ResolvedReceiver { receiver, .. }) => *receiver,
            _ => panic!("Unexpected"),
        };
        let offset = SubstateOffset::ResourceManager(ResourceManagerOffset::ResourceManager);
        let resman_handle = system_api.lock_substate(node_id, offset, LockFlags::read_only())?;

        let substate_ref = system_api.get_ref(resman_handle)?;
        let metadata = &substate_ref.resource_manager().metadata;

        Ok((metadata.clone(), CallFrameUpdate::empty()))
    }
}

impl NativeInvocation for ResourceManagerGetMetadataInput {
    fn info(&self) -> NativeInvocationInfo {
        NativeInvocationInfo::Method(
            NativeMethod::ResourceManager(ResourceManagerMethod::CreateBucket),
            RENodeId::Global(GlobalAddress::Resource(self.resource_address)),
            CallFrameUpdate::empty(),
        )
    }
}

impl NativeExecutable for ResourceManagerGetResourceTypeInput {
    type Output = ResourceType;

    fn execute<'s, 'a, Y, R>(
        _input: Self,
        system_api: &mut Y,
    ) -> Result<(ResourceType, CallFrameUpdate), RuntimeError>
        where
            Y: SystemApi<'s, R> + InvokableNative<'a>,
            R: FeeReserve,
    {
        // TODO: Remove this hack and get resolved receiver in a better way
        let node_id = match system_api.get_actor() {
            REActor::Method(_, ResolvedReceiver { receiver, .. }) => *receiver,
            _ => panic!("Unexpected"),
        };
        let offset = SubstateOffset::ResourceManager(ResourceManagerOffset::ResourceManager);
        let resman_handle = system_api.lock_substate(node_id, offset, LockFlags::read_only())?;

        let substate_ref = system_api.get_ref(resman_handle)?;
        let resource_type = substate_ref.resource_manager().resource_type;

        Ok((resource_type, CallFrameUpdate::empty()))
    }
}

impl NativeInvocation for ResourceManagerGetResourceTypeInput {
    fn info(&self) -> NativeInvocationInfo {
        NativeInvocationInfo::Method(
            NativeMethod::ResourceManager(ResourceManagerMethod::GetResourceType),
            RENodeId::Global(GlobalAddress::Resource(self.resource_address)),
            CallFrameUpdate::empty(),
        )
    }
}

impl NativeExecutable for ResourceManagerGetTotalSupplyInput {
    type Output = Decimal;

    fn execute<'s, 'a, Y, R>(
        _input: Self,
        system_api: &mut Y,
    ) -> Result<(Decimal, CallFrameUpdate), RuntimeError>
        where
            Y: SystemApi<'s, R> + InvokableNative<'a>,
            R: FeeReserve,
    {
        // TODO: Remove this hack and get resolved receiver in a better way
        let node_id = match system_api.get_actor() {
            REActor::Method(_, ResolvedReceiver { receiver, .. }) => *receiver,
            _ => panic!("Unexpected"),
        };
        let offset = SubstateOffset::ResourceManager(ResourceManagerOffset::ResourceManager);
        let resman_handle = system_api.lock_substate(node_id, offset, LockFlags::read_only())?;
        let substate_ref = system_api.get_ref(resman_handle)?;
        let total_supply = substate_ref.resource_manager().total_supply;

        Ok((total_supply, CallFrameUpdate::empty()))
    }
}

impl NativeInvocation for ResourceManagerGetTotalSupplyInput {
    fn info(&self) -> NativeInvocationInfo {
        NativeInvocationInfo::Method(
            NativeMethod::ResourceManager(ResourceManagerMethod::GetTotalSupply),
            RENodeId::Global(GlobalAddress::Resource(self.resource_address)),
            CallFrameUpdate::empty(),
        )
    }
}


impl NativeExecutable for ResourceManagerUpdateMetadataInput {
    type Output = ();

    fn execute<'s, 'a, Y, R>(
        input: Self,
        system_api: &mut Y,
    ) -> Result<((), CallFrameUpdate), RuntimeError>
        where
            Y: SystemApi<'s, R> + InvokableNative<'a>,
            R: FeeReserve,
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
            .update_metadata(input.metadata)
            .map_err(|e| {
                match e {
                    InvokeError::Error(e) => RuntimeError::ApplicationError(ApplicationError::ResourceManagerError(e)),
                    InvokeError::Downstream(runtime_error) => runtime_error,
                }
            })?;

        Ok(((), CallFrameUpdate::empty()))
    }
}

impl NativeInvocation for ResourceManagerUpdateMetadataInput {
    fn info(&self) -> NativeInvocationInfo {
        NativeInvocationInfo::Method(
            NativeMethod::ResourceManager(ResourceManagerMethod::UpdateMetadata),
            RENodeId::Global(GlobalAddress::Resource(self.resource_address)),
            CallFrameUpdate::empty(),
        )
    }
}

impl NativeExecutable for ResourceManagerUpdateNonFungibleDataInput {
    type Output = ();

    fn execute<'s, 'a, Y, R>(
        input: Self,
        system_api: &mut Y,
    ) -> Result<((), CallFrameUpdate), RuntimeError>
        where
            Y: SystemApi<'s, R> + InvokableNative<'a>,
            R: FeeReserve,
    {
        // TODO: Remove this hack and get resolved receiver in a better way
        let node_id = match system_api.get_actor() {
            REActor::Method(_, ResolvedReceiver { receiver, .. }) => *receiver,
            _ => panic!("Unexpected"),
        };
        let offset = SubstateOffset::ResourceManager(ResourceManagerOffset::ResourceManager);
        let resman_handle = system_api.lock_substate(node_id, offset, LockFlags::MUTABLE)?;

        let substate_ref = system_api.get_ref(resman_handle)?;
        let resource_manager = substate_ref.resource_manager();
        let nf_store_id = resource_manager
            .nf_store_id
            .ok_or(InvokeError::Error(ResourceManagerError::NotNonFungible))
            .map_err(|e| {
                match e {
                    InvokeError::Error(e) => RuntimeError::ApplicationError(ApplicationError::ResourceManagerError(e)),
                    InvokeError::Downstream(runtime_error) => runtime_error,
                }
            })?;
        let resource_address = resource_manager.resource_address.unwrap();

        let node_id = RENodeId::NonFungibleStore(nf_store_id);
        let offset = SubstateOffset::NonFungibleStore(NonFungibleStoreOffset::Entry(
            input.id.clone(),
        ));

        let non_fungible_handle =
            system_api.lock_substate(node_id, offset, LockFlags::MUTABLE)?;
        let mut substate_mut = system_api.get_ref_mut(non_fungible_handle)?;
        let non_fungible_mut = substate_mut.non_fungible();
        if let Some(ref mut non_fungible) = non_fungible_mut.0 {
            non_fungible.set_mutable_data(input.data);
        } else {
            let non_fungible_address = NonFungibleAddress::new(resource_address, input.id);
            return Err(RuntimeError::ApplicationError(ApplicationError::ResourceManagerError(
                ResourceManagerError::NonFungibleNotFound(non_fungible_address),
            )));
        }

        system_api.drop_lock(non_fungible_handle)?;


        Ok(((), CallFrameUpdate::empty()))
    }
}

impl NativeInvocation for ResourceManagerUpdateNonFungibleDataInput {
    fn info(&self) -> NativeInvocationInfo {
        NativeInvocationInfo::Method(
            NativeMethod::ResourceManager(ResourceManagerMethod::UpdateNonFungibleData),
            RENodeId::Global(GlobalAddress::Resource(self.resource_address)),
            CallFrameUpdate::empty(),
        )
    }
}


pub struct ResourceManager;

impl ResourceManager {
    pub fn new(
        resource_type: ResourceType,
        metadata: HashMap<String, String>,
        mut auth: HashMap<ResourceMethodAuthKey, (AccessRule, Mutability)>,
        nf_store_id: Option<NonFungibleStoreId>,
    ) -> Result<ResourceManagerSubstate, InvokeError<ResourceManagerError>> {
        let mut vault_method_table: HashMap<VaultMethod, ResourceMethodRule> = HashMap::new();
        vault_method_table.insert(VaultMethod::LockFee, Protected(Withdraw));
        vault_method_table.insert(VaultMethod::Take, Protected(Withdraw));
        vault_method_table.insert(VaultMethod::Put, Protected(Deposit));
        vault_method_table.insert(VaultMethod::GetAmount, Public);
        vault_method_table.insert(VaultMethod::GetResourceAddress, Public);
        vault_method_table.insert(VaultMethod::GetNonFungibleIds, Public);
        vault_method_table.insert(VaultMethod::CreateProof, Public);
        vault_method_table.insert(VaultMethod::CreateProofByAmount, Public);
        vault_method_table.insert(VaultMethod::CreateProofByIds, Public);
        vault_method_table.insert(VaultMethod::TakeNonFungibles, Protected(Withdraw));

        let bucket_method_table: HashMap<BucketMethod, ResourceMethodRule> = HashMap::new();

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
        method_table.insert(ResourceManagerMethod::Burn, Protected(Burn));
        method_table.insert(ResourceManagerMethod::SetResourceAddress, Public);

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

        let resource_manager = ResourceManagerSubstate {
            resource_type,
            metadata,
            method_table,
            vault_method_table,
            bucket_method_table,
            authorization,
            total_supply: 0.into(),
            nf_store_id,
            resource_address: None,
        };

        Ok(resource_manager)
    }

    fn method_lock_flags(method: ResourceManagerMethod) -> LockFlags {
        match method {
            ResourceManagerMethod::Burn => LockFlags::MUTABLE,
            ResourceManagerMethod::UpdateAuth => LockFlags::MUTABLE,
            ResourceManagerMethod::LockAuth => LockFlags::MUTABLE,
            ResourceManagerMethod::Mint => LockFlags::MUTABLE,
            ResourceManagerMethod::UpdateNonFungibleData => LockFlags::MUTABLE,
            ResourceManagerMethod::GetNonFungible => LockFlags::read_only(),
            ResourceManagerMethod::GetMetadata => LockFlags::read_only(),
            ResourceManagerMethod::GetResourceType => LockFlags::read_only(),
            ResourceManagerMethod::GetTotalSupply => LockFlags::read_only(),
            ResourceManagerMethod::UpdateMetadata => LockFlags::MUTABLE,
            ResourceManagerMethod::NonFungibleExists => LockFlags::read_only(),
            ResourceManagerMethod::CreateBucket => LockFlags::MUTABLE,
            ResourceManagerMethod::CreateVault => LockFlags::MUTABLE,
            ResourceManagerMethod::SetResourceAddress => LockFlags::MUTABLE,
        }
    }

    pub fn main<'s, Y, R>(
        resource_manager_id: ResourceManagerId,
        method: ResourceManagerMethod,
        args: ScryptoValue,
        system_api: &mut Y,
    ) -> Result<ScryptoValue, InvokeError<ResourceManagerError>>
    where
        Y: SystemApi<'s, R> + Invokable<NativeMethodInvocation>,
        R: FeeReserve,
    {
        let node_id = RENodeId::ResourceManager(resource_manager_id);
        let offset = SubstateOffset::ResourceManager(ResourceManagerOffset::ResourceManager);
        let resman_handle =
            system_api.lock_substate(node_id, offset, Self::method_lock_flags(method))?;

        let rtn = match method {
            ResourceManagerMethod::Burn => {
                panic!("Unexpected")
            }
            ResourceManagerMethod::UpdateAuth => {
                panic!("Unexpected")
            }
            ResourceManagerMethod::LockAuth => {
                panic!("Unexpected")
            }
            ResourceManagerMethod::CreateVault => {
                panic!("Unexpected")
            }
            ResourceManagerMethod::CreateBucket => {
                panic!("Unexpected")
            }
            ResourceManagerMethod::Mint => {
                panic!("Unexpected")
            }
            ResourceManagerMethod::GetMetadata => {
                panic!("Unexpected")
            }
            ResourceManagerMethod::GetResourceType => {
                panic!("Unexpected")
            }
            ResourceManagerMethod::GetTotalSupply => {
                panic!("Unexpected")
            }
            ResourceManagerMethod::UpdateMetadata => {
                panic!("Unexpected")
            }
            ResourceManagerMethod::UpdateNonFungibleData => {
                panic!("Unexpected")
            }
            ResourceManagerMethod::NonFungibleExists => {
                let input: ResourceManagerNonFungibleExistsInput = scrypto_decode(&args.raw)
                    .map_err(|e| InvokeError::Error(ResourceManagerError::InvalidRequestData(e)))?;
                let substate_ref = system_api.get_ref(resman_handle)?;
                let resource_manager = substate_ref.resource_manager();
                let nf_store_id = resource_manager
                    .nf_store_id
                    .ok_or(InvokeError::Error(ResourceManagerError::NotNonFungible))?;

                let node_id = RENodeId::NonFungibleStore(nf_store_id);
                let offset =
                    SubstateOffset::NonFungibleStore(NonFungibleStoreOffset::Entry(input.id));
                let non_fungible_handle =
                    system_api.lock_substate(node_id, offset, LockFlags::read_only())?;
                let substate = system_api.get_ref(non_fungible_handle)?;
                let exists = substate.non_fungible().0.is_some();

                ScryptoValue::from_typed(&exists)
            }
            ResourceManagerMethod::GetNonFungible => {
                let input: ResourceManagerGetNonFungibleInput = scrypto_decode(&args.raw)
                    .map_err(|e| InvokeError::Error(ResourceManagerError::InvalidRequestData(e)))?;
                let substate_ref = system_api.get_ref(resman_handle)?;
                let resource_manager = substate_ref.resource_manager();
                let nf_store_id = resource_manager
                    .nf_store_id
                    .ok_or(InvokeError::Error(ResourceManagerError::NotNonFungible))?;

                let non_fungible_address = NonFungibleAddress::new(
                    resource_manager.resource_address.unwrap(),
                    input.id.clone(),
                );

                let node_id = RENodeId::NonFungibleStore(nf_store_id);
                let offset =
                    SubstateOffset::NonFungibleStore(NonFungibleStoreOffset::Entry(input.id));
                let non_fungible_handle =
                    system_api.lock_substate(node_id, offset, LockFlags::read_only())?;
                let non_fungible_ref = system_api.get_ref(non_fungible_handle)?;
                let wrapper = non_fungible_ref.non_fungible();
                if let Some(non_fungible) = wrapper.0.as_ref() {
                    ScryptoValue::from_typed(&[
                        non_fungible.immutable_data(),
                        non_fungible.mutable_data(),
                    ])
                } else {
                    return Err(InvokeError::Error(
                        ResourceManagerError::NonFungibleNotFound(non_fungible_address),
                    ));
                }
            }
            ResourceManagerMethod::SetResourceAddress => {
                let input: ResourceManagerSetResourceAddressInput = scrypto_decode(&args.raw)
                    .map_err(|e| InvokeError::Error(ResourceManagerError::InvalidRequestData(e)))?;

                let mut substate_mut = system_api.get_ref_mut(resman_handle)?;
                substate_mut
                    .resource_manager()
                    .set_resource_address(input.address)?;

                ScryptoValue::from_typed(&())
            }
        };

        Ok(rtn)
    }
}
