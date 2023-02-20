use crate::errors::{InterpreterError, InvokeError, RuntimeError};
use crate::kernel::kernel_api::LockFlags;
use crate::kernel::kernel_api::{KernelNodeApi, KernelSubstateApi};
use crate::system::kernel_modules::execution_trace::ProofSnapshot;
use crate::system::node::RENodeInit;
use crate::types::*;
use radix_engine_interface::api::types::*;
use radix_engine_interface::api::types::{ProofOffset, RENodeId, SubstateOffset};
use radix_engine_interface::api::{ClientApi, ClientNativeInvokeApi, ClientSubstateApi};
use radix_engine_interface::blueprints::resource::*;
use radix_engine_interface::data::ScryptoValue;

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum ProofError {
    /// Error produced by a resource container.
    ResourceError(ResourceError),
    /// Can't generate zero-amount or empty non-fungible set proofs.
    EmptyProofNotAllowed,
    /// The base proofs are not enough to cover the requested amount or non-fungible ids.
    InsufficientBaseProofs,
    /// Can't apply a non-fungible operation on fungible proofs.
    NonFungibleOperationNotAllowed,
    /// Can't apply a fungible operation on non-fungible proofs.
    FungibleOperationNotAllowed,
    InvalidRequestData(DecodeError),
}

#[derive(Debug, Clone, Hash, Eq, PartialEq, PartialOrd, Ord)]
pub enum ResourceContainerId {
    Bucket(BucketId),
    Vault(VaultId),
}

#[derive(Debug)]
pub struct FungibleProofSubstate {
    /// The resource address.
    pub resource_address: ResourceAddress,
    /// Whether movement of this proof is restricted.
    pub restricted: bool,
    /// The total locked amount or non-fungible ids.
    pub total_locked: Decimal,
    /// The supporting containers.
    pub evidence: BTreeMap<ResourceContainerId, (Rc<RefCell<FungibleResource>>, Decimal)>,
}

impl FungibleProofSubstate {
    pub fn new(
        resource_address: ResourceAddress,
        total_locked: Decimal,
        evidence: BTreeMap<ResourceContainerId, (Rc<RefCell<FungibleResource>>, Decimal)>,
    ) -> Result<FungibleProofSubstate, ProofError> {
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

    fn compute_max_locked(
        proofs: &[FungibleProofSubstate],
        resource_address: ResourceAddress,
    ) -> (Decimal, BTreeMap<ResourceContainerId, Decimal>) {
        // filter proofs by resource address and restricted flag
        let proofs: Vec<&FungibleProofSubstate> = proofs
            .iter()
            .filter(|p| p.resource_address() == resource_address && !p.is_restricted())
            .collect();

        // calculate the max locked amount of each container
        let mut max = BTreeMap::<ResourceContainerId, Decimal>::new();
        for proof in &proofs {
            for (container_id, (_, locked_amount)) in &proof.evidence {
                if let Some(existing) = max.get_mut(container_id) {
                    *existing = Decimal::max(*existing, locked_amount.clone());
                } else {
                    max.insert(container_id.clone(), locked_amount.clone());
                }
            }
        }
        let total = max
            .values()
            .cloned()
            .reduce(|a, b| a + b)
            .unwrap_or_default();
        let per_container = max.into_iter().collect();
        (total, per_container)
    }

    pub fn compose(
        proofs: &[FungibleProofSubstate],
        resource_address: ResourceAddress,
        amount: Option<Decimal>,
    ) -> Result<FungibleProofSubstate, ProofError> {
        let (total_locked, mut per_container) = Self::compute_max_locked(proofs, resource_address);
        let amount = amount.unwrap_or(total_locked);

        // Check if base proofs are sufficient for the request amount
        if amount > total_locked {
            return Err(ProofError::InsufficientBaseProofs);
        }

        // TODO: review resource selection algorithm here
        let mut evidence = BTreeMap::new();
        let mut remaining = amount.clone();
        'outer: for proof in proofs {
            for (container_id, (container, _)) in &proof.evidence {
                if remaining.is_zero() {
                    break 'outer;
                }

                if let Some(quota) = per_container.remove(container_id) {
                    let amount = Decimal::min(remaining, quota);
                    container
                        .borrow_mut()
                        .lock_by_amount(amount)
                        .map_err(ProofError::ResourceError)?;
                    remaining -= amount;
                    evidence.insert(container_id.clone(), (container.clone(), amount));
                }
            }
        }

        FungibleProofSubstate::new(resource_address, amount, evidence)
    }

    /// Makes a clone of this proof.
    ///
    /// Note that cloning a proof will update the ref count of the locked
    /// resources in the source containers.
    pub fn clone_proof(&self) -> Self {
        for (_, (container, locked_amount)) in &self.evidence {
            container
                .borrow_mut()
                .lock_by_amount(*locked_amount)
                .expect("Failed to clone a proof");
        }
        Self {
            resource_address: self.resource_address.clone(),
            restricted: self.restricted,
            total_locked: self.total_locked.clone(),
            evidence: self.evidence.clone(),
        }
    }

