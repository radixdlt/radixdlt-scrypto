use crate::errors::RuntimeError;
use crate::types::*;
use radix_engine_interface::api::substate_lock_api::LockFlags;
use radix_engine_interface::api::ClientApi;
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
pub struct ProofMoveableSubstate {
    /// Whether movement of this proof is restricted.
    pub restricted: bool,
}

impl ProofMoveableSubstate {
    pub fn of_self<Y: ClientApi<RuntimeError>>(api: &mut Y) -> Result<Self, RuntimeError> {
        let handle = api.lock_field(FungibleProofOffset::Moveable.into(), LockFlags::read_only())?;
        let substate_ref: ProofMoveableSubstate = api.sys_read_substate_typed(handle)?;
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
        for (container, locked_amount) in &self.evidence {
            api.call_method(
                container.as_node_id(),
                match container {
                    LocalRef::Bucket(_) => FUNGIBLE_BUCKET_LOCK_AMOUNT_IDENT,
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
                    LocalRef::Bucket(_) => FUNGIBLE_BUCKET_UNLOCK_AMOUNT_IDENT,
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
        for (container, locked_ids) in &self.evidence {
            api.call_method(
                container.as_node_id(),
                match container {
                    LocalRef::Bucket(_) => NON_FUNGIBLE_BUCKET_LOCK_NON_FUNGIBLES_IDENT,
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
                    LocalRef::Bucket(_) => NON_FUNGIBLE_BUCKET_UNLOCK_NON_FUNGIBLES_IDENT,
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

pub struct FungibleProofBlueprint;

impl FungibleProofBlueprint {
    pub(crate) fn clone<Y>(
        api: &mut Y,
    ) -> Result<Proof, RuntimeError>
        where
            Y: ClientApi<RuntimeError>,
    {
        let proof_info = ProofMoveableSubstate::of_self(api)?;
        let handle = api.lock_field(FungibleProofOffset::ProofRef.into(), LockFlags::read_only())?;
        let substate_ref: FungibleProof = api.sys_read_substate_typed(handle)?;
        let proof = substate_ref.clone();
        let clone = proof.clone_proof(api)?;

        let proof_id = api.new_object(
            FUNGIBLE_PROOF_BLUEPRINT,
            vec![
                scrypto_encode(&proof_info).unwrap(),
                scrypto_encode(&clone).unwrap(),
            ],
        )?;

        // Drop after object creation to keep the reference alive
        api.sys_drop_lock(handle)?;

        Ok(Proof(Own(proof_id)))
    }


    pub(crate) fn get_amount<Y>(
        api: &mut Y,
    ) -> Result<Decimal, RuntimeError>
        where
            Y: ClientApi<RuntimeError>,
    {
        let handle = api.lock_field(FungibleProofOffset::ProofRef.into(), LockFlags::read_only())?;
        let substate_ref: FungibleProof = api.sys_read_substate_typed(handle)?;
        let amount = substate_ref.amount();
        api.sys_drop_lock(handle)?;
        Ok(amount)
    }

    pub(crate) fn get_resource_address<Y>(
        api: &mut Y,
    ) -> Result<ResourceAddress, RuntimeError>
        where
            Y: ClientApi<RuntimeError>,
    {
        let address = ResourceAddress::new_or_panic(api.get_info()?.blueprint_parent.unwrap().into());
        Ok(address)
    }

    pub(crate) fn drop<Y>(
        proof: Proof,
        api: &mut Y,
    ) -> Result<(), RuntimeError>
        where
            Y: ClientApi<RuntimeError>,
    {
        // FIXME: check type before schema check is ready! applicable to all functions!

        let parent = api.get_object_info(proof.0.as_node_id())?.blueprint_parent.unwrap();

        api.call_method(
            parent.as_node_id(),
            RESOURCE_MANAGER_DROP_PROOF_IDENT,
            scrypto_encode(&ResourceManagerDropProofInput {
                proof
            }).unwrap()
        )?;

        Ok(())
    }
}

pub struct NonFungibleProofBlueprint;

impl NonFungibleProofBlueprint {
    pub(crate) fn clone<Y>(
        api: &mut Y,
    ) -> Result<Proof, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let proof_info = ProofMoveableSubstate::of_self(api)?;
        let handle = api.lock_field(NonFungibleProofOffset::ProofRef.into(), LockFlags::read_only())?;
        let substate_ref: NonFungibleProof = api.sys_read_substate_typed(handle)?;
        let proof = substate_ref.clone();
        let clone = proof.clone_proof(api)?;

        let proof_id = api.new_object(
            NON_FUNGIBLE_PROOF_BLUEPRINT,
            vec![
                scrypto_encode(&proof_info).unwrap(),
                scrypto_encode(&clone).unwrap(),
            ],
        )?;

        // Drop after object creation to keep the reference alive
        api.sys_drop_lock(handle)?;

        Ok(Proof(Own(proof_id)))
    }

    pub(crate) fn get_amount<Y>(
        api: &mut Y,
    ) -> Result<Decimal, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let handle = api.lock_field(NonFungibleProofOffset::ProofRef.into(), LockFlags::read_only())?;
        let substate_ref: NonFungibleProof = api.sys_read_substate_typed(handle)?;
        let amount = substate_ref.amount();
        api.sys_drop_lock(handle)?;
        Ok(amount)
    }

    pub(crate) fn get_local_ids<Y>(
        api: &mut Y,
    ) -> Result<BTreeSet<NonFungibleLocalId>, RuntimeError>
        where Y: ClientApi<RuntimeError>,
    {
        let handle = api.lock_field(NonFungibleProofOffset::ProofRef.into(), LockFlags::read_only())?;
        let substate_ref: NonFungibleProof = api.sys_read_substate_typed(handle)?;
        let ids = substate_ref.non_fungible_local_ids().clone();
        api.sys_drop_lock(handle)?;
        Ok(ids)
    }

    pub(crate) fn get_resource_address<Y>(
        api: &mut Y,
    ) -> Result<ResourceAddress, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let address = ResourceAddress::new_or_panic(api.get_info()?.blueprint_parent.unwrap().into());
        Ok(address)
    }

    pub(crate) fn drop<Y>(
        proof: Proof,
        api: &mut Y,
    ) -> Result<(), RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        // FIXME: check type before schema check is ready! applicable to all functions!
        let parent = api.get_object_info(proof.0.as_node_id())?.blueprint_parent.unwrap();

        api.call_method(
            parent.as_node_id(),
            RESOURCE_MANAGER_DROP_PROOF_IDENT,
            scrypto_encode(&ResourceManagerDropProofInput {
                proof
            }).unwrap()
        )?;

        Ok(())
    }
}
