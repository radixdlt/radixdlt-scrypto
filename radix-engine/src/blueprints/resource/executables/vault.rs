use crate::blueprints::resource::*;
use crate::errors::ApplicationError;
use crate::errors::RuntimeError;
use crate::kernel::actor::ResolvedActor;
use crate::kernel::actor::ResolvedReceiver;
use crate::kernel::call_frame::CallFrameUpdate;
use crate::kernel::kernel_api::{
    ExecutableInvocation, Executor, KernelNodeApi, KernelSubstateApi, LockFlags,
};
use crate::system::kernel_modules::costing::CostingError;
use crate::system::node::RENodeInit;
use crate::types::*;
use crate::wasm::WasmEngine;
use radix_engine_interface::api::types::*;
use radix_engine_interface::api::types::{
    GlobalAddress, NativeFn, RENodeId, SubstateOffset, VaultFn, VaultOffset,
};
use radix_engine_interface::api::ClientDerefApi;
use radix_engine_interface::api::ClientEventApi;
use radix_engine_interface::api::ClientNativeInvokeApi;
use radix_engine_interface::blueprints::resource::*;

#[derive(Debug, Clone, PartialEq, Eq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub enum VaultError {
    InvalidRequestData(DecodeError),
    ResourceOperationError(ResourceOperationError),
    CouldNotCreateBucket,
    CouldNotTakeBucket,
    ProofError(ProofError),
    CouldNotCreateProof,
    LockFeeNotRadixToken,
    LockFeeInsufficientBalance,
    LockFeeRepayFailure(CostingError),
}

impl ExecutableInvocation for VaultRecallInvocation {
    type Exec = VaultTakeInvocation;

    fn resolve<D: ClientDerefApi<RuntimeError>>(
        self,
        _api: &mut D,
    ) -> Result<(ResolvedActor, CallFrameUpdate, Self::Exec), RuntimeError> {
        let receiver = RENodeId::Vault(self.receiver);
        let call_frame_update = CallFrameUpdate::copy_ref(receiver);
        let actor = ResolvedActor::method(
            NativeFn::Vault(VaultFn::Recall),
            ResolvedReceiver::new(receiver),
        );
        let executor = VaultTakeInvocation {
            receiver: self.receiver,
            amount: self.amount,
        };
        Ok((actor, call_frame_update, executor))
    }
}

impl ExecutableInvocation for VaultTakeInvocation {
    type Exec = Self;

    fn resolve<D: ClientDerefApi<RuntimeError>>(
        self,
        _api: &mut D,
    ) -> Result<(ResolvedActor, CallFrameUpdate, Self::Exec), RuntimeError> {
        let receiver = RENodeId::Vault(self.receiver);
        let call_frame_update = CallFrameUpdate::copy_ref(receiver);
        let actor = ResolvedActor::method(
            NativeFn::Vault(VaultFn::Take),
            ResolvedReceiver::new(receiver),
        );
        Ok((actor, call_frame_update, self))
    }
}

impl Executor for VaultTakeInvocation {
    type Output = Bucket;