    pub fn drop_proof(&mut self) {
        for (_, (container, amount)) in &mut self.evidence {
            container.borrow_mut().unlock(*amount);
        }
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
pub struct NonFungibleProofSubstate {
    /// The resource address.
    pub resource_address: ResourceAddress,
    /// Whether movement of this proof is restricted.
    pub restricted: bool,
    /// The total locked amount or non-fungible ids.
    pub total_locked: BTreeSet<NonFungibleLocalId>,
    /// The supporting containers.
    pub evidence: BTreeMap<
        ResourceContainerId,
        (
            Rc<RefCell<NonFungibleResource>>,
            BTreeSet<NonFungibleLocalId>,
        ),
    >,
}

impl NonFungibleProofSubstate {
    pub fn new(
        resource_address: ResourceAddress,
        total_locked: BTreeSet<NonFungibleLocalId>,
        evidence: BTreeMap<
            ResourceContainerId,
            (
                Rc<RefCell<NonFungibleResource>>,
                BTreeSet<NonFungibleLocalId>,
            ),
        >,
    ) -> Result<NonFungibleProofSubstate, ProofError> {
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

    /// Computes the locked amount or non-fungible IDs, in total and per resource container.
    pub fn compute_max_locked(
        proofs: &[NonFungibleProofSubstate],
        resource_address: ResourceAddress,
    ) -> (
        BTreeSet<NonFungibleLocalId>,
        HashMap<ResourceContainerId, BTreeSet<NonFungibleLocalId>>,
    ) {
        // filter proofs by resource address and restricted flag
        let proofs: Vec<&NonFungibleProofSubstate> = proofs
            .iter()
            .filter(|p| p.resource_address() == resource_address && !p.is_restricted())
            .collect();

        // calculate the max locked amount (or ids) of each container
        let mut max = HashMap::<ResourceContainerId, BTreeSet<NonFungibleLocalId>>::new();
        for proof in &proofs {
            for (container_id, (_, locked_ids)) in &proof.evidence {
                let new_ids = locked_ids.clone();
                if let Some(ids) = max.get_mut(container_id) {
                    ids.extend(new_ids);
                } else {
                    max.insert(container_id.clone(), new_ids);
                }
            }
        }
        let mut total = BTreeSet::<NonFungibleLocalId>::new();
        for value in max.values() {
            total.extend(value.clone());
        }
        let per_container = max.into_iter().collect();
        (total, per_container)
    }

    pub fn compose_by_amount(
        proofs: &[NonFungibleProofSubstate],
        resource_address: ResourceAddress,
        amount: Option<Decimal>,
    ) -> Result<NonFungibleProofSubstate, ProofError> {
        let (total_locked, mut per_container) = Self::compute_max_locked(proofs, resource_address);
        let total_amount = total_locked.len().into();
        let amount = amount.unwrap_or(total_amount);

        if amount > total_amount {
            Err(ProofError::InsufficientBaseProofs)
        } else {
            let n: usize = amount
                .to_string()
                .parse()
                .expect("Failed to convert non-fungible amount to usize");
            let ids: BTreeSet<NonFungibleLocalId> = total_locked.iter().take(n).cloned().collect();
            Self::compose_by_ids(proofs, resource_address, Some(ids))
        }
    }

    pub fn compose_by_ids(
        proofs: &[NonFungibleProofSubstate],
        resource_address: ResourceAddress,
        ids: Option<BTreeSet<NonFungibleLocalId>>,
    ) -> Result<NonFungibleProofSubstate, ProofError> {
        let (total_locked, mut per_container) = Self::compute_max_locked(proofs, resource_address);
        let ids = ids.unwrap_or(total_locked.clone());

        if !total_locked.is_superset(&ids) {
            return Err(ProofError::InsufficientBaseProofs);
        }

        // Locked the max (or needed) ids from the containers, in the order that the containers were referenced.
        // TODO: observe the performance/feedback of this container selection algorithm and decide next steps
        let mut evidence = HashMap::new();
        let mut remaining = ids.clone();
        'outer: for proof in proofs {
            for (container_id, (container, _)) in &proof.evidence {
                if remaining.is_empty() {
                    break 'outer;
                }

                if let Some(quota) = per_container.remove(container_id) {
                    let ids = remaining.intersection(&quota).cloned().collect();
                    container
                        .borrow_mut()
                        .lock_by_ids(&ids)
                        .map_err(ProofError::ResourceError)?;
                    for id in &ids {
                        remaining.remove(id);
                    }
                    evidence.insert(container_id.clone(), (container.clone(), ids));
                }
            }
        }

        NonFungibleProofSubstate::new(resource_address, ids.clone(), evidence)
    }

    /// Makes a clone of this proof.
    ///
    /// Note that cloning a proof will update the ref count of the locked
    /// resources in the source containers.
    pub fn clone_proof(&self) -> Self {
        for (_, (container, locked_ids)) in &self.evidence {
            container
                .borrow_mut()
                .lock_by_ids(locked_ids)
                .expect("Failed to clone a proof");
        }
        Self {
            resource_address: self.resource_address.clone(),
            restricted: self.restricted,
            total_locked: self.total_locked.clone(),
            evidence: self.evidence.clone(),
        }
    }

    pub fn drop_proof(&mut self) {
        for (_, (container, locked_ids)) in &mut self.evidence {
            container.borrow_mut().unlock(locked_ids);
        }
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
        receiver: ProofId,
        input: ScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi
            + KernelSubstateApi
            + ClientSubstateApi<RuntimeError>
            + ClientApi<RuntimeError>
            + ClientNativeInvokeApi<RuntimeError>,
    {
        // TODO: Remove decode/encode mess
        let _input: ProofCloneInput = scrypto_decode(&scrypto_encode(&input).unwrap())
            .map_err(|_| RuntimeError::InterpreterError(InterpreterError::InvalidInvocation))?;

        let handle = api.kernel_lock_substate(
            RENodeId::Proof(receiver),
            NodeModuleId::SELF,
            SubstateOffset::Proof(ProofOffset::Proof),
            LockFlags::read_only(),
        )?;
        let substate_ref = api.kernel_get_substate_ref(handle)?;
        let proof = substate_ref.proof();
        let cloned_proof = proof.clone();

        let node_id = api.kernel_allocate_node_id(RENodeType::Proof)?;
        api.kernel_create_node(node_id, RENodeInit::Proof(cloned_proof), BTreeMap::new())?;
        let proof_id = node_id.into();

        Ok(IndexedScryptoValue::from_typed(&Proof(proof_id)))
    }

    pub(crate) fn get_amount<Y>(
        receiver: ProofId,
        input: ScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi
            + KernelSubstateApi
            + ClientSubstateApi<RuntimeError>
            + ClientApi<RuntimeError>
            + ClientNativeInvokeApi<RuntimeError>,
    {
        // TODO: Remove decode/encode mess
        let _input: ProofGetAmountInput = scrypto_decode(&scrypto_encode(&input).unwrap())
            .map_err(|_| RuntimeError::InterpreterError(InterpreterError::InvalidInvocation))?;

        let handle = api.kernel_lock_substate(
            RENodeId::Proof(receiver),
            NodeModuleId::SELF,
            SubstateOffset::Proof(ProofOffset::Proof),
            LockFlags::read_only(),
        )?;
        let substate_ref = api.kernel_get_substate_ref(handle)?;
        let proof = substate_ref.proof();
        Ok(IndexedScryptoValue::from_typed(&proof.total_amount()))
    }

    pub(crate) fn get_non_fungible_local_ids<Y>(
        receiver: ProofId,
        input: ScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi
            + KernelSubstateApi
            + ClientSubstateApi<RuntimeError>
            + ClientApi<RuntimeError>
            + ClientNativeInvokeApi<RuntimeError>,
    {
        // TODO: Remove decode/encode mess
        let _input: ProofGetNonFungibleLocalIdsInput =
            scrypto_decode(&scrypto_encode(&input).unwrap())
                .map_err(|_| RuntimeError::InterpreterError(InterpreterError::InvalidInvocation))?;

        let handle = api.kernel_lock_substate(
            RENodeId::Proof(receiver),
            NodeModuleId::SELF,
            SubstateOffset::Proof(ProofOffset::Proof),
            LockFlags::read_only(),
        )?;
        let substate_ref = api.kernel_get_substate_ref(handle)?;
        let proof = substate_ref.proof();
        let ids = proof.total_ids()?;
        Ok(IndexedScryptoValue::from_typed(&ids))
    }

    pub(crate) fn get_resource_address<Y>(
        receiver: ProofId,
        input: ScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi
            + KernelSubstateApi
            + ClientSubstateApi<RuntimeError>
            + ClientApi<RuntimeError>
            + ClientNativeInvokeApi<RuntimeError>,
    {
        // TODO: Remove decode/encode mess
        let _input: ProofGetResourceAddressInput = scrypto_decode(&scrypto_encode(&input).unwrap())
            .map_err(|_| RuntimeError::InterpreterError(InterpreterError::InvalidInvocation))?;

        let handle = api.kernel_lock_substate(
            RENodeId::Proof(receiver),
            NodeModuleId::SELF,
            SubstateOffset::Proof(ProofOffset::Proof),
            LockFlags::read_only(),
        )?;
        let substate_ref = api.kernel_get_substate_ref(handle)?;
        let proof = substate_ref.proof();
        Ok(IndexedScryptoValue::from_typed(&proof.resource_address))
    }
}
