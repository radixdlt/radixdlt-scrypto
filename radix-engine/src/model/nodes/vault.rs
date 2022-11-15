use radix_engine_lib::engine::types::{GlobalAddress, NativeMethod, RENodeId, SubstateOffset, VaultMethod, VaultOffset};
use radix_engine_lib::resource::{VaultCreateProofByAmountInvocation, VaultCreateProofByIdsInvocation, VaultCreateProofInvocation, VaultGetAmountInvocation, VaultGetNonFungibleIdsInvocation, VaultGetResourceAddressInvocation, VaultLockFeeInvocation, VaultPutInvocation, VaultTakeInvocation, VaultTakeNonFungiblesInvocation};
use crate::engine::{
    ApplicationError, CallFrameUpdate, InvokableNative, LockFlags, NativeExecutable,
    NativeInvocation, NativeInvocationInfo, RENode, RuntimeError, SystemApi,
};
use crate::fee::FeeReserveError;
use crate::model::{
    BucketSubstate, InvokeError, ProofError, ResourceContainerId, ResourceOperationError,
};
use crate::types::*;

#[derive(Debug, Clone, PartialEq, Eq, TypeId, Encode, Decode)]
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

impl NativeExecutable for VaultTakeInvocation {
    type NativeOutput = radix_engine_lib::resource::Bucket;

    fn execute<'a, Y>(
        input: Self,
        system_api: &mut Y,
    ) -> Result<(radix_engine_lib::resource::Bucket, CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi + InvokableNative<'a>,
    {
        let node_id = RENodeId::Vault(input.receiver);
        let offset = SubstateOffset::Vault(VaultOffset::Vault);
        let vault_handle = system_api.lock_substate(node_id, offset, LockFlags::MUTABLE)?;

        let container = {
            let mut substate_mut = system_api.get_ref_mut(vault_handle)?;
            let vault = substate_mut.vault();
            vault.take(input.amount).map_err(|e| match e {
                InvokeError::Error(e) => {
                    RuntimeError::ApplicationError(ApplicationError::VaultError(e))
                }
                InvokeError::Downstream(runtime_error) => runtime_error,
            })?
        };

        let bucket_id = system_api
            .create_node(RENode::Bucket(BucketSubstate::new(container)))?
            .into();

        Ok((
            radix_engine_lib::resource::Bucket(bucket_id),
            CallFrameUpdate::move_node(RENodeId::Bucket(bucket_id)),
        ))
    }
}

impl NativeInvocation for VaultTakeInvocation {
    fn info(&self) -> NativeInvocationInfo {
        NativeInvocationInfo::Method(
            NativeMethod::Vault(VaultMethod::Take),
            RENodeId::Vault(self.receiver),
            CallFrameUpdate::empty(),
        )
    }
}

impl NativeExecutable for VaultPutInvocation {
    type NativeOutput = ();