    fn execute<'a, Y, W: WasmEngine>(
        self,
        api: &mut Y,
    ) -> Result<(Bucket, CallFrameUpdate), RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi,
    {
        let offset = SubstateOffset::Vault(VaultOffset::Vault);
        let vault_handle = api.lock_substate(
            RENodeId::Vault(self.receiver),
            NodeModuleId::SELF,
            offset,
            LockFlags::MUTABLE,
        )?;

        let container = {
            let mut substate_mut = api.get_ref_mut(vault_handle)?;
            let vault = substate_mut.vault();
            vault.take(self.amount)?
        };

        let node_id = api.allocate_node_id(RENodeType::Bucket)?;
        api.create_node(
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

impl ExecutableInvocation for VaultPutInvocation {
    type Exec = Self;

    fn resolve<D: ClientDerefApi<RuntimeError>>(
        self,
        _api: &mut D,
    ) -> Result<(ResolvedActor, CallFrameUpdate, Self::Exec), RuntimeError> {
        let receiver = RENodeId::Vault(self.receiver);
        let mut call_frame_update = CallFrameUpdate::copy_ref(receiver);
        call_frame_update
            .nodes_to_move
            .push(RENodeId::Bucket(self.bucket.0));
        let actor = ResolvedActor::method(
            NativeFn::Vault(VaultFn::Put),
            ResolvedReceiver::new(receiver),
        );
        Ok((actor, call_frame_update, self))
    }
}

impl Executor for VaultPutInvocation {
    type Output = ();

    fn execute<'a, Y, W: WasmEngine>(
        self,
        api: &mut Y,
    ) -> Result<((), CallFrameUpdate), RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi,
    {
        let node_id = RENodeId::Vault(self.receiver);
        let offset = SubstateOffset::Vault(VaultOffset::Vault);
        let vault_handle =
            api.lock_substate(node_id, NodeModuleId::SELF, offset, LockFlags::MUTABLE)?;

        let bucket = api.drop_node(RENodeId::Bucket(self.bucket.0))?.into();

        let mut substate_mut = api.get_ref_mut(vault_handle)?;
        let vault = substate_mut.vault();
        vault.put(bucket).map_err(|e| {
            RuntimeError::ApplicationError(ApplicationError::VaultError(
                VaultError::ResourceOperationError(e),
            ))
        })?;

        Ok(((), CallFrameUpdate::empty()))
    }
}

impl ExecutableInvocation for VaultLockFeeInvocation {
    type Exec = Self;

    fn resolve<D: ClientDerefApi<RuntimeError>>(
        self,
        _api: &mut D,
    ) -> Result<(ResolvedActor, CallFrameUpdate, Self::Exec), RuntimeError> {
        let receiver = RENodeId::Vault(self.receiver);
        let call_frame_update = CallFrameUpdate::copy_ref(receiver);
        let actor = ResolvedActor::method(
            NativeFn::Vault(VaultFn::LockFee),
            ResolvedReceiver::new(receiver),
        );
        Ok((actor, call_frame_update, self))
    }
}

impl Executor for VaultLockFeeInvocation {
    type Output = ();

    fn execute<'a, Y, W: WasmEngine>(
        self,
        api: &mut Y,
    ) -> Result<((), CallFrameUpdate), RuntimeError>
    where
        Y: KernelNodeApi
            + KernelSubstateApi
            + ClientNativeInvokeApi<RuntimeError>
            + ClientEventApi<RuntimeError>,
    {
        let node_id = RENodeId::Vault(self.receiver);
        let offset = SubstateOffset::Vault(VaultOffset::Vault);
        let vault_handle = api.lock_substate(
            node_id,
            NodeModuleId::SELF,
            offset,
            LockFlags::MUTABLE | LockFlags::UNMODIFIED_BASE | LockFlags::FORCE_WRITE,
        )?;

        // Take by amount
        let fee = {
            let mut substate_mut = api.get_ref_mut(vault_handle)?;
            let vault = substate_mut.vault();

            // Check resource and take amount
            if vault.resource_address() != RADIX_TOKEN {
                return Err(RuntimeError::ApplicationError(
                    ApplicationError::VaultError(VaultError::LockFeeNotRadixToken),
                ));
            }

            // Take fee from the vault
            vault.take(self.amount).map_err(|_| {
                RuntimeError::ApplicationError(ApplicationError::VaultError(
                    VaultError::LockFeeInsufficientBalance,
                ))
            })?
        };

        // Credit cost units
        let changes: Resource = api.credit_cost_units(self.receiver, fee, self.contingent)?;

        // Keep changes
        {
            let mut substate_mut = api.get_ref_mut(vault_handle)?;
            let vault = substate_mut.vault();
            vault.put(BucketSubstate::new(changes)).map_err(|e| {
                RuntimeError::ApplicationError(ApplicationError::VaultError(
                    VaultError::ResourceOperationError(e),
                ))
            })?;
        }

        Ok(((), CallFrameUpdate::empty()))
    }
}

impl ExecutableInvocation for VaultRecallNonFungiblesInvocation {
    type Exec = VaultTakeNonFungiblesInvocation;

