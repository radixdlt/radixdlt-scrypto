use colored::*;
use sbor::path::SborPath;
use sbor::rust::borrow::ToOwned;
use sbor::rust::cell::RefCell;
use sbor::rust::collections::*;
use sbor::rust::format;
use sbor::rust::marker::*;
use sbor::rust::rc::Rc;
use sbor::rust::string::String;
use sbor::rust::string::ToString;
use sbor::rust::vec;
use sbor::rust::vec::Vec;
use sbor::*;
use scrypto::core::{SNodeRef, ScryptoActor};
use scrypto::engine::types::*;
use scrypto::resource::AuthZoneMethod;
use scrypto::values::*;

use crate::engine::LazyMapState::{Committed, Uncommitted};
use crate::engine::LoadedSNodeState::{Borrowed, Consumed, Static};
use crate::engine::*;
use crate::ledger::*;
use crate::model::*;
use crate::wasm::*;

/// A call frame is the basic unit that forms a transaction call stack. It keeps track of the
/// owned objects by this function.
///
/// A call frame can be either native or wasm (when the callee is a blueprint or component).
///
/// Radix Engine manages the lifecycle of call frames and enforces the call and move semantics.
pub struct CallFrame<
    'p, // parent frame lifetime
    's, // substate store lifetime
    S,  // substore store type
    W,  // Scrypto wasm engine type
> where
    S: ReadableSubstateStore,
    W: WasmEngine,
{
    /// The transaction hash
    transaction_hash: Hash,
    /// The call depth
    depth: usize,
    /// Whether to show trace messages
    trace: bool,

    /// State track
    track: Rc<RefCell<Track<'s, S>>>,
    /// Wasm engine
    wasm_engine: Rc<RefCell<W>>,

    /// Owned Snodes
    buckets: HashMap<BucketId, Bucket>,
    proofs: HashMap<ProofId, Proof>,
    owned_snodes: ComponentObjects,

    /// Referenced Snodes
    worktop: Option<Worktop>,
    auth_zone: Option<AuthZone>,

    /// Caller's auth zone
    caller_auth_zone: Option<&'p AuthZone>,

    /// Component state, lazily loaded
    component_state: Option<ComponentState>,
}

pub enum ConsumedSNodeState {
    Bucket(Bucket),
    Proof(Proof),
}

pub enum BorrowedSNodeState {
    AuthZone(AuthZone),
    Worktop(Worktop),
    Scrypto(ScryptoActorInfo, Package, String, Option<ComponentState>),
    Resource(ResourceAddress, ResourceManager),
    Bucket(BucketId, Bucket),
    Proof(ProofId, Proof),
    Vault(VaultId, Option<ComponentAddress>, Vault),
}

pub enum StaticSNodeState {
    Package,
    Resource,
    System,
    TransactionProcessor,
}

pub enum LoadedSNodeState {
    Static(StaticSNodeState),
    Consumed(Option<ConsumedSNodeState>),
    Borrowed(BorrowedSNodeState),
}