    fn execute<'a, Y>(
        input: Self,
        system_api: &mut Y,
    ) -> Result<((), CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi + InvokableNative<'a>,
    {
        let node_id = RENodeId::Vault(input.receiver);
        let offset = SubstateOffset::Vault(VaultOffset::Vault);
        let vault_handle = system_api.lock_substate(node_id, offset, LockFlags::MUTABLE)?;

        let bucket = system_api
            .drop_node(RENodeId::Bucket(input.bucket.0))?
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

impl NativeInvocation for VaultPutInvocation {
    fn info(&self) -> NativeInvocationInfo {
        NativeInvocationInfo::Method(
            NativeMethod::Vault(VaultMethod::Put),
            RENodeId::Vault(self.receiver),
            CallFrameUpdate::move_node(RENodeId::Bucket(self.bucket.0)),
        )
    }
}

impl NativeExecutable for VaultLockFeeInvocation {
    type NativeOutput = ();

    fn execute<'a, Y>(
        input: Self,
        system_api: &mut Y,
    ) -> Result<((), CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi + InvokableNative<'a>,
    {
        let node_id = RENodeId::Vault(input.receiver);
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
            vault.take(input.amount).map_err(|_| {
                RuntimeError::ApplicationError(ApplicationError::VaultError(
                    VaultError::LockFeeInsufficientBalance,
                ))
            })?
        };

        // Refill fee reserve
        let changes = system_api.lock_fee(input.receiver, fee, input.contingent)?;

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

impl NativeInvocation for VaultLockFeeInvocation {
    fn info(&self) -> NativeInvocationInfo {
        NativeInvocationInfo::Method(
            NativeMethod::Vault(VaultMethod::LockFee),
            RENodeId::Vault(self.receiver),
            CallFrameUpdate::empty(),
        )
    }
}

impl NativeExecutable for VaultTakeNonFungiblesInvocation {
    type NativeOutput = radix_engine_lib::resource::Bucket;

    fn execute<'a, Y>(
        input: Self,
        system_api: &mut Y,
    ) -> Result<(radix_engine_lib::resource::Bucket, CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi + InvokableNative<'a>,
    {
        let node_id = RENodeId::Vault(input.receiver);
        let offset = SubstateOffset::Vault(VaultOffset::Vault);
        let vault_handle = system_api.lock_substate(node_id, offset, LockFlags::MUTABLE)?;

        let container = {
            let mut substate_mut = system_api.get_ref_mut(vault_handle)?;
            let vault = substate_mut.vault();
            vault
                .take_non_fungibles(&input.non_fungible_ids)
                .map_err(|e| match e {
                    InvokeError::Error(e) => {
                        RuntimeError::ApplicationError(ApplicationError::VaultError(e))
                    }
                    InvokeError::Downstream(runtime_error) => runtime_error,
                })?
        };

        let bucket_id = system_api
            .create_node(RENode::Bucket(BucketSubstate::new(container)))?
            .into();

        Ok((
            radix_engine_lib::resource::Bucket(bucket_id),
            CallFrameUpdate::move_node(RENodeId::Bucket(bucket_id)),
        ))
    }
}

impl NativeInvocation for VaultTakeNonFungiblesInvocation {
    fn info(&self) -> NativeInvocationInfo {
        NativeInvocationInfo::Method(
            NativeMethod::Vault(VaultMethod::TakeNonFungibles),
            RENodeId::Vault(self.receiver),
            CallFrameUpdate::empty(),
        )
    }
}

impl NativeExecutable for VaultGetAmountInvocation {
    type NativeOutput = Decimal;

    fn execute<'a, Y>(
        input: Self,
        system_api: &mut Y,
    ) -> Result<(Decimal, CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi + InvokableNative<'a>,
    {
        let node_id = RENodeId::Vault(input.receiver);
        let offset = SubstateOffset::Vault(VaultOffset::Vault);
        let vault_handle = system_api.lock_substate(node_id, offset, LockFlags::read_only())?;

        let substate_ref = system_api.get_ref(vault_handle)?;
        let vault = substate_ref.vault();
        let amount = vault.total_amount();

        Ok((amount, CallFrameUpdate::empty()))
    }
}

impl NativeInvocation for VaultGetAmountInvocation {
    fn info(&self) -> NativeInvocationInfo {
        NativeInvocationInfo::Method(
            NativeMethod::Vault(VaultMethod::GetAmount),
            RENodeId::Vault(self.receiver),
            CallFrameUpdate::empty(),
        )
    }
}

impl NativeExecutable for VaultGetResourceAddressInvocation {
    type NativeOutput = ResourceAddress;

    fn execute<'a, Y>(
        input: Self,
        system_api: &mut Y,
    ) -> Result<(ResourceAddress, CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi + InvokableNative<'a>,
    {
        let node_id = RENodeId::Vault(input.receiver);
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

impl NativeInvocation for VaultGetResourceAddressInvocation {
    fn info(&self) -> NativeInvocationInfo {
        NativeInvocationInfo::Method(
            NativeMethod::Vault(VaultMethod::GetResourceAddress),
            RENodeId::Vault(self.receiver),
            CallFrameUpdate::empty(),
        )
    }
}

impl NativeExecutable for VaultGetNonFungibleIdsInvocation {
    type NativeOutput = BTreeSet<NonFungibleId>;

    fn execute<'a, Y>(
        input: Self,
        system_api: &mut Y,
    ) -> Result<(BTreeSet<NonFungibleId>, CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi + InvokableNative<'a>,
    {
        let node_id = RENodeId::Vault(input.receiver);
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

impl NativeInvocation for VaultGetNonFungibleIdsInvocation {
    fn info(&self) -> NativeInvocationInfo {
        NativeInvocationInfo::Method(
            NativeMethod::Vault(VaultMethod::GetNonFungibleIds),
            RENodeId::Vault(self.receiver),
            CallFrameUpdate::empty(),
        )
    }
}

impl NativeExecutable for VaultCreateProofInvocation {
    type NativeOutput = radix_engine_lib::resource::Proof;

    fn execute<'a, Y>(
        input: Self,
        system_api: &mut Y,
    ) -> Result<(radix_engine_lib::resource::Proof, CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi + InvokableNative<'a>,
    {
        let node_id = RENodeId::Vault(input.receiver);
        let offset = SubstateOffset::Vault(VaultOffset::Vault);
        let vault_handle = system_api.lock_substate(node_id, offset, LockFlags::MUTABLE)?;

        let proof = {
            let mut substate_mut = system_api.get_ref_mut(vault_handle)?;
            let vault = substate_mut.vault();
            vault
                .create_proof(ResourceContainerId::Vault(input.receiver))
                .map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::VaultError(
                        VaultError::ProofError(e),
                    ))
                })?
        };
        let proof_id = system_api.create_node(RENode::Proof(proof))?.into();

        Ok((
            radix_engine_lib::resource::Proof(proof_id),
            CallFrameUpdate::move_node(RENodeId::Proof(proof_id)),
        ))
    }
}

impl NativeInvocation for VaultCreateProofInvocation {
    fn info(&self) -> NativeInvocationInfo {
        NativeInvocationInfo::Method(
            NativeMethod::Vault(VaultMethod::CreateProof),
            RENodeId::Vault(self.receiver),
            CallFrameUpdate::empty(),
        )
    }
}

impl NativeExecutable for VaultCreateProofByAmountInvocation {
    type NativeOutput = radix_engine_lib::resource::Proof;

    fn execute<'a, Y>(
        input: Self,
        system_api: &mut Y,
    ) -> Result<(radix_engine_lib::resource::Proof, CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi + InvokableNative<'a>,
    {
        let node_id = RENodeId::Vault(input.receiver);
        let offset = SubstateOffset::Vault(VaultOffset::Vault);
        let vault_handle = system_api.lock_substate(node_id, offset, LockFlags::MUTABLE)?;

        let proof = {
            let mut substate_mut = system_api.get_ref_mut(vault_handle)?;
            let vault = substate_mut.vault();
            vault
                .create_proof_by_amount(input.amount, ResourceContainerId::Vault(input.receiver))
                .map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::VaultError(
                        VaultError::ProofError(e),
                    ))
                })?
        };

        let proof_id = system_api.create_node(RENode::Proof(proof))?.into();

        Ok((
            radix_engine_lib::resource::Proof(proof_id),
            CallFrameUpdate::move_node(RENodeId::Proof(proof_id)),
        ))
    }
}

impl NativeInvocation for VaultCreateProofByAmountInvocation {
    fn info(&self) -> NativeInvocationInfo {
        NativeInvocationInfo::Method(
            NativeMethod::Vault(VaultMethod::CreateProofByAmount),
            RENodeId::Vault(self.receiver),
            CallFrameUpdate::empty(),
        )
    }
}

impl NativeExecutable for VaultCreateProofByIdsInvocation {
    type NativeOutput = radix_engine_lib::resource::Proof;

    fn execute<'a, Y>(
        input: Self,
        system_api: &mut Y,
    ) -> Result<(radix_engine_lib::resource::Proof, CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi + InvokableNative<'a>,
    {
        let node_id = RENodeId::Vault(input.receiver);
        let offset = SubstateOffset::Vault(VaultOffset::Vault);
        let vault_handle = system_api.lock_substate(node_id, offset, LockFlags::MUTABLE)?;

        let proof = {
            let mut substate_mut = system_api.get_ref_mut(vault_handle)?;
            let vault = substate_mut.vault();
            vault
                .create_proof_by_ids(&input.ids, ResourceContainerId::Vault(input.receiver))
                .map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::VaultError(
                        VaultError::ProofError(e),
                    ))
                })?
        };

        let proof_id = system_api.create_node(RENode::Proof(proof))?.into();

        Ok((
            radix_engine_lib::resource::Proof(proof_id),
            CallFrameUpdate::move_node(RENodeId::Proof(proof_id)),
        ))
    }
}

impl NativeInvocation for VaultCreateProofByIdsInvocation {
    fn info(&self) -> NativeInvocationInfo {
        NativeInvocationInfo::Method(
            NativeMethod::Vault(VaultMethod::CreateProofByIds),
            RENodeId::Vault(self.receiver),
            CallFrameUpdate::empty(),
        )
    }
}