    fn resolve<D: ClientDerefApi<RuntimeError>>(
        self,
        _api: &mut D,
    ) -> Result<(ResolvedActor, CallFrameUpdate, Self::Exec), RuntimeError> {
        let receiver = RENodeId::Vault(self.receiver);
        let call_frame_update = CallFrameUpdate::copy_ref(receiver);
        let actor = ResolvedActor::method(
            NativeFn::Vault(VaultFn::RecallNonFungibles),
            ResolvedReceiver::new(receiver),
        );
        let executor = VaultTakeNonFungiblesInvocation {
            receiver: self.receiver,
            non_fungible_local_ids: self.non_fungible_local_ids,
        };
        Ok((actor, call_frame_update, executor))
    }
}

impl ExecutableInvocation for VaultTakeNonFungiblesInvocation {
    type Exec = Self;

    fn resolve<D: ClientDerefApi<RuntimeError>>(
        self,
        _api: &mut D,
    ) -> Result<(ResolvedActor, CallFrameUpdate, Self::Exec), RuntimeError> {
        let receiver = RENodeId::Vault(self.receiver);
        let call_frame_update = CallFrameUpdate::copy_ref(receiver);
        let actor = ResolvedActor::method(
            NativeFn::Vault(VaultFn::TakeNonFungibles),
            ResolvedReceiver::new(receiver),
        );
        Ok((actor, call_frame_update, self))
    }
}

impl Executor for VaultTakeNonFungiblesInvocation {
    type Output = Bucket;

    fn execute<'a, Y, W: WasmEngine>(
        self,
        api: &mut Y,
    ) -> Result<(Bucket, CallFrameUpdate), RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi,
    {
        let node_id = RENodeId::Vault(self.receiver);
        let offset = SubstateOffset::Vault(VaultOffset::Vault);
        let vault_handle =
            api.lock_substate(node_id, NodeModuleId::SELF, offset, LockFlags::MUTABLE)?;

        let container = {
            let mut substate_mut = api.get_ref_mut(vault_handle)?;
            let vault = substate_mut.vault();
            vault.take_non_fungibles(&self.non_fungible_local_ids)?
        };

        let node_id = api.allocate_node_id(RENodeType::Bucket)?;
        api.create_node(
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

impl ExecutableInvocation for VaultGetAmountInvocation {
    type Exec = Self;

    fn resolve<D: ClientDerefApi<RuntimeError>>(
        self,
        _api: &mut D,
    ) -> Result<(ResolvedActor, CallFrameUpdate, Self::Exec), RuntimeError> {
        let receiver = RENodeId::Vault(self.receiver);
        let call_frame_update = CallFrameUpdate::copy_ref(receiver);
        let actor = ResolvedActor::method(
            NativeFn::Vault(VaultFn::GetAmount),
            ResolvedReceiver::new(receiver),
        );
        Ok((actor, call_frame_update, self))
    }
}

impl Executor for VaultGetAmountInvocation {
    type Output = Decimal;

    fn execute<'a, Y, W: WasmEngine>(
        self,
        api: &mut Y,
    ) -> Result<(Decimal, CallFrameUpdate), RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi,
    {
        let node_id = RENodeId::Vault(self.receiver);
        let offset = SubstateOffset::Vault(VaultOffset::Vault);
        let vault_handle =
            api.lock_substate(node_id, NodeModuleId::SELF, offset, LockFlags::read_only())?;

        let substate_ref = api.get_ref(vault_handle)?;
        let vault = substate_ref.vault();
        let amount = vault.total_amount();

        Ok((amount, CallFrameUpdate::empty()))
    }
}

impl ExecutableInvocation for VaultGetResourceAddressInvocation {
    type Exec = Self;

    fn resolve<D: ClientDerefApi<RuntimeError>>(
        self,
        _api: &mut D,
    ) -> Result<(ResolvedActor, CallFrameUpdate, Self::Exec), RuntimeError> {
        let receiver = RENodeId::Vault(self.receiver);
        let call_frame_update = CallFrameUpdate::copy_ref(receiver);
        let actor = ResolvedActor::method(
            NativeFn::Vault(VaultFn::GetResourceAddress),
            ResolvedReceiver::new(receiver),
        );
        Ok((actor, call_frame_update, self))
    }
}

impl Executor for VaultGetResourceAddressInvocation {
    type Output = ResourceAddress;

    fn execute<'a, Y, W: WasmEngine>(
        self,
        api: &mut Y,
    ) -> Result<(ResourceAddress, CallFrameUpdate), RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi,
    {
        let node_id = RENodeId::Vault(self.receiver);
        let offset = SubstateOffset::Vault(VaultOffset::Vault);
        let vault_handle =
            api.lock_substate(node_id, NodeModuleId::SELF, offset, LockFlags::read_only())?;

        let substate_ref = api.get_ref(vault_handle)?;
        let vault = substate_ref.vault();
        let resource_address = vault.resource_address();

        Ok((
            resource_address,
            CallFrameUpdate::copy_ref(RENodeId::Global(GlobalAddress::Resource(resource_address))),
        ))
    }
}

