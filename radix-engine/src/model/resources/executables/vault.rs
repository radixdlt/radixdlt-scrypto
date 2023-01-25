use crate::engine::{
    ApplicationError, CallFrameUpdate, ExecutableInvocation, Executor, LockFlags, RENodeInit,
    ResolvedActor, ResolvedReceiver, ResolverApi, RuntimeError, SystemApi,
};
use crate::fee::FeeReserveError;
use crate::model::{BucketSubstate, ProofError, ResourceContainerId, ResourceOperationError};
use crate::types::*;
use crate::wasm::WasmEngine;
use radix_engine_interface::api::types::{
    GlobalAddress, NativeFn, RENodeId, SubstateOffset, VaultFn, VaultOffset,
};
use radix_engine_interface::model::*;

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
    LockFeeRepayFailure(FeeReserveError),
}

impl ExecutableInvocation for VaultRecallInvocation {
    type Exec = VaultTakeInvocation;

    fn resolve<D: ResolverApi>(
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

    fn resolve<D: ResolverApi>(
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
        Y: SystemApi,
    {
        let offset = SubstateOffset::Vault(VaultOffset::Vault);
        let vault_handle =
            api.lock_substate(RENodeId::Vault(self.receiver), offset, LockFlags::MUTABLE)?;

        let container = {
            let mut substate_mut = api.get_ref_mut(vault_handle)?;
            let vault = substate_mut.vault();
            vault.take(self.amount)?
        };

        let node_id = api.allocate_node_id(RENodeType::Bucket)?;
        api.create_node(node_id, RENodeInit::Bucket(BucketSubstate::new(container)))?;
        let bucket_id = node_id.into();

        Ok((
            Bucket(bucket_id),
            CallFrameUpdate::move_node(RENodeId::Bucket(bucket_id)),
        ))
    }
}

impl ExecutableInvocation for VaultPutInvocation {
    type Exec = Self;

    fn resolve<D: ResolverApi>(
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
        system_api: &mut Y,
    ) -> Result<((), CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi,
    {
        let node_id = RENodeId::Vault(self.receiver);
        let offset = SubstateOffset::Vault(VaultOffset::Vault);
        let vault_handle = system_api.lock_substate(node_id, offset, LockFlags::MUTABLE)?;

        let bucket = system_api
            .drop_node(RENodeId::Bucket(self.bucket.0))?
            .into();

        let mut substate_mut = system_api.get_ref_mut(vault_handle)?;
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

    fn resolve<D: ResolverApi>(
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
        system_api: &mut Y,
    ) -> Result<((), CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi,
    {
        let node_id = RENodeId::Vault(self.receiver);
        let offset = SubstateOffset::Vault(VaultOffset::Vault);
        let vault_handle = system_api.lock_substate(
            node_id,
            offset,
            LockFlags::MUTABLE | LockFlags::UNMODIFIED_BASE | LockFlags::FORCE_WRITE,
        )?;

        let fee = {
            let mut substate_mut = system_api.get_ref_mut(vault_handle)?;
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

        // Refill fee reserve
        let changes = system_api.lock_fee(self.receiver, fee, self.contingent)?;

        // Return changes
        {
            let mut substate_mut = system_api.get_ref_mut(vault_handle)?;
            let vault = substate_mut.vault();
            vault
                .borrow_resource_mut()
                .put(changes)
                .expect("Failed to return fee changes to a locking-fee vault");
        }

        Ok(((), CallFrameUpdate::empty()))
    }
}

impl ExecutableInvocation for VaultRecallNonFungiblesInvocation {
    type Exec = VaultTakeNonFungiblesInvocation;

    fn resolve<D: ResolverApi>(
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

    fn resolve<D: ResolverApi>(
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
        Y: SystemApi,
    {
        let node_id = RENodeId::Vault(self.receiver);
        let offset = SubstateOffset::Vault(VaultOffset::Vault);
        let vault_handle = api.lock_substate(node_id, offset, LockFlags::MUTABLE)?;

        let container = {
            let mut substate_mut = api.get_ref_mut(vault_handle)?;
            let vault = substate_mut.vault();
            vault.take_non_fungibles(&self.non_fungible_local_ids)?
        };

        let node_id = api.allocate_node_id(RENodeType::Bucket)?;
        api.create_node(node_id, RENodeInit::Bucket(BucketSubstate::new(container)))?;
        let bucket_id = node_id.into();

        Ok((
            Bucket(bucket_id),
            CallFrameUpdate::move_node(RENodeId::Bucket(bucket_id)),
        ))
    }
}

impl ExecutableInvocation for VaultGetAmountInvocation {
    type Exec = Self;

    fn resolve<D: ResolverApi>(
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
        system_api: &mut Y,
    ) -> Result<(Decimal, CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi,
    {
        let node_id = RENodeId::Vault(self.receiver);
        let offset = SubstateOffset::Vault(VaultOffset::Vault);
        let vault_handle = system_api.lock_substate(node_id, offset, LockFlags::read_only())?;

        let substate_ref = system_api.get_ref(vault_handle)?;
        let vault = substate_ref.vault();
        let amount = vault.total_amount();

        Ok((amount, CallFrameUpdate::empty()))
    }
}

impl ExecutableInvocation for VaultGetResourceAddressInvocation {
    type Exec = Self;

    fn resolve<D: ResolverApi>(
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
        system_api: &mut Y,
    ) -> Result<(ResourceAddress, CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi,
    {
        let node_id = RENodeId::Vault(self.receiver);
        let offset = SubstateOffset::Vault(VaultOffset::Vault);
        let vault_handle = system_api.lock_substate(node_id, offset, LockFlags::read_only())?;

        let substate_ref = system_api.get_ref(vault_handle)?;
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

    fn resolve<D: ResolverApi>(
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
        system_api: &mut Y,
    ) -> Result<(BTreeSet<NonFungibleLocalId>, CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi,
    {
        let node_id = RENodeId::Vault(self.receiver);
        let offset = SubstateOffset::Vault(VaultOffset::Vault);
        let vault_handle = system_api.lock_substate(node_id, offset, LockFlags::read_only())?;

        let substate_ref = system_api.get_ref(vault_handle)?;
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

    fn resolve<D: ResolverApi>(
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
        Y: SystemApi,
    {
        let node_id = RENodeId::Vault(self.receiver);
        let offset = SubstateOffset::Vault(VaultOffset::Vault);
        let vault_handle = api.lock_substate(node_id, offset, LockFlags::MUTABLE)?;

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
        api.create_node(node_id, RENodeInit::Proof(proof))?;
        let proof_id = node_id.into();

        Ok((
            Proof(proof_id),
            CallFrameUpdate::move_node(RENodeId::Proof(proof_id)),
        ))
    }
}

impl ExecutableInvocation for VaultCreateProofByAmountInvocation {
    type Exec = Self;

    fn resolve<D: ResolverApi>(
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
        Y: SystemApi,
    {
        let node_id = RENodeId::Vault(self.receiver);
        let offset = SubstateOffset::Vault(VaultOffset::Vault);
        let vault_handle = api.lock_substate(node_id, offset, LockFlags::MUTABLE)?;

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
        api.create_node(node_id, RENodeInit::Proof(proof))?;
        let proof_id = node_id.into();

        Ok((
            Proof(proof_id),
            CallFrameUpdate::move_node(RENodeId::Proof(proof_id)),
        ))
    }
}

impl ExecutableInvocation for VaultCreateProofByIdsInvocation {
    type Exec = Self;

    fn resolve<D: ResolverApi>(
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
        Y: SystemApi,
    {
        let node_id = RENodeId::Vault(self.receiver);
        let offset = SubstateOffset::Vault(VaultOffset::Vault);
        let vault_handle = api.lock_substate(node_id, offset, LockFlags::MUTABLE)?;

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
        api.create_node(node_id, RENodeInit::Proof(proof))?;
        let proof_id = node_id.into();

        Ok((
            Proof(proof_id),
            CallFrameUpdate::move_node(RENodeId::Proof(proof_id)),
        ))
    }
}
