use crate::errors::{ApplicationError, InterpreterError, RuntimeError};
use crate::kernel::kernel_api::{KernelNodeApi, KernelSubstateApi};
use crate::types::*;
use radix_engine_interface::api::substate_api::LockFlags;
use radix_engine_interface::api::types::{ProofOffset, RENodeId, SubstateOffset};
use radix_engine_interface::api::ClientApi;
use radix_engine_interface::api::{types::*, ClientSubstateApi};
use radix_engine_interface::blueprints::resource::*;
use radix_engine_interface::data::ScryptoValue;

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, ScryptoSbor)]
pub enum LocalRef {
    Bucket(ComponentId),
    Vault(VaultId),
}

impl LocalRef {
    pub fn to_re_node_id(&self) -> RENodeId {
        match self {
            LocalRef::Bucket(id) => RENodeId::Component(id.clone()),
            LocalRef::Vault(id) => RENodeId::Vault(id.clone()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum ProofError {
    InvalidRequestData(DecodeError),
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
        node_id: RENodeId,
        api: &mut Y,
    ) -> Result<Self, RuntimeError> {
        let handle = api.sys_lock_substate(
            node_id,
            SubstateOffset::Proof(ProofOffset::Info),
            LockFlags::read_only(),
        )?;
        let substate_ref: &ProofInfoSubstate = api.kernel_get_substate_ref(handle)?;
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

#[derive(Debug, Clone, ScryptoSbor)]
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
        for (container_id, locked_amount) in &self.evidence {
            api.call_method(
                container_id.to_re_node_id(),
                match container_id {
                    LocalRef::Bucket(_) => BUCKET_LOCK_AMOUNT_IDENT,
                    LocalRef::Vault(_) => VAULT_LOCK_AMOUNT_IDENT,
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
        for (container_id, locked_amount) in &self.evidence {
            api.call_method(
                container_id.to_re_node_id(),
                match container_id {
                    LocalRef::Bucket(_) => BUCKET_UNLOCK_AMOUNT_IDENT,
                    LocalRef::Vault(_) => VAULT_UNLOCK_AMOUNT_IDENT,
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

#[derive(Debug, Clone, ScryptoSbor)]
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
        for (container_id, locked_ids) in &self.evidence {
            api.call_method(
                container_id.to_re_node_id(),
                match container_id {
                    LocalRef::Bucket(_) => BUCKET_LOCK_NON_FUNGIBLES_IDENT,
                    LocalRef::Vault(_) => VAULT_LOCK_NON_FUNGIBLES_IDENT,
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
        for (container_id, locked_ids) in &self.evidence {
            api.call_method(
                container_id.to_re_node_id(),
                match container_id {
                    LocalRef::Bucket(_) => BUCKET_UNLOCK_NON_FUNGIBLES_IDENT,
                    LocalRef::Vault(_) => VAULT_UNLOCK_NON_FUNGIBLES_IDENT,
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
        receiver: RENodeId,
        input: ScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        // TODO: Remove decode/encode mess
        let _input: ProofCloneInput = scrypto_decode(&scrypto_encode(&input).unwrap())
            .map_err(|_| RuntimeError::InterpreterError(InterpreterError::InvalidInvocation))?;

        let proof_info = ProofInfoSubstate::of(receiver, api)?;
        let node_id = if proof_info.resource_type.is_fungible() {
            let handle = api.sys_lock_substate(
                receiver,
                SubstateOffset::Proof(ProofOffset::Fungible),
                LockFlags::read_only(),
            )?;
            let substate_ref: &FungibleProof = api.kernel_get_substate_ref(handle)?;
            let proof = substate_ref.clone();
            let clone = proof.clone_proof(api)?;
            api.sys_drop_lock(handle)?;

            let proof_id = api.new_object(
                PROOF_BLUEPRINT,
                btreemap!(
                    0 => scrypto_encode(&proof_info).unwrap(),
                    1 => scrypto_encode(&clone).unwrap()
                ),
            )?;

            RENodeId::Proof(proof_id)
        } else {
            let handle = api.sys_lock_substate(
                receiver,
                SubstateOffset::Proof(ProofOffset::NonFungible),
                LockFlags::read_only(),
            )?;
            let substate_ref: &NonFungibleProof = api.kernel_get_substate_ref(handle)?;
            let proof = substate_ref.clone();
            let clone = proof.clone_proof(api)?;
            api.sys_drop_lock(handle)?;

            let proof_id = api.new_object(
                PROOF_BLUEPRINT,
                btreemap!(
                    0 => scrypto_encode(&proof_info).unwrap(),
                    1 => scrypto_encode(&clone).unwrap()
                ),
            )?;

            RENodeId::Proof(proof_id)
        };

        let proof_id = node_id.into();
        Ok(IndexedScryptoValue::from_typed(&Proof(proof_id)))
    }

    pub(crate) fn get_amount<Y>(
        receiver: RENodeId,
        input: ScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        // TODO: Remove decode/encode mess
        let _input: ProofGetAmountInput = scrypto_decode(&scrypto_encode(&input).unwrap())
            .map_err(|_| RuntimeError::InterpreterError(InterpreterError::InvalidInvocation))?;

        let proof_info = ProofInfoSubstate::of(receiver, api)?;
        let amount = if proof_info.resource_type.is_fungible() {
            let handle = api.sys_lock_substate(
                receiver,
                SubstateOffset::Proof(ProofOffset::Fungible),
                LockFlags::read_only(),
            )?;
            let substate_ref: &FungibleProof = api.kernel_get_substate_ref(handle)?;
            let amount = substate_ref.amount();
            api.sys_drop_lock(handle)?;
            amount
        } else {
            let handle = api.sys_lock_substate(
                receiver,
                SubstateOffset::Proof(ProofOffset::NonFungible),
                LockFlags::read_only(),
            )?;
            let substate_ref: &NonFungibleProof = api.kernel_get_substate_ref(handle)?;
            let amount = substate_ref.amount();
            api.sys_drop_lock(handle)?;
            amount
        };
        Ok(IndexedScryptoValue::from_typed(&amount))
    }

    pub(crate) fn get_non_fungible_local_ids<Y>(
        receiver: RENodeId,
        input: ScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        // TODO: Remove decode/encode mess
        let _input: ProofGetNonFungibleLocalIdsInput =
            scrypto_decode(&scrypto_encode(&input).unwrap())
                .map_err(|_| RuntimeError::InterpreterError(InterpreterError::InvalidInvocation))?;

        let proof_info = ProofInfoSubstate::of(receiver, api)?;
        if proof_info.resource_type.is_fungible() {
            Err(RuntimeError::ApplicationError(
                ApplicationError::ProofError(ProofError::NonFungibleOperationNotSupported),
            ))
        } else {
            let handle = api.sys_lock_substate(
                receiver,
                SubstateOffset::Proof(ProofOffset::NonFungible),
                LockFlags::read_only(),
            )?;
            let substate_ref: &NonFungibleProof = api.kernel_get_substate_ref(handle)?;
            let ids = substate_ref.non_fungible_local_ids().clone();
            api.sys_drop_lock(handle)?;
            Ok(IndexedScryptoValue::from_typed(&ids))
        }
    }

    pub(crate) fn get_resource_address<Y>(
        receiver: RENodeId,
        input: ScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        // TODO: Remove decode/encode mess
        let _input: ProofGetResourceAddressInput = scrypto_decode(&scrypto_encode(&input).unwrap())
            .map_err(|_| RuntimeError::InterpreterError(InterpreterError::InvalidInvocation))?;

        let proof_info = ProofInfoSubstate::of(receiver, api)?;
        Ok(IndexedScryptoValue::from_typed(
            &proof_info.resource_address,
        ))
    }

    pub(crate) fn drop<Y>(proof: Proof, api: &mut Y) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        let mut heap_node = api.kernel_drop_node(RENodeId::Proof(proof.0))?;
        let proof_info: ProofInfoSubstate = heap_node
            .substates
            .remove(&(NodeModuleId::SELF, SubstateOffset::Proof(ProofOffset::Info)))
            .unwrap()
            .into();
        if proof_info.resource_type.is_fungible() {
            let proof: FungibleProof = heap_node
                .substates
                .remove(&(
                    NodeModuleId::SELF,
                    SubstateOffset::Proof(ProofOffset::Fungible),
                ))
                .unwrap()
                .into();
            proof.drop_proof(api)?;
        } else {
            let proof: NonFungibleProof = heap_node
                .substates
                .remove(&(
                    NodeModuleId::SELF,
                    SubstateOffset::Proof(ProofOffset::NonFungible),
                ))
                .unwrap()
                .into();
            proof.drop_proof(api)?;
        }

        Ok(IndexedScryptoValue::from_typed(&()))
    }
}