impl ExecutableInvocation for VaultGetNonFungibleLocalIdsInvocation {
    type Exec = Self;

    fn resolve<D: ClientDerefApi<RuntimeError>>(
        self,
        _api: &mut D,
    ) -> Result<(ResolvedActor, CallFrameUpdate, Self::Exec), RuntimeError> {
        let receiver = RENodeId::Vault(self.receiver);
        let call_frame_update = CallFrameUpdate::copy_ref(receiver);
        let actor = ResolvedActor::method(
            NativeFn::Vault(VaultFn::GetNonFungibleLocalIds),
            ResolvedReceiver::new(receiver),
        );
        Ok((actor, call_frame_update, self))
    }
}

impl Executor for VaultGetNonFungibleLocalIdsInvocation {
    type Output = BTreeSet<NonFungibleLocalId>;

    fn execute<'a, Y, W: WasmEngine>(
        self,
        api: &mut Y,
    ) -> Result<(BTreeSet<NonFungibleLocalId>, CallFrameUpdate), RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi,
    {
        let node_id = RENodeId::Vault(self.receiver);
        let offset = SubstateOffset::Vault(VaultOffset::Vault);
        let vault_handle =
            api.lock_substate(node_id, NodeModuleId::SELF, offset, LockFlags::read_only())?;

        let substate_ref = api.get_ref(vault_handle)?;
        let vault = substate_ref.vault();
        let ids = vault.total_ids().map_err(|e| {
            RuntimeError::ApplicationError(ApplicationError::VaultError(
                VaultError::ResourceOperationError(e),
            ))
        })?;

        Ok((ids, CallFrameUpdate::empty()))
    }
}

impl ExecutableInvocation for VaultCreateProofInvocation {
    type Exec = Self;

    fn resolve<D: ClientDerefApi<RuntimeError>>(
        self,
        _api: &mut D,
    ) -> Result<(ResolvedActor, CallFrameUpdate, Self::Exec), RuntimeError> {
        let receiver = RENodeId::Vault(self.receiver);
        let call_frame_update = CallFrameUpdate::copy_ref(receiver);
        let actor = ResolvedActor::method(
            NativeFn::Vault(VaultFn::CreateProof),
            ResolvedReceiver::new(receiver),
        );
        Ok((actor, call_frame_update, self))
    }
}

impl Executor for VaultCreateProofInvocation {
    type Output = Proof;

    fn execute<'a, Y, W: WasmEngine>(
        self,
        api: &mut Y,
    ) -> Result<(Proof, CallFrameUpdate), RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi,
    {
        let node_id = RENodeId::Vault(self.receiver);
        let offset = SubstateOffset::Vault(VaultOffset::Vault);
        let vault_handle =
            api.lock_substate(node_id, NodeModuleId::SELF, offset, LockFlags::MUTABLE)?;

        let proof = {
            let mut substate_mut = api.get_ref_mut(vault_handle)?;
            let vault = substate_mut.vault();
            vault
                .create_proof(ResourceContainerId::Vault(self.receiver))
                .map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::VaultError(
                        VaultError::ProofError(e),
                    ))
                })?
        };

        let node_id = api.allocate_node_id(RENodeType::Proof)?;
        api.create_node(node_id, RENodeInit::Proof(proof), BTreeMap::new())?;
        let proof_id = node_id.into();

        Ok((
            Proof(proof_id),
            CallFrameUpdate::move_node(RENodeId::Proof(proof_id)),
        ))
    }
}

