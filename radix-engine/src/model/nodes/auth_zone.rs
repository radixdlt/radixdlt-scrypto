use crate::engine::{HeapRENode, SystemApi};
use crate::fee::FeeReserve;
use crate::model::{
    InvokeError, LockableResource, LockedAmountOrIds, Proof, ProofError, ResourceContainerId,
};
use crate::types::*;
use crate::wasm::*;
use scrypto::resource::AuthZoneDrainInput;

#[derive(Debug, Clone, PartialEq, Eq, TypeId, Encode, Decode)]
pub enum AuthZoneError {
    EmptyAuthZone,
    ProofError(ProofError),
    CouldNotCreateProof,
    InvalidRequestData(DecodeError),
    CouldNotGetProof,
    CouldNotGetResource,
    NoMethodSpecified,
}

/// A transient resource container.
#[derive(Debug)]
pub struct AuthZone {
    pub proofs: Vec<Proof>,
    /// IDs of buckets that act as an evidence for virtual proofs.
    /// A virtual proof for any NonFunbigleId can be created for any ResourceAddress in the map.
    /// Note: when a virtual proof is created,
    /// the resources aren't actually being added to the bucket.
    pub virtual_proofs_buckets: BTreeMap<ResourceAddress, BucketId>,
}

impl AuthZone {
    pub fn new_with_proofs(
        proofs: Vec<Proof>,
        virtual_proofs_buckets: BTreeMap<ResourceAddress, BucketId>,
    ) -> Self {
        Self {
            proofs,
            virtual_proofs_buckets,
        }
    }

    pub fn new() -> Self {
        Self {
            proofs: Vec::new(),
            virtual_proofs_buckets: BTreeMap::new(),
        }
    }

    pub fn is_proof_virtualizable(&self, resource_address: &ResourceAddress) -> bool {
        self.virtual_proofs_buckets.contains_key(resource_address)
    }

    fn virtualize_non_fungible_proof(
        &self,
        resource_address: &ResourceAddress,
        ids: &BTreeSet<NonFungibleId>,
    ) -> Proof {
        let bucket_id = self
            .virtual_proofs_buckets
            .get(resource_address)
            .expect("Failed to create a virtual proof (bucket does not exist)")
            .clone();

        let mut locked_ids = BTreeMap::new();
        for id in ids.clone() {
            locked_ids.insert(id, 0);
        }
        let mut evidence = HashMap::new();
        evidence.insert(
            ResourceContainerId::Bucket(bucket_id),
            (
                Rc::new(RefCell::new(LockableResource::NonFungible {
                    resource_address: resource_address.clone(),
                    locked_ids: locked_ids,
                    liquid_ids: BTreeSet::new(),
                })),
                LockedAmountOrIds::Ids(ids.clone()),
            ),
        );
        Proof::new(
            resource_address.clone(),
            ResourceType::NonFungible,
            LockedAmountOrIds::Ids(ids.clone()),
            evidence,
        )
        .expect("Failed to create a virtual proof")
    }

    fn pop(&mut self) -> Result<Proof, InvokeError<AuthZoneError>> {
        if self.proofs.is_empty() {
            return Err(InvokeError::Error(AuthZoneError::EmptyAuthZone));
        }

        Ok(self.proofs.remove(self.proofs.len() - 1))
    }

    fn push(&mut self, proof: Proof) {
        self.proofs.push(proof);
    }

    pub fn drain(&mut self) -> Vec<Proof> {
        self.proofs.drain(0..).collect()
    }

    pub fn clear(&mut self) {
        loop {
            if let Some(proof) = self.proofs.pop() {
                proof.drop();
            } else {
                break;
            }
        }
    }

    fn create_proof(
        &self,
        resource_address: ResourceAddress,
        resource_type: ResourceType,
    ) -> Result<Proof, InvokeError<AuthZoneError>> {
        Proof::compose(&self.proofs, resource_address, resource_type)
            .map_err(|e| InvokeError::Error(AuthZoneError::ProofError(e)))
    }

