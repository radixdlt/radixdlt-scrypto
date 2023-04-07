use crate::errors::{ApplicationError, InterpreterError, RuntimeError};
use crate::kernel::heap::{DroppedProof, DroppedProofResource};
use crate::kernel::kernel_api::{KernelNodeApi, KernelSubstateApi};
use crate::types::*;
use radix_engine_interface::api::substate_api::LockFlags;
use radix_engine_interface::api::ClientApi;
use radix_engine_interface::api::ClientSubstateApi;
use radix_engine_interface::blueprints::resource::*;

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, ScryptoSbor)]
pub enum LocalRef {
    Bucket(Reference),
    Vault(Reference),
}

impl LocalRef {
    pub fn as_node_id(&self) -> &NodeId {
        match self {
            LocalRef::Bucket(id) => id.as_node_id(),
            LocalRef::Vault(id) => id.as_node_id(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum ProofError {
    /// Error produced by a resource container.
    ResourceError(ResourceError),
    /// Can't generate zero-amount or empty non-fungible set proofs.
    EmptyProofNotAllowed,
    /// Can't apply a non-fungible operation on fungible proofs.
    NonFungibleOperationNotSupported,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, ScryptoSbor)]
pub struct ProofInfoSubstate {
    /// The resource address.
    pub resource_address: ResourceAddress,
    /// The resource type.
    pub resource_type: ResourceType,
    /// Whether movement of this proof is restricted.
    pub restricted: bool,
}

impl ProofInfoSubstate {
    pub fn of<Y: KernelSubstateApi + ClientSubstateApi<RuntimeError>>(
        receiver: &NodeId,
        api: &mut Y,
    ) -> Result<Self, RuntimeError> {
        let handle =
            api.sys_lock_substate(receiver, &ProofOffset::Info.into(), LockFlags::read_only())?;
        let substate_ref: ProofInfoSubstate = api.sys_read_substate_typed(handle)?;
        let info = substate_ref.clone();
        api.sys_drop_lock(handle)?;
        Ok(info)
    }

    pub fn change_to_unrestricted(&mut self) {
        self.restricted = false;
    }

    pub fn change_to_restricted(&mut self) {
        self.restricted = true;
    }
}

#[derive(Debug, Clone, ScryptoSbor, Default)]
pub struct FungibleProof {
    pub total_locked: Decimal,
    /// The supporting containers.
    pub evidence: BTreeMap<LocalRef, Decimal>,
}

impl FungibleProof {
    pub fn new(
        total_locked: Decimal,
        evidence: BTreeMap<LocalRef, Decimal>,
    ) -> Result<FungibleProof, ProofError> {
        if total_locked.is_zero() {
            return Err(ProofError::EmptyProofNotAllowed);
        }

        Ok(Self {
            total_locked,
            evidence,
        })
    }

    pub fn clone_proof<Y: ClientApi<RuntimeError>>(
        &self,
        api: &mut Y,
    ) -> Result<Self, RuntimeError> {
        for (container, locked_amount) in &self.evidence {
            api.call_method(
                container.as_node_id(),
                match container {
                    LocalRef::Bucket(_) => BUCKET_LOCK_AMOUNT_IDENT,
                    LocalRef::Vault(_) => FUNGIBLE_VAULT_LOCK_FUNGIBLE_AMOUNT_IDENT,
                },
                scrypto_args!(locked_amount),
            )?;
        }
        Ok(Self {
            total_locked: self.total_locked.clone(),
            evidence: self.evidence.clone(),
        })
    }

    pub fn drop_proof<Y: ClientApi<RuntimeError>>(self, api: &mut Y) -> Result<(), RuntimeError> {
        for (container, locked_amount) in &self.evidence {
            api.call_method(
                container.as_node_id(),
                match container {
                    LocalRef::Bucket(_) => BUCKET_UNLOCK_AMOUNT_IDENT,
                    LocalRef::Vault(_) => FUNGIBLE_VAULT_UNLOCK_FUNGIBLE_AMOUNT_IDENT,
                },
                scrypto_args!(locked_amount),
            )?;
        }
        Ok(())
    }

    pub fn amount(&self) -> Decimal {
        self.total_locked
    }
}

#[derive(Debug, Clone, ScryptoSbor, Default)]
pub struct NonFungibleProof {
    /// The total locked amount or non-fungible ids.
    pub total_locked: BTreeSet<NonFungibleLocalId>,
    /// The supporting containers.
    pub evidence: BTreeMap<LocalRef, BTreeSet<NonFungibleLocalId>>,
}

impl NonFungibleProof {
    pub fn new(
        total_locked: BTreeSet<NonFungibleLocalId>,
        evidence: BTreeMap<LocalRef, BTreeSet<NonFungibleLocalId>>,
    ) -> Result<NonFungibleProof, ProofError> {
        if total_locked.is_empty() {
            return Err(ProofError::EmptyProofNotAllowed);
        }

        Ok(Self {
            total_locked,
            evidence,
        })
    }

    pub fn clone_proof<Y: ClientApi<RuntimeError>>(
        &self,
        api: &mut Y,
    ) -> Result<Self, RuntimeError> {
        for (container, locked_ids) in &self.evidence {
            api.call_method(
                container.as_node_id(),
                match container {
                    LocalRef::Bucket(_) => BUCKET_LOCK_NON_FUNGIBLES_IDENT,
                    LocalRef::Vault(_) => NON_FUNGIBLE_VAULT_LOCK_NON_FUNGIBLES_IDENT,
                },
                scrypto_args!(locked_ids),
            )?;
        }
        Ok(Self {
            total_locked: self.total_locked.clone(),
            evidence: self.evidence.clone(),
        })
    }

    pub fn drop_proof<Y: ClientApi<RuntimeError>>(self, api: &mut Y) -> Result<(), RuntimeError> {
        for (container, locked_ids) in &self.evidence {
            api.call_method(
                container.as_node_id(),
                match container {
                    LocalRef::Bucket(_) => BUCKET_UNLOCK_NON_FUNGIBLES_IDENT,
                    LocalRef::Vault(_) => NON_FUNGIBLE_VAULT_UNLOCK_NON_FUNGIBLES_IDENT,
                },
                scrypto_args!(locked_ids),
            )?;
        }
        Ok(())
    }

    pub fn amount(&self) -> Decimal {
        self.non_fungible_local_ids().len().into()
    }

    pub fn non_fungible_local_ids(&self) -> &BTreeSet<NonFungibleLocalId> {
        &self.total_locked
    }
}

pub struct ProofBlueprint;

impl ProofBlueprint {
    pub(crate) fn clone<Y>(
        receiver: &NodeId,
        input: &IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        let _input: ProofCloneInput = input.as_typed().map_err(|e| {
            RuntimeError::InterpreterError(InterpreterError::ScryptoInputDecodeError(e))
        })?;

        let proof_info = ProofInfoSubstate::of(receiver, api)?;
        let node_id = if proof_info.resource_type.is_fungible() {
            let handle = api.sys_lock_substate(
                receiver,
                &ProofOffset::Fungible.into(),
                LockFlags::read_only(),
            )?;
            let substate_ref: FungibleProof = api.sys_read_substate_typed(handle)?;
            let proof = substate_ref.clone();
            let clone = proof.clone_proof(api)?;
            api.sys_drop_lock(handle)?;

            let proof_id = api.new_object(
                PROOF_BLUEPRINT,
                vec![
                    scrypto_encode(&proof_info).unwrap(),
                    scrypto_encode(&clone).unwrap(),
                    scrypto_encode(&NonFungibleProof::default()).unwrap(),
                ],
            )?;

            proof_id
        } else {
            let handle = api.sys_lock_substate(
                receiver,
                &ProofOffset::NonFungible.into(),
                LockFlags::read_only(),
            )?;
            let substate_ref: NonFungibleProof = api.sys_read_substate_typed(handle)?;
            let proof = substate_ref.clone();
            let clone = proof.clone_proof(api)?;
            api.sys_drop_lock(handle)?;

            let proof_id = api.new_object(
                PROOF_BLUEPRINT,
                vec![
                    scrypto_encode(&proof_info).unwrap(),
                    scrypto_encode(&FungibleProof::default()).unwrap(),
                    scrypto_encode(&clone).unwrap(),
                ],
            )?;

            proof_id
        };

        Ok(IndexedScryptoValue::from_typed(&Proof(Own(node_id))))
    }

    pub(crate) fn get_amount<Y>(
        receiver: &NodeId,
        input: &IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        let _input: ProofGetAmountInput = input.as_typed().map_err(|e| {
            RuntimeError::InterpreterError(InterpreterError::ScryptoInputDecodeError(e))
        })?;

        let proof_info = ProofInfoSubstate::of(receiver, api)?;
        let amount = if proof_info.resource_type.is_fungible() {
            let handle = api.sys_lock_substate(
                receiver,
                &ProofOffset::Fungible.into(),
                LockFlags::read_only(),
            )?;
            let substate_ref: FungibleProof = api.sys_read_substate_typed(handle)?;
            let amount = substate_ref.amount();
            api.sys_drop_lock(handle)?;
            amount
        } else {
            let handle = api.sys_lock_substate(
                receiver,
                &ProofOffset::NonFungible.into(),
                LockFlags::read_only(),
            )?;
            let substate_ref: NonFungibleProof = api.sys_read_substate_typed(handle)?;
            let amount = substate_ref.amount();
            api.sys_drop_lock(handle)?;
            amount
        };
        Ok(IndexedScryptoValue::from_typed(&amount))
    }

    pub(crate) fn get_non_fungible_local_ids<Y>(
        receiver: &NodeId,
        input: &IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        let _input: ProofGetNonFungibleLocalIdsInput = input.as_typed().map_err(|e| {
            RuntimeError::InterpreterError(InterpreterError::ScryptoInputDecodeError(e))
        })?;

        let proof_info = ProofInfoSubstate::of(receiver, api)?;
        if proof_info.resource_type.is_fungible() {
            Err(RuntimeError::ApplicationError(
                ApplicationError::ProofError(ProofError::NonFungibleOperationNotSupported),
            ))
        } else {
            let handle = api.sys_lock_substate(
                receiver,
                &ProofOffset::NonFungible.into(),
                LockFlags::read_only(),
            )?;
            let substate_ref: NonFungibleProof = api.sys_read_substate_typed(handle)?;
            let ids = substate_ref.non_fungible_local_ids().clone();
            api.sys_drop_lock(handle)?;
            Ok(IndexedScryptoValue::from_typed(&ids))
        }
    }

    pub(crate) fn get_resource_address<Y>(
        receiver: &NodeId,
        input: &IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        let _input: ProofGetResourceAddressInput = input.as_typed().map_err(|e| {
            RuntimeError::InterpreterError(InterpreterError::ScryptoInputDecodeError(e))
        })?;

        let proof_info = ProofInfoSubstate::of(receiver, api)?;
        Ok(IndexedScryptoValue::from_typed(
            &proof_info.resource_address,
        ))
    }

    pub(crate) fn drop<Y>(
        input: &IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        let input: ProofDropInput = input.as_typed().map_err(|e| {
            RuntimeError::InterpreterError(InterpreterError::ScryptoInputDecodeError(e))
        })?;
        let proof = input.proof;

        // FIXME: check type before schema check is ready! applicable to all functions!

        let heap_node = api.kernel_drop_node(proof.0.as_node_id())?;
        let dropped_proof: DroppedProof = heap_node.into();
        match dropped_proof.resource {
            DroppedProofResource::Fungible(p) => p.drop_proof(api)?,
            DroppedProofResource::NonFungible(p) => p.drop_proof(api)?,
        };

        Ok(IndexedScryptoValue::from_typed(&()))
    }
}