impl ExecutableInvocation for VaultCreateProofByAmountInvocation {
    type Exec = Self;

    fn resolve<D: ClientDerefApi<RuntimeError>>(
        self,
        _api: &mut D,
    ) -> Result<(ResolvedActor, CallFrameUpdate, Self::Exec), RuntimeError> {
        let receiver = RENodeId::Vault(self.receiver);
        let call_frame_update = CallFrameUpdate::copy_ref(receiver);
        let actor = ResolvedActor::method(
            NativeFn::Vault(VaultFn::CreateProofByAmount),
            ResolvedReceiver::new(receiver),
        );
        Ok((actor, call_frame_update, self))
    }
}

impl Executor for VaultCreateProofByAmountInvocation {
    type Output = Proof;

    fn execute<'a, Y, W: WasmEngine>(
        self,
        api: &mut Y,
    ) -> Result<(Proof, CallFrameUpdate), RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi,
    {
        let node_id = RENodeId::Vault(self.receiver);
        let offset = SubstateOffset::Vault(VaultOffset::Vault);
        let vault_handle =
            api.lock_substate(node_id, NodeModuleId::SELF, offset, LockFlags::MUTABLE)?;

        let proof = {
            let mut substate_mut = api.get_ref_mut(vault_handle)?;
            let vault = substate_mut.vault();
            vault
                .create_proof_by_amount(self.amount, ResourceContainerId::Vault(self.receiver))
                .map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::VaultError(
                        VaultError::ProofError(e),
                    ))
                })?
        };

        let node_id = api.allocate_node_id(RENodeType::Proof)?;
        api.create_node(node_id, RENodeInit::Proof(proof), BTreeMap::new())?;
        let proof_id = node_id.into();

        Ok((
            Proof(proof_id),
            CallFrameUpdate::move_node(RENodeId::Proof(proof_id)),
        ))
    }
}

impl ExecutableInvocation for VaultCreateProofByIdsInvocation {
    type Exec = Self;

    fn resolve<D: ClientDerefApi<RuntimeError>>(
        self,
        _api: &mut D,
    ) -> Result<(ResolvedActor, CallFrameUpdate, Self::Exec), RuntimeError> {
        let receiver = RENodeId::Vault(self.receiver);
        let call_frame_update = CallFrameUpdate::copy_ref(receiver);
        let actor = ResolvedActor::method(
            NativeFn::Vault(VaultFn::CreateProofByIds),
            ResolvedReceiver::new(receiver),
        );
        Ok((actor, call_frame_update, self))
    }
}

impl Executor for VaultCreateProofByIdsInvocation {
    type Output = Proof;

    fn execute<'a, Y, W: WasmEngine>(
        self,
        api: &mut Y,
    ) -> Result<(Proof, CallFrameUpdate), RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi,
    {
        let node_id = RENodeId::Vault(self.receiver);
        let offset = SubstateOffset::Vault(VaultOffset::Vault);
        let vault_handle =
            api.lock_substate(node_id, NodeModuleId::SELF, offset, LockFlags::MUTABLE)?;

        let proof = {
            let mut substate_mut = api.get_ref_mut(vault_handle)?;
            let vault = substate_mut.vault();
            vault
                .create_proof_by_ids(&self.ids, ResourceContainerId::Vault(self.receiver))
                .map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::VaultError(
                        VaultError::ProofError(e),
                    ))
                })?
        };

        let node_id = api.allocate_node_id(RENodeType::Proof)?;
        api.create_node(node_id, RENodeInit::Proof(proof), BTreeMap::new())?;
        let proof_id = node_id.into();

        Ok((
            Proof(proof_id),
            CallFrameUpdate::move_node(RENodeId::Proof(proof_id)),
        ))
    }
}