    fn create_proof_by_amount(
        &self,
        amount: Decimal,
        resource_address: ResourceAddress,
        resource_type: ResourceType,
    ) -> Result<Proof, InvokeError<AuthZoneError>> {
        Proof::compose_by_amount(&self.proofs, amount, resource_address, resource_type)
            .map_err(|e| InvokeError::Error(AuthZoneError::ProofError(e)))
    }

    fn create_proof_by_ids(
        &self,
        ids: &BTreeSet<NonFungibleId>,
        resource_address: ResourceAddress,
        resource_type: ResourceType,
    ) -> Result<Proof, InvokeError<AuthZoneError>> {
        Proof::compose_by_ids(&self.proofs, ids, resource_address, resource_type)
            .map_err(|e| InvokeError::Error(AuthZoneError::ProofError(e)))
    }

    pub fn main<'s, Y, W, I, R>(
        auth_zone_id: AuthZoneId,
        method: AuthZoneMethod,
        args: ScryptoValue,
        system_api: &mut Y,
    ) -> Result<ScryptoValue, InvokeError<AuthZoneError>>
    where
        Y: SystemApi<'s, W, I, R>,
        W: WasmEngine<I>,
        I: WasmInstance,
        R: FeeReserve,
    {
        let node_id = RENodeId::AuthZone(auth_zone_id);
        let offset = SubstateOffset::AuthZone(AuthZoneOffset::AuthZone);
        let auth_zone_handle = system_api.lock_substate(node_id, offset, true, false)?;

        let rtn = match method {
            AuthZoneMethod::Pop => {
                let _: AuthZonePopInput = scrypto_decode(&args.raw)
                    .map_err(|e| InvokeError::Error(AuthZoneError::InvalidRequestData(e)))?;

                let proof = {
                    let mut substate_mut = system_api.get_ref_mut(auth_zone_handle)?;
                    let mut raw_mut = substate_mut.get_raw_mut();
                    let auth_zone = raw_mut.auth_zone();
                    let proof = auth_zone.pop()?;
                    substate_mut.flush()?;
                    proof
                };

                let proof_id = system_api.node_create(HeapRENode::Proof(proof))?.into();
                ScryptoValue::from_typed(&scrypto::resource::Proof(proof_id))
            }
            AuthZoneMethod::Push => {
                let input: AuthZonePushInput = scrypto_decode(&args.raw)
                    .map_err(|e| InvokeError::Error(AuthZoneError::InvalidRequestData(e)))?;
                let mut proof: Proof = system_api.node_drop(RENodeId::Proof(input.proof.0))?.into();
                proof.change_to_unrestricted();

                let mut substate_mut = system_api.get_ref_mut(auth_zone_handle)?;
                let mut raw_mut = substate_mut.get_raw_mut();
                let auth_zone = raw_mut.auth_zone();
                auth_zone.push(proof);
                substate_mut.flush()?;

                ScryptoValue::from_typed(&())
            }
            AuthZoneMethod::CreateProof => {
                let input: AuthZoneCreateProofInput = scrypto_decode(&args.raw)
                    .map_err(|e| InvokeError::Error(AuthZoneError::InvalidRequestData(e)))?;

                let resource_type = {
                    let resource_id =
                        RENodeId::Global(GlobalAddress::Resource(input.resource_address));
                    let offset =
                        SubstateOffset::ResourceManager(ResourceManagerOffset::ResourceManager);
                    let resource_handle = system_api.lock_substate(resource_id, offset, false, false)?;
                    let substate_ref = system_api.get_ref(resource_handle)?;
                    substate_ref.resource_manager().resource_type
                };

                let proof = {
                    let mut substate_mut = system_api.get_ref_mut(auth_zone_handle)?;
                    let mut raw_mut = substate_mut.get_raw_mut();
                    let auth_zone = raw_mut.auth_zone();
                    let proof = auth_zone.create_proof(input.resource_address, resource_type)?;
                    substate_mut.flush()?;
                    proof
                };

                let proof_id = system_api.node_create(HeapRENode::Proof(proof))?.into();
                ScryptoValue::from_typed(&scrypto::resource::Proof(proof_id))
            }
            AuthZoneMethod::CreateProofByAmount => {
                let input: AuthZoneCreateProofByAmountInput = scrypto_decode(&args.raw)
                    .map_err(|e| InvokeError::Error(AuthZoneError::InvalidRequestData(e)))?;

                let resource_type = {
                    let resource_id =
                        RENodeId::Global(GlobalAddress::Resource(input.resource_address));
                    let offset =
                        SubstateOffset::ResourceManager(ResourceManagerOffset::ResourceManager);
                    let resource_handle = system_api.lock_substate(resource_id, offset, false, false)?;
                    let substate_ref = system_api.get_ref(resource_handle)?;
                    substate_ref.resource_manager().resource_type
                };

                let proof = {
                    let mut substate_mut = system_api.get_ref_mut(auth_zone_handle)?;
                    let mut raw_mut = substate_mut.get_raw_mut();
                    let auth_zone = raw_mut.auth_zone();
                    let proof = auth_zone.create_proof_by_amount(
                        input.amount,
                        input.resource_address,
                        resource_type,
                    )?;
                    substate_mut.flush()?;
                    proof
                };

                let proof_id = system_api.node_create(HeapRENode::Proof(proof))?.into();
                ScryptoValue::from_typed(&scrypto::resource::Proof(proof_id))
            }
            AuthZoneMethod::CreateProofByIds => {
                let input: AuthZoneCreateProofByIdsInput = scrypto_decode(&args.raw)
                    .map_err(|e| InvokeError::Error(AuthZoneError::InvalidRequestData(e)))?;

                let resource_type = {
                    let resource_id =
                        RENodeId::Global(GlobalAddress::Resource(input.resource_address));
                    let offset =
                        SubstateOffset::ResourceManager(ResourceManagerOffset::ResourceManager);
                    let resource_handle = system_api.lock_substate(resource_id, offset, false, false)?;
                    let substate_ref = system_api.get_ref(resource_handle)?;
                    substate_ref.resource_manager().resource_type
                };

                let proof = {
                    let mut substate_mut = system_api.get_ref_mut(auth_zone_handle)?;
                    let mut raw_mut = substate_mut.get_raw_mut();
                    let auth_zone = raw_mut.auth_zone();
                    let maybe_existing_proof = auth_zone.create_proof_by_ids(
                        &input.ids,
                        input.resource_address,
                        resource_type,
                    );
                    let proof = match maybe_existing_proof {
                        Ok(proof) => proof,
                        Err(_) if auth_zone.is_proof_virtualizable(&input.resource_address) => {
                            auth_zone
                                .virtualize_non_fungible_proof(&input.resource_address, &input.ids)
                        }
                        Err(e) => Err(e)?,
                    };
                    substate_mut.flush()?;
                    proof
                };

                let proof_id = system_api.node_create(HeapRENode::Proof(proof))?.into();
                ScryptoValue::from_typed(&scrypto::resource::Proof(proof_id))
            }
            AuthZoneMethod::Clear => {
                let _: AuthZoneClearInput = scrypto_decode(&args.raw)
                    .map_err(|e| InvokeError::Error(AuthZoneError::InvalidRequestData(e)))?;
                let mut substate_mut = system_api.get_ref_mut(auth_zone_handle)?;
                let mut raw_mut = substate_mut.get_raw_mut();
                let auth_zone = raw_mut.auth_zone();
                auth_zone.clear();
                substate_mut.flush()?;
                ScryptoValue::from_typed(&())
            }
            AuthZoneMethod::Drain => {
                let _: AuthZoneDrainInput = scrypto_decode(&args.raw)
                    .map_err(|e| InvokeError::Error(AuthZoneError::InvalidRequestData(e)))?;

                let proofs = {
                    let mut substate_mut = system_api.get_ref_mut(auth_zone_handle)?;
                    let mut raw_mut = substate_mut.get_raw_mut();
                    let auth_zone = raw_mut.auth_zone();
                    let proofs = auth_zone.drain();
                    substate_mut.flush()?;
                    proofs
                };

                let mut proof_ids: Vec<scrypto::resource::Proof> = Vec::new();
                for proof in proofs {
                    let proof_id: ProofId =
                        system_api.node_create(HeapRENode::Proof(proof))?.into();
                    proof_ids.push(scrypto::resource::Proof(proof_id));
                }

                ScryptoValue::from_typed(&proof_ids)
            }
        };

        Ok(rtn)
    }
}
