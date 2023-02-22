use crate::errors::{ApplicationError, InterpreterError, RuntimeError};
use crate::kernel::kernel_api::LockFlags;
use crate::kernel::kernel_api::{KernelNodeApi, KernelSubstateApi};
use crate::system::kernel_modules::execution_trace::ProofSnapshot;
use crate::system::node::RENodeInit;
use crate::types::*;
use radix_engine_interface::api::types::*;
use radix_engine_interface::api::types::{ProofOffset, RENodeId, SubstateOffset};
use radix_engine_interface::api::{ClientApi, ClientSubstateApi};
use radix_engine_interface::blueprints::resource::*;
use radix_engine_interface::data::ScryptoValue;

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, ScryptoSbor)]
pub enum LocalRef {
    Bucket(BucketId),
    Vault(VaultId),
}

impl LocalRef {
    pub fn to_re_node_id(&self) -> RENodeId {
        match self {
            LocalRef::Bucket(id) => RENodeId::Bucket(id.clone()),
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
    NonFungibleOperationOnFungible,
}

#[derive(Debug)]
pub enum ProofSubstate {
    Fungible(FungibleProof),
    NonFungible(NonFungibleProof),
}

impl From<FungibleProof> for ProofSubstate {
    fn from(value: FungibleProof) -> Self {
        Self::Fungible(value)
    }
}

impl From<NonFungibleProof> for ProofSubstate {
    fn from(value: NonFungibleProof) -> Self {
        Self::NonFungible(value)
    }
}

impl ProofSubstate {
    pub fn change_to_unrestricted(&mut self) {
        match self {
            ProofSubstate::Fungible(f) => f.change_to_unrestricted(),
            ProofSubstate::NonFungible(nf) => nf.change_to_unrestricted(),
        }
    }

    pub fn change_to_restricted(&mut self) {
        match self {
            ProofSubstate::Fungible(f) => f.change_to_restricted(),
            ProofSubstate::NonFungible(nf) => nf.change_to_restricted(),
        }
    }

    pub fn resource_address(&self) -> ResourceAddress {
        match self {
            ProofSubstate::Fungible(f) => f.resource_address(),
            ProofSubstate::NonFungible(nf) => nf.resource_address(),
        }
    }

    pub fn total_amount(&self) -> Decimal {
        match self {
            ProofSubstate::Fungible(f) => f.total_amount(),
            ProofSubstate::NonFungible(nf) => nf.total_amount(),
        }
    }

    pub fn total_ids(&self) -> Option<&BTreeSet<NonFungibleLocalId>> {
        match self {
            ProofSubstate::Fungible(_) => None,
            ProofSubstate::NonFungible(nf) => Some(nf.total_ids()),
        }
    }

    pub fn is_restricted(&self) -> bool {
        match self {
            ProofSubstate::Fungible(f) => f.is_restricted(),
            ProofSubstate::NonFungible(nf) => nf.is_restricted(),
        }
    }

    pub fn clone_proof<Y: ClientApi<RuntimeError>>(
        &self,
        api: &mut Y,
    ) -> Result<Self, RuntimeError> {
        match self {
            ProofSubstate::Fungible(f) => Ok(f.clone_proof(api)?.into()),
            ProofSubstate::NonFungible(nf) => Ok(nf.clone_proof(api)?.into()),
        }
    }

    pub fn drop_proof<Y: ClientApi<RuntimeError>>(
        &mut self,
        api: &mut Y,
    ) -> Result<(), RuntimeError> {
        match self {
            ProofSubstate::Fungible(f) => f.drop_proof(api),
            ProofSubstate::NonFungible(nf) => nf.drop_proof(api),
        }
    }

    pub fn snapshot(&self) -> ProofSnapshot {
        match self {
            ProofSubstate::Fungible(f) => f.snapshot(),
            ProofSubstate::NonFungible(nf) => nf.snapshot(),
        }
    }
}

#[derive(Debug)]
pub struct FungibleProof {
    /// The resource address.
    pub resource_address: ResourceAddress,
    /// Whether movement of this proof is restricted.
    pub restricted: bool,
    /// The total locked amount or non-fungible ids.
    pub total_locked: Decimal,
    /// The supporting containers.
    pub evidence: BTreeMap<LocalRef, Decimal>,
}

impl FungibleProof {
    pub fn new(
        resource_address: ResourceAddress,
        total_locked: Decimal,
        evidence: BTreeMap<LocalRef, Decimal>,
    ) -> Result<FungibleProof, ProofError> {
        if total_locked.is_zero() {
            return Err(ProofError::EmptyProofNotAllowed);
        }

        Ok(Self {
            resource_address,
            restricted: false,
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
            resource_address: self.resource_address.clone(),
            restricted: self.restricted,
            total_locked: self.total_locked.clone(),
            evidence: self.evidence.clone(),
        })
    }

    pub fn drop_proof<Y: ClientApi<RuntimeError>>(
        &mut self,
        api: &mut Y,
    ) -> Result<(), RuntimeError> {
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

    pub fn change_to_unrestricted(&mut self) {
        self.restricted = false;
    }

    pub fn change_to_restricted(&mut self) {
        self.restricted = true;
    }

    pub fn resource_address(&self) -> ResourceAddress {
        self.resource_address
    }

    pub fn total_amount(&self) -> Decimal {
        self.total_locked
    }

    pub fn is_restricted(&self) -> bool {
        self.restricted
    }

    pub fn snapshot(&self) -> ProofSnapshot {
        ProofSnapshot::Fungible {
            resource_address: self.resource_address,
            restricted: self.restricted,
            total_locked: self.total_locked.clone(),
        }
    }
}

#[derive(Debug)]
pub struct NonFungibleProof {
    /// The resource address.
    pub resource_address: ResourceAddress,
    /// Whether movement of this proof is restricted.
    pub restricted: bool,
    /// The total locked amount or non-fungible ids.
    pub total_locked: BTreeSet<NonFungibleLocalId>,
    /// The supporting containers.
    pub evidence: BTreeMap<LocalRef, BTreeSet<NonFungibleLocalId>>,
}

impl NonFungibleProof {
    pub fn new(
        resource_address: ResourceAddress,
        total_locked: BTreeSet<NonFungibleLocalId>,
        evidence: BTreeMap<LocalRef, BTreeSet<NonFungibleLocalId>>,
    ) -> Result<NonFungibleProof, ProofError> {
        if total_locked.is_empty() {
            return Err(ProofError::EmptyProofNotAllowed);
        }

        Ok(Self {
            resource_address,
            restricted: false,
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
            resource_address: self.resource_address.clone(),
            restricted: self.restricted,
            total_locked: self.total_locked.clone(),
            evidence: self.evidence.clone(),
        })
    }

    pub fn drop_proof<Y: ClientApi<RuntimeError>>(
        &mut self,
        api: &mut Y,
    ) -> Result<(), RuntimeError> {
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

    pub fn change_to_unrestricted(&mut self) {
        self.restricted = false;
    }

    pub fn change_to_restricted(&mut self) {
        self.restricted = true;
    }

    pub fn resource_address(&self) -> ResourceAddress {
        self.resource_address
    }

    pub fn total_amount(&self) -> Decimal {
        self.total_ids().len().into()
    }

    pub fn total_ids(&self) -> &BTreeSet<NonFungibleLocalId> {
        &self.total_locked
    }

    pub fn is_restricted(&self) -> bool {
        self.restricted
    }

    pub fn snapshot(&self) -> ProofSnapshot {
        ProofSnapshot::NonFungible {
            resource_address: self.resource_address,
            restricted: self.restricted,
            total_locked: self.total_locked.clone(),
        }
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
        Y: KernelNodeApi
            + KernelSubstateApi
            + ClientSubstateApi<RuntimeError>
            + ClientApi<RuntimeError>,
    {
        // TODO: Remove decode/encode mess
        let _input: ProofCloneInput = scrypto_decode(&scrypto_encode(&input).unwrap())
            .map_err(|_| RuntimeError::InterpreterError(InterpreterError::InvalidInvocation))?;

        let handle = api.kernel_lock_substate(
            receiver,
            NodeModuleId::SELF,
            SubstateOffset::Proof(ProofOffset::Proof),
            LockFlags::read_only(),
        )?;
        let substate_ref = api.kernel_get_substate_ref(handle)?;
        let proof = substate_ref.proof();
        let cloned_proof = proof.clone_proof(api)?;

        let node_id = api.kernel_allocate_node_id(RENodeType::Proof)?;
        api.kernel_create_node(node_id, RENodeInit::Proof(cloned_proof), BTreeMap::new())?;
        let proof_id = node_id.into();

        Ok(IndexedScryptoValue::from_typed(&Proof(proof_id)))
    }

    pub(crate) fn get_amount<Y>(
        receiver: RENodeId,
        input: ScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi
            + KernelSubstateApi
            + ClientSubstateApi<RuntimeError>
            + ClientApi<RuntimeError>,
    {
        // TODO: Remove decode/encode mess
        let _input: ProofGetAmountInput = scrypto_decode(&scrypto_encode(&input).unwrap())
            .map_err(|_| RuntimeError::InterpreterError(InterpreterError::InvalidInvocation))?;

        let handle = api.kernel_lock_substate(
            receiver,
            NodeModuleId::SELF,
            SubstateOffset::Proof(ProofOffset::Proof),
            LockFlags::read_only(),
        )?;
        let substate_ref = api.kernel_get_substate_ref(handle)?;
        let proof = substate_ref.proof();
        Ok(IndexedScryptoValue::from_typed(&proof.total_amount()))
    }

    pub(crate) fn get_non_fungible_local_ids<Y>(
        receiver: RENodeId,
        input: ScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi
            + KernelSubstateApi
            + ClientSubstateApi<RuntimeError>
            + ClientApi<RuntimeError>,
    {
        // TODO: Remove decode/encode mess
        let _input: ProofGetNonFungibleLocalIdsInput =
            scrypto_decode(&scrypto_encode(&input).unwrap())
                .map_err(|_| RuntimeError::InterpreterError(InterpreterError::InvalidInvocation))?;

        let handle = api.kernel_lock_substate(
            receiver,
            NodeModuleId::SELF,
            SubstateOffset::Proof(ProofOffset::Proof),
            LockFlags::read_only(),
        )?;
        let substate_ref = api.kernel_get_substate_ref(handle)?;
        let proof = substate_ref.proof();
        let ids = proof.total_ids().ok_or(RuntimeError::ApplicationError(
            ApplicationError::ProofError(ProofError::NonFungibleOperationOnFungible),
        ))?;
        Ok(IndexedScryptoValue::from_typed(&ids))
    }

    pub(crate) fn get_resource_address<Y>(
        receiver: RENodeId,
        input: ScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi
            + KernelSubstateApi
            + ClientSubstateApi<RuntimeError>
            + ClientApi<RuntimeError>,
    {
        // TODO: Remove decode/encode mess
        let _input: ProofGetResourceAddressInput = scrypto_decode(&scrypto_encode(&input).unwrap())
            .map_err(|_| RuntimeError::InterpreterError(InterpreterError::InvalidInvocation))?;

        let handle = api.kernel_lock_substate(
            receiver,
            NodeModuleId::SELF,
            SubstateOffset::Proof(ProofOffset::Proof),
            LockFlags::read_only(),
        )?;
        let substate_ref = api.kernel_get_substate_ref(handle)?;
        let proof = substate_ref.proof();
        Ok(IndexedScryptoValue::from_typed(&proof.resource_address()))
    }
}