pub enum SNodeState<'a> {
    Root,
    SystemStatic,
    TransactionProcessorStatic,
    PackageStatic,
    AuthZoneRef(&'a mut AuthZone),
    Worktop(&'a mut Worktop),
    // TODO: use reference to the package
    Scrypto(ScryptoActorInfo, Package, String, Option<ComponentState>),
    ResourceStatic,
    ResourceRef(ResourceAddress, &'a mut ResourceManager),
    BucketRef(BucketId, &'a mut Bucket),
    Bucket(Bucket),
    ProofRef(ProofId, &'a mut Proof),
    Proof(Proof),
    VaultRef(VaultId, Option<ComponentAddress>, &'a mut Vault),
}

#[derive(Debug)]
pub struct ComponentState {
    pub component_address: ComponentAddress,
    pub component: Component,
    pub initial_loaded_object_refs: ComponentObjectRefs,
    pub snode_refs: ComponentObjectRefs,
}

///TODO: Remove
#[derive(Debug)]
pub enum LazyMapState {
    Uncommitted { root: LazyMapId },
    Committed { component_address: ComponentAddress },
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum MoveMethod {
    AsReturn,
    AsArgument,
}

impl LoadedSNodeState {
    fn to_snode_state(&mut self) -> SNodeState {
        match self {
            Static(static_state) => match static_state {
                StaticSNodeState::Package => SNodeState::PackageStatic,
                StaticSNodeState::Resource => SNodeState::ResourceStatic,
                StaticSNodeState::System => SNodeState::SystemStatic,
                StaticSNodeState::TransactionProcessor => SNodeState::TransactionProcessorStatic,
            },
            Consumed(ref mut to_consume) => match to_consume.take().unwrap() {
                ConsumedSNodeState::Proof(proof) => SNodeState::Proof(proof),
                ConsumedSNodeState::Bucket(bucket) => SNodeState::Bucket(bucket),
            },
            Borrowed(ref mut borrowed) => match borrowed {
                BorrowedSNodeState::AuthZone(s) => SNodeState::AuthZoneRef(s),
                BorrowedSNodeState::Worktop(s) => SNodeState::Worktop(s),
                BorrowedSNodeState::Scrypto(info, package, export_name, component_state) => {
                    SNodeState::Scrypto(
                        info.clone(),
                        package.clone(),
                        export_name.clone(),
                        component_state.take(),
                    )
                }
                BorrowedSNodeState::Resource(addr, s) => SNodeState::ResourceRef(*addr, s),
                BorrowedSNodeState::Bucket(id, s) => SNodeState::BucketRef(*id, s),
                BorrowedSNodeState::Proof(id, s) => SNodeState::ProofRef(*id, s),
                BorrowedSNodeState::Vault(id, addr, s) => {
                    SNodeState::VaultRef(*id, addr.clone(), s)
                }
            },
        }
    }
}

impl BorrowedSNodeState {
    fn return_borrowed_state<'p, 's, S, W>(self, frame: &mut CallFrame<'p, 's, S, W>)
    where
        S: ReadableSubstateStore,
        W: WasmEngine,
    {
        match self {
            BorrowedSNodeState::AuthZone(auth_zone) => {
                frame.auth_zone = Some(auth_zone);
            }
            BorrowedSNodeState::Worktop(worktop) => {
                frame.worktop = Some(worktop);
            }
            BorrowedSNodeState::Scrypto(actor, _, _, component_state) => {
                if let Some(component_address) = actor.component_address() {
                    frame.track.borrow_mut().return_borrowed_global_mut_value(
                        component_address,
                        component_state.unwrap().component, // TODO: how about the refs?
                    );
                }
            }
            BorrowedSNodeState::Resource(resource_address, resource_manager) => {
                frame
                    .track
                    .borrow_mut()
                    .return_borrowed_global_mut_value(resource_address, resource_manager);
            }
            BorrowedSNodeState::Bucket(bucket_id, bucket) => {
                frame.buckets.insert(bucket_id, bucket);
            }
            BorrowedSNodeState::Proof(proof_id, proof) => {
                frame.proofs.insert(proof_id, proof);
            }
            BorrowedSNodeState::Vault(vault_id, maybe_component_address, vault) => {
                if let Some(component_address) = maybe_component_address {
                    frame.track.borrow_mut().return_borrowed_vault(
                        &component_address,
                        &vault_id,
                        vault,
                    );
                } else {
                    frame.owned_snodes.return_borrowed_vault_mut(vault);
                }
            }
        }
    }
}

impl<'p, 's, S, W> CallFrame<'p, 's, S, W>
where
    S: ReadableSubstateStore,
    W: WasmEngine,
{
    pub fn new_root(
        verbose: bool,
        transaction_hash: Hash,
        transaction_signers: Vec<EcdsaPublicKey>,
        track: Rc<RefCell<Track<'s, S>>>,
        wasm_engine: Rc<RefCell<W>>,
    ) -> Self {
        let signers: BTreeSet<NonFungibleId> = transaction_signers
            .clone()
            .into_iter()
            .map(|public_key| NonFungibleId::from_bytes(public_key.to_vec()))
            .collect();

        let mut initial_auth_zone_proofs = Vec::new();
        if !signers.is_empty() {
            // Proofs can't be zero amount
            let mut ecdsa_bucket =
                Bucket::new(ResourceContainer::new_non_fungible(ECDSA_TOKEN, signers));
            let ecdsa_proof = ecdsa_bucket.create_proof(ECDSA_TOKEN_BUCKET_ID).unwrap();
            initial_auth_zone_proofs.push(ecdsa_proof);
        }

        Self::new(
            transaction_hash,
            0,
            verbose,
            track,
            wasm_engine,
            Some(AuthZone::new_with_proofs(initial_auth_zone_proofs)),
            Some(Worktop::new()),
            HashMap::new(),
            HashMap::new(),
            None,
        )
    }

    pub fn new(
        transaction_hash: Hash,
        depth: usize,
        trace: bool,
        track: Rc<RefCell<Track<'s, S>>>,
        wasm_engine: Rc<RefCell<W>>,
        auth_zone: Option<AuthZone>,
        worktop: Option<Worktop>,
        buckets: HashMap<BucketId, Bucket>,
        proofs: HashMap<ProofId, Proof>,
        caller_auth_zone: Option<&'p AuthZone>,
    ) -> Self {
        Self {
            transaction_hash,
            depth,
            trace,
            track,
            wasm_engine,
            buckets,
            proofs,
            owned_snodes: ComponentObjects::new(),
            worktop,
            auth_zone,
            caller_auth_zone,
            component_state: None,
        }
    }

    /// Checks resource leak.
    fn check_resource(&self) -> Result<(), RuntimeError> {
        let success = self.buckets.is_empty()
            && self.owned_snodes.vaults.is_empty()
            && self.owned_snodes.lazy_maps.is_empty()
            && match &self.worktop {
                Some(worktop) => worktop.is_empty(),
                None => true,
            };

        if success {
            Ok(())
        } else {
            Err(RuntimeError::ResourceCheckFailure)
        }
    }

    fn process_call_data(&mut self, validated: &ScryptoValue) -> Result<(), RuntimeError> {
        if !validated.lazy_map_ids.is_empty() {
            return Err(RuntimeError::LazyMapNotAllowed);
        }
        if !validated.vault_ids.is_empty() {
            return Err(RuntimeError::VaultNotAllowed);
        }
        Ok(())
    }

    fn process_return_data(
        &mut self,
        from: Option<SNodeRef>,
        validated: &ScryptoValue,
    ) -> Result<(), RuntimeError> {
        if !validated.lazy_map_ids.is_empty() {
            return Err(RuntimeError::LazyMapNotAllowed);
        }

        // Allow vaults to be returned from ResourceStatic
        // TODO: Should we allow vaults to be returned by any component?
        if !matches!(from, Some(SNodeRef::ResourceRef(_))) {
            if !validated.vault_ids.is_empty() {
                return Err(RuntimeError::VaultNotAllowed);
            }
        }

        Ok(())
    }

    /// Process and parse entry data from any component object (components and maps)
    fn process_entry_data(data: &[u8]) -> Result<ComponentObjectRefs, RuntimeError> {
        let validated =
            ScryptoValue::from_slice(data).map_err(RuntimeError::ParseScryptoValueError)?;
        if !validated.bucket_ids.is_empty() {
            return Err(RuntimeError::BucketNotAllowed);
        }
        if !validated.proof_ids.is_empty() {
            return Err(RuntimeError::ProofNotAllowed);
        }

        // lazy map allowed
        // vaults allowed
        Ok(ComponentObjectRefs {
            lazy_map_ids: validated.lazy_map_ids,
            vault_ids: validated.vault_ids,
        })
    }

    /// Sends buckets to another component/blueprint, either as argument or return
    fn send_buckets(
        &mut self,
        bucket_ids: &HashMap<BucketId, SborPath>,
    ) -> Result<HashMap<BucketId, Bucket>, RuntimeError> {
        let mut buckets = HashMap::new();
        for (bucket_id, _) in bucket_ids {
            let bucket = self
                .buckets
                .remove(bucket_id)
                .ok_or(RuntimeError::BucketNotFound(*bucket_id))?;
            if bucket.is_locked() {
                return Err(RuntimeError::CantMoveLockedBucket);
            }
            buckets.insert(*bucket_id, bucket);
        }
        Ok(buckets)
    }

    /// Sends proofs to another component/blueprint, either as argument or return
    fn send_vaults(
        &mut self,
        vault_ids: &HashSet<VaultId>,
    ) -> Result<HashMap<VaultId, Vault>, RuntimeError> {
        let mut vaults = HashMap::new();
        for vault_id in vault_ids {
            let vault = self
                .owned_snodes
                .vaults
                .remove(vault_id)
                .ok_or(RuntimeError::VaultNotFound(*vault_id))?;
            vaults.insert(*vault_id, vault);
        }
        Ok(vaults)
    }

    /// Sends proofs to another component/blueprint, either as argument or return
    fn send_proofs(
        &mut self,
        proof_ids: &HashMap<ProofId, SborPath>,
        method: MoveMethod,
    ) -> Result<HashMap<ProofId, Proof>, RuntimeError> {
        let mut proofs = HashMap::new();
        for (proof_id, _) in proof_ids {
            let mut proof = self
                .proofs
                .remove(proof_id)
                .ok_or(RuntimeError::ProofNotFound(*proof_id))?;
            if proof.is_restricted() {
                return Err(RuntimeError::CantMoveRestrictedProof(*proof_id));
            }
            if matches!(method, MoveMethod::AsArgument) {
                proof.change_to_restricted();
            }
            proofs.insert(*proof_id, proof);
        }
        Ok(proofs)
    }

    pub fn run<'f>(
        &'f mut self,
        snode_ref: Option<SNodeRef>, // TODO: Remove, abstractions between invoke_snode() and run() are a bit messy right now
        snode: SNodeState<'f>,
        call_data: ScryptoValue,
    ) -> Result<
        (
            ScryptoValue,
            HashMap<BucketId, Bucket>,
            HashMap<ProofId, Proof>,
            HashMap<VaultId, Vault>,
        ),
        RuntimeError,
    > {
        let output = match snode {
            SNodeState::Root => {
                panic!("Root is not runnable")
            }
            SNodeState::SystemStatic => {
                System::static_main(call_data, self).map_err(RuntimeError::SystemError)
            }
            SNodeState::TransactionProcessorStatic => {
                TransactionProcessor::static_main(call_data, self).map_err(|e| match e {
                    TransactionProcessorError::InvalidRequestData(_) => panic!("Illegal state"),
                    TransactionProcessorError::RuntimeError(e) => e,
                })
            }
            SNodeState::PackageStatic => {
                Package::static_main(call_data, self, self.wasm_engine.clone())
                    .map_err(RuntimeError::PackageError)
            }
            SNodeState::AuthZoneRef(auth_zone) => auth_zone
                .main(call_data, self)
                .map_err(RuntimeError::AuthZoneError),
            SNodeState::Worktop(worktop) => worktop
                .main(call_data, self)
                .map_err(RuntimeError::WorktopError),
            SNodeState::Scrypto(actor, package, export_name, component_state) => {
                self.component_state = component_state;

                package.invoke(
                    actor,
                    export_name,
                    call_data,
                    self,
                    self.wasm_engine.clone(),
                )
            }
            SNodeState::ResourceStatic => ResourceManager::static_main(call_data, self)
                .map_err(RuntimeError::ResourceManagerError),
            SNodeState::ResourceRef(resource_address, resource_manager) => {
                let return_value = resource_manager
                    .main(resource_address, call_data, self)
                    .map_err(RuntimeError::ResourceManagerError)?;

                Ok(return_value)
            }
            SNodeState::BucketRef(bucket_id, bucket) => bucket
                .main(bucket_id, call_data, self)
                .map_err(RuntimeError::BucketError),
            SNodeState::Bucket(bucket) => bucket
                .consuming_main(call_data, self)
                .map_err(RuntimeError::BucketError),
            SNodeState::ProofRef(_, proof) => proof
                .main(call_data, self)
                .map_err(RuntimeError::ProofError),
            SNodeState::Proof(proof) => proof
                .main_consume(call_data)
                .map_err(RuntimeError::ProofError),
            SNodeState::VaultRef(vault_id, _, vault) => vault
                .main(vault_id, call_data, self)
                .map_err(RuntimeError::VaultError),
        }?;

        self.process_return_data(snode_ref, &output)?;

        // figure out what buckets and resources to return
        let moving_buckets = self.send_buckets(&output.bucket_ids)?;
        let moving_proofs = self.send_proofs(&output.proof_ids, MoveMethod::AsReturn)?;
        let moving_vaults = self.send_vaults(&output.vault_ids)?;

        // drop proofs and check resource leak
        for (_, proof) in self.proofs.drain() {
            proof.drop();
        }

        if let Some(_) = &mut self.auth_zone {
            self.invoke_snode(
                SNodeRef::AuthZoneRef,
                ScryptoValue::from_value(&AuthZoneMethod::Clear()),
            )?;
        }
        self.check_resource()?;

        Ok((output, moving_buckets, moving_proofs, moving_vaults))
    }
}

impl<'p, 's, S, W> SystemApi for CallFrame<'p, 's, S, W>
where
    S: ReadableSubstateStore,
    W: WasmEngine,
{
    fn invoke_snode(
        &mut self,
        snode_ref: SNodeRef,
        call_data: ScryptoValue,
    ) -> Result<ScryptoValue, RuntimeError> {
        self.sys_log(Level::Debug, format!("{:?}", snode_ref));
        let function = if let Value::Enum { name, .. } = &call_data.dom {
            name.clone()
        } else {
            return Err(RuntimeError::InvalidInvocation);
        };

        // Authorization and state load
        let (mut loaded_snode, method_auths) = match &snode_ref {
            SNodeRef::TransactionProcessor => {
                // FIXME: only TransactionExecutor can invoke this function
                Ok((Static(StaticSNodeState::TransactionProcessor), vec![]))
            }
            SNodeRef::PackageStatic => Ok((Static(StaticSNodeState::Package), vec![])),
            SNodeRef::SystemStatic => Ok((Static(StaticSNodeState::System), vec![])),
            SNodeRef::AuthZoneRef => {
                if let Some(auth_zone) = self.auth_zone.take() {
                    Ok((Borrowed(BorrowedSNodeState::AuthZone(auth_zone)), vec![]))
                } else {
                    Err(RuntimeError::AuthZoneDoesNotExist)
                }
            }
            SNodeRef::WorktopRef => {
                if let Some(worktop) = self.worktop.take() {
                    Ok((Borrowed(BorrowedSNodeState::Worktop(worktop)), vec![]))
                } else {
                    Err(RuntimeError::WorktopDoesNotExist)
                }
            }
            SNodeRef::Scrypto(actor) => {
                match actor {
                    ScryptoActor::Blueprint(package_address, blueprint_name) => {
                        let mut track = self.track.borrow_mut();
                        let substate_value = track
                            .read_value(package_address.clone())
                            .ok_or(RuntimeError::PackageNotFound(*package_address))?;
                        let package = match substate_value {
                            SubstateValue::Package(package) => package,
                            _ => panic!("Value is not a package"),
                        };
                        if !package.contains_blueprint(blueprint_name) {
                            return Err(RuntimeError::BlueprintNotFound(
                                package_address.clone(),
                                blueprint_name.clone(),
                            ));
                        }
                        let export_name = format!("{}_main", blueprint_name);

                        Ok((
                            Borrowed(BorrowedSNodeState::Scrypto(
                                ScryptoActorInfo::blueprint(
                                    package_address.clone(),
                                    blueprint_name.clone(),
                                ),
                                package.clone(),
                                export_name.clone(),
                                None,
                            )),
                            vec![],
                        ))
                    }
                    ScryptoActor::Component(component_address) => {
                        let component_address = *component_address;

                        let component: Component = self
                            .track
                            .borrow_mut()
                            .borrow_global_mut_value(component_address)
                            .map_err(|e| match e {
                                TrackError::NotFound => {
                                    RuntimeError::ComponentNotFound(component_address)
                                }
                                TrackError::Reentrancy => {
                                    RuntimeError::ComponentReentrancy(component_address)
                                }
                            })?
                            .into();
                        let package_address = component.package_address();
                        let blueprint_name = component.blueprint_name().to_string();
                        let export_name = format!("{}_main", blueprint_name);

                        let mut track = self.track.borrow_mut();
                        let substate_value = track
                            .read_value(package_address)
                            .ok_or(RuntimeError::PackageNotFound(package_address))?;
                        let package = match substate_value {
                            SubstateValue::Package(package) => package,
                            _ => panic!("Value is not a package"),
                        };

                        // TODO: Remove clone
                        let schema = package
                            .load_blueprint_schema(&blueprint_name)
                            .unwrap()
                            .clone();

                        let (_, method_auths) = component.method_authorization(&schema, &function);

                        // set up component state
                        let data = ScryptoValue::from_slice(component.state()).unwrap();
                        let initial_loaded_object_refs = ComponentObjectRefs {
                            vault_ids: data.vault_ids.into_iter().collect(),
                            lazy_map_ids: data.lazy_map_ids.into_iter().collect(),
                        };
                        let snode_refs = ComponentObjectRefs::new();

                        Ok((
                            Borrowed(BorrowedSNodeState::Scrypto(
                                ScryptoActorInfo::component(
                                    package_address,
                                    blueprint_name,
                                    component_address,
                                ),
                                package.clone(),
                                export_name,
                                Some(ComponentState {
                                    component_address,
                                    component,
                                    initial_loaded_object_refs,
                                    snode_refs,
                                }),
                            )),
                            method_auths,
                        ))
                    }
                }
            }
            SNodeRef::ResourceStatic => Ok((Static(StaticSNodeState::Resource), vec![])),
            SNodeRef::ResourceRef(resource_address) => {
                let resource_manager: ResourceManager = self
                    .track
                    .borrow_mut()
                    .borrow_global_mut_value(resource_address.clone())
                    .map_err(|e| match e {
                        TrackError::NotFound => {
                            RuntimeError::ResourceManagerNotFound(resource_address.clone())
                        }
                        TrackError::Reentrancy => panic!("Reentrancy occurred in resource manager"),
                    })?
                    .into();
                let method_auth = resource_manager.get_auth(&call_data).clone();
                Ok((
                    Borrowed(BorrowedSNodeState::Resource(
                        resource_address.clone(),
                        resource_manager,
                    )),
                    vec![method_auth],
                ))
            }
            SNodeRef::Bucket(bucket_id) => {
                let bucket = self
                    .buckets
                    .remove(&bucket_id)
                    .ok_or(RuntimeError::BucketNotFound(bucket_id.clone()))?;
                let resource_address = bucket.resource_address();
                let mut track = self.track.borrow_mut();
                let substate_value = track.read_value(resource_address.clone()).unwrap();
                let resource_manager = match substate_value {
                    SubstateValue::Resource(resource_manager) => resource_manager,
                    _ => panic!("Value is not a resource manager"),
                };
                let method_auth = resource_manager.get_consuming_bucket_auth(&call_data);
                Ok((
                    Consumed(Some(ConsumedSNodeState::Bucket(bucket))),
                    vec![method_auth.clone()],
                ))
            }
            SNodeRef::BucketRef(bucket_id) => {
                let bucket = self
                    .buckets
                    .remove(&bucket_id)
                    .ok_or(RuntimeError::BucketNotFound(bucket_id.clone()))?;
                Ok((
                    Borrowed(BorrowedSNodeState::Bucket(bucket_id.clone(), bucket)),
                    vec![],
                ))
            }
            SNodeRef::ProofRef(proof_id) => {
                let proof = self
                    .proofs
                    .remove(&proof_id)
                    .ok_or(RuntimeError::ProofNotFound(proof_id.clone()))?;
                Ok((
                    Borrowed(BorrowedSNodeState::Proof(proof_id.clone(), proof)),
                    vec![],
                ))
            }
            SNodeRef::Proof(proof_id) => {
                let proof = self
                    .proofs
                    .remove(&proof_id)
                    .ok_or(RuntimeError::ProofNotFound(proof_id.clone()))?;
                Ok((Consumed(Some(ConsumedSNodeState::Proof(proof))), vec![]))
            }
            SNodeRef::VaultRef(vault_id) => {
                let (component, vault) =
                    if let Some(vault) = self.owned_snodes.borrow_vault_mut(vault_id) {
                        (None, vault)
                    } else if let Some(ComponentState {
                        component_address,
                        snode_refs,
                        ..
                    }) = &self.component_state
                    {
                        if !snode_refs.vault_ids.contains(vault_id) {
                            return Err(RuntimeError::VaultNotFound(*vault_id));
                        }
                        let vault = self
                            .track
                            .borrow_mut()
                            .borrow_vault_mut(component_address, vault_id);
                        (Some(*component_address), vault)
                    } else {
                        panic!("Should never get here");
                    };

                let resource_address = vault.resource_address();
                let mut track = self.track.borrow_mut();
                let substate_value = track.read_value(resource_address.clone()).unwrap();
                let resource_manager = match substate_value {
                    SubstateValue::Resource(resource_manager) => resource_manager,
                    _ => panic!("Value is not a resource manager"),
                };
                let method_auth = resource_manager.get_vault_auth(&call_data);
                Ok((
                    Borrowed(BorrowedSNodeState::Vault(
                        vault_id.clone(),
                        component,
                        vault,
                    )),
                    vec![method_auth.clone()],
                ))
            }
        }?;

        // Authorization check
        if !method_auths.is_empty() {
            let mut auth_zones = Vec::new();
            if let Some(self_auth_zone) = &self.auth_zone {
                auth_zones.push(self_auth_zone);
            }

            match &loaded_snode {
                // Resource auth check includes caller
                Borrowed(BorrowedSNodeState::Resource(_, _))
                | Borrowed(BorrowedSNodeState::Vault(_, _, _))
                | Borrowed(BorrowedSNodeState::Bucket(_, _))
                | Borrowed(BorrowedSNodeState::Scrypto(_, _, _, _))
                | Consumed(Some(ConsumedSNodeState::Bucket(_))) => {
                    if let Some(auth_zone) = self.caller_auth_zone {
                        auth_zones.push(auth_zone);
                    }
                }
                // Extern call auth check
                _ => {}
            };

            for method_auth in method_auths {
                method_auth.check(&auth_zones).map_err(|error| {
                    RuntimeError::AuthorizationError {
                        function: function.clone(),
                        authorization: method_auth,
                        error,
                    }
                })?;
            }
        }

        // Figure out what buckets and proofs to move from this process
        let mut moving_buckets = HashMap::new();
        let mut moving_proofs = HashMap::new();
        self.process_call_data(&call_data)?;
        moving_buckets.extend(self.send_buckets(&call_data.bucket_ids)?);
        moving_proofs.extend(self.send_proofs(&call_data.proof_ids, MoveMethod::AsArgument)?);

        // start a new frame
        let mut frame = CallFrame::new(
            self.transaction_hash,
            self.depth + 1,
            self.trace,
            self.track.clone(),
            self.wasm_engine.clone(),
            match loaded_snode {
                Borrowed(BorrowedSNodeState::Scrypto(_, _, _, _))
                | Static(StaticSNodeState::TransactionProcessor) => Some(AuthZone::new()),
                _ => None,
            },
            match loaded_snode {
                Static(StaticSNodeState::TransactionProcessor) => Some(Worktop::new()),
                _ => None,
            },
            moving_buckets,
            moving_proofs,
            self.auth_zone.as_ref(),
        );

        // invoke the main function
        let snode = loaded_snode.to_snode_state();
        let (result, received_buckets, received_proofs, received_vaults) =
            frame.run(Some(snode_ref), snode, call_data)?;

        // move buckets and proofs to this process.
        self.buckets.extend(received_buckets);
        self.proofs.extend(received_proofs);
        self.owned_snodes.vaults.extend(received_vaults);

        // Return borrowed snodes
        if let Borrowed(borrowed) = loaded_snode {
            borrowed.return_borrowed_state(self);
        }

        Ok(result)
    }

    fn get_non_fungible(
        &mut self,
        non_fungible_address: &NonFungibleAddress,
    ) -> Option<NonFungible> {
        let parent_address = Address::NonFungibleSet(non_fungible_address.resource_address());
        let key = non_fungible_address.non_fungible_id().to_vec();
        if let SubstateValue::NonFungible(non_fungible) =
            self.track.borrow_mut().read_key_value(parent_address, key)
        {
            non_fungible
        } else {
            panic!("Value is not a non fungible");
        }
    }

    fn set_non_fungible(
        &mut self,
        non_fungible_address: NonFungibleAddress,
        non_fungible: Option<NonFungible>,
    ) {
        let parent_address = Address::NonFungibleSet(non_fungible_address.resource_address());
        let key = non_fungible_address.non_fungible_id().to_vec();
        self.track
            .borrow_mut()
            .set_key_value(parent_address, key, non_fungible)
    }

    fn borrow_global_mut_resource_manager(
        &mut self,
        resource_address: ResourceAddress,
    ) -> Result<ResourceManager, RuntimeError> {
        self.track
            .borrow_mut()
            .borrow_global_mut_value(resource_address.clone())
            .map(|v| v.into())
            .map_err(|e| match e {
                TrackError::NotFound => {
                    RuntimeError::ResourceManagerNotFound(resource_address.clone())
                }
                TrackError::Reentrancy => panic!("Reentrancy occurred in resource manager"),
            })
    }

    fn return_borrowed_global_resource_manager(
        &mut self,
        resource_address: ResourceAddress,
        resource_manager: ResourceManager,
    ) {
        self.track
            .borrow_mut()
            .return_borrowed_global_mut_value(resource_address, resource_manager)
    }

    fn create_proof(&mut self, proof: Proof) -> Result<ProofId, RuntimeError> {
        let proof_id = self.track.borrow_mut().new_proof_id();
        self.proofs.insert(proof_id, proof);
        Ok(proof_id)
    }

    fn take_proof(&mut self, proof_id: ProofId) -> Result<Proof, RuntimeError> {
        let proof = self
            .proofs
            .remove(&proof_id)
            .ok_or(RuntimeError::ProofNotFound(proof_id))?;

        Ok(proof)
    }

    fn create_bucket(&mut self, container: ResourceContainer) -> Result<BucketId, RuntimeError> {
        let bucket_id = self.track.borrow_mut().new_bucket_id();
        self.buckets.insert(bucket_id, Bucket::new(container));
        Ok(bucket_id)
    }

    fn create_vault(&mut self, container: ResourceContainer) -> Result<VaultId, RuntimeError> {
        let vault_id = self.track.borrow_mut().new_vault_id();
        self.owned_snodes
            .vaults
            .insert(vault_id, Vault::new(container));
        Ok(vault_id)
    }

    fn take_bucket(&mut self, bucket_id: BucketId) -> Result<Bucket, RuntimeError> {
        self.buckets
            .remove(&bucket_id)
            .ok_or(RuntimeError::BucketNotFound(bucket_id))
    }

    fn create_resource(&mut self, resource_manager: ResourceManager) -> ResourceAddress {
        self.track
            .borrow_mut()
            .create_uuid_value(resource_manager)
            .into()
    }

    fn create_package(&mut self, package: Package) -> PackageAddress {
        self.track.borrow_mut().create_uuid_value(package).into()
    }

    fn create_component(&mut self, component: Component) -> Result<ComponentAddress, RuntimeError> {
        let data = Self::process_entry_data(component.state())?;
        let new_objects = self.owned_snodes.take(data)?;
        let address = self.track.borrow_mut().create_uuid_value(component);
        self.track
            .borrow_mut()
            .insert_objects_into_component(new_objects, address.clone().into());
        Ok(address.into())
    }

    fn read_component_state(&mut self, addr: ComponentAddress) -> Result<Vec<u8>, RuntimeError> {
        if let Some(ComponentState {
            component_address,
            component,
            initial_loaded_object_refs,
            snode_refs,
        }) = &mut self.component_state
        {
            if addr.eq(component_address) {
                snode_refs.extend(initial_loaded_object_refs.clone());
                let state = component.state().to_vec();
                return Ok(state);
            }
        }

        Err(RuntimeError::ComponentNotFound(addr))
    }

    fn write_component_state(
        &mut self,
        addr: ComponentAddress,
        state: Vec<u8>,
    ) -> Result<(), RuntimeError> {
        if let Some(ComponentState {
            component_address,
            component,
            initial_loaded_object_refs,
            ..
        }) = &mut self.component_state
        {
            if addr.eq(component_address) {
                let mut new_set = Self::process_entry_data(&state)?;
                new_set.remove(&initial_loaded_object_refs)?;
                let new_objects = self.owned_snodes.take(new_set)?;
                self.track
                    .borrow_mut()
                    .insert_objects_into_component(new_objects, *component_address);
                component.set_state(state);
                return Ok(());
            }
        }
        Err(RuntimeError::ComponentNotFound(addr))
    }

    fn read_lazy_map_entry(
        &mut self,
        lazy_map_id: LazyMapId,
        key: Vec<u8>,
    ) -> Result<Option<Vec<u8>>, RuntimeError> {
        if let Some((_, value)) = self.owned_snodes.get_lazy_map_entry(&lazy_map_id, &key) {
            return Ok(value);
        }

        if let Some(ComponentState {
            component_address,
            snode_refs,
            ..
        }) = &mut self.component_state
        {
            if snode_refs.lazy_map_ids.contains(&lazy_map_id) {
                let substate_value = self
                    .track
                    .borrow_mut()
                    .read_key_value(Address::LazyMap(*component_address, lazy_map_id), key);
                let value = match substate_value {
                    SubstateValue::LazyMapEntry(v) => v,
                    _ => panic!("Substate value is not a LazyMapEntry"),
                };
                if value.is_some() {
                    let map_entry_objects =
                        Self::process_entry_data(&value.as_ref().unwrap()).unwrap();
                    snode_refs.extend(map_entry_objects);
                }

                return Ok(value);
            }
        }

        return Err(RuntimeError::LazyMapNotFound(lazy_map_id));
    }

    fn write_lazy_map_entry(
        &mut self,
        lazy_map_id: LazyMapId,
        key: Vec<u8>,
        value: Vec<u8>,
    ) -> Result<(), RuntimeError> {
        let (old_value, lazy_map_state) =
            match self.owned_snodes.get_lazy_map_entry(&lazy_map_id, &key) {
                None => match &self.component_state {
                    Some(ComponentState {
                        component_address,
                        snode_refs,
                        ..
                    }) => {
                        if !snode_refs.lazy_map_ids.contains(&lazy_map_id) {
                            return Err(RuntimeError::LazyMapNotFound(lazy_map_id));
                        }
                        let old_substate_value = self.track.borrow_mut().read_key_value(
                            Address::LazyMap(*component_address, lazy_map_id),
                            key.clone(),
                        );
                        let old_value = match old_substate_value {
                            SubstateValue::LazyMapEntry(v) => v,
                            _ => panic!("Substate value is not a LazyMapEntry"),
                        };
                        Ok((
                            old_value,
                            Committed {
                                component_address: *component_address,
                            },
                        ))
                    }
                    _ => Err(RuntimeError::LazyMapNotFound(lazy_map_id)),
                },
                Some((root, value)) => Ok((value, Uncommitted { root })),
            }?;
        let mut new_entry_object_refs = Self::process_entry_data(&value)?;
        let old_entry_object_refs = match old_value {
            None => ComponentObjectRefs::new(),
            Some(e) => Self::process_entry_data(&e).unwrap(),
        };
        new_entry_object_refs.remove(&old_entry_object_refs)?;

        // Check for cycles
        if let Uncommitted { root } = lazy_map_state {
            if new_entry_object_refs.lazy_map_ids.contains(&root) {
                return Err(RuntimeError::CyclicLazyMap(root));
            }
        }

        let new_objects = self.owned_snodes.take(new_entry_object_refs)?;

        match lazy_map_state {
            Uncommitted { root } => {
                self.owned_snodes
                    .insert_lazy_map_entry(&lazy_map_id, key, value);
                self.owned_snodes
                    .insert_objects_into_map(new_objects, &root);
            }
            Committed { component_address } => {
                self.track.borrow_mut().set_key_value(
                    Address::LazyMap(component_address, lazy_map_id),
                    key,
                    SubstateValue::LazyMapEntry(Some(value)),
                );
                self.track
                    .borrow_mut()
                    .insert_objects_into_component(new_objects, component_address);
            }
        }

        Ok(())
    }

    fn get_component_info(
        &mut self,
        component_address: ComponentAddress,
    ) -> Result<(PackageAddress, String), RuntimeError> {
        let mut track = self.track.borrow_mut();
        let substate_value = track
            .read_value(component_address)
            .ok_or(RuntimeError::ComponentNotFound(component_address))?;

        if let SubstateValue::Component(component) = substate_value {
            Ok((
                component.package_address(),
                component.blueprint_name().to_owned(),
            ))
        } else {
            panic!("Value is not a component");
        }
    }

    fn create_lazy_map(&mut self) -> LazyMapId {
        let lazy_map_id = self.track.borrow_mut().new_lazy_map_id();
        self.owned_snodes
            .lazy_maps
            .insert(lazy_map_id, UnclaimedLazyMap::new());
        lazy_map_id
    }

    fn get_epoch(&mut self) -> u64 {
        self.track.borrow().current_epoch()
    }

    fn get_transaction_hash(&mut self) -> Hash {
        self.track.borrow().transaction_hash()
    }

    fn generate_uuid(&mut self) -> u128 {
        self.track.borrow_mut().new_uuid()
    }

    fn user_log(&mut self, level: Level, message: String) {
        self.track.borrow_mut().add_log(level, message);
    }

    #[allow(unused_variables)]
    fn sys_log(&self, level: Level, msg: String) {
        let (l, m) = match level {
            Level::Error => ("ERROR".red(), msg.red()),
            Level::Warn => ("WARN".yellow(), msg.yellow()),
            Level::Info => ("INFO".green(), msg.green()),
            Level::Debug => ("DEBUG".cyan(), msg.cyan()),
            Level::Trace => ("TRACE".normal(), msg.normal()),
        };

        #[cfg(not(feature = "alloc"))]
        println!("{}[{:5}] {}", "  ".repeat(self.depth), l, m);
    }
}
