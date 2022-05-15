use colored::*;

use sbor::path::SborPath;
use sbor::rust::borrow::ToOwned;
use sbor::rust::collections::*;
use sbor::rust::format;
use sbor::rust::string::String;
use sbor::rust::string::ToString;
use sbor::rust::vec;
use sbor::rust::vec::Vec;
use sbor::*;
use scrypto::core::{SNodeRef, ScryptoActor};
use scrypto::engine::types::*;
use scrypto::resource::AuthZoneMethod;
use scrypto::values::*;

use self::LazyMapState::{Committed, Uncommitted};
use self::LoadedSNodeState::{Borrowed, Consumed, Static};
use crate::engine::*;
use crate::ledger::*;
use crate::model::*;
use crate::wasm::*;

macro_rules! re_debug {
    ($proc:expr, $($args: expr),+) => {
        if $proc.trace {
            $proc.log(Level::Debug, format!($($args),+));
        }
    };
}

macro_rules! re_info {
    ($proc:expr, $($args: expr),+) => {
        if $proc.trace {
            $proc.log(Level::Info, format!($($args),+));
        }
    };
}

macro_rules! re_warn {
    ($proc:expr, $($args: expr),+) => {
        if $proc.trace {
            $proc.log(Level::Warn, format!($($args),+));
        }
    };
}

pub enum ConsumedSNodeState {
    Bucket(Bucket),
    Proof(Proof),
}

pub enum BorrowedSNodeState {
    AuthZone(AuthZone),
    Worktop(Worktop),
    Scrypto(ScryptoActorInfo, Vec<u8>, String, Option<Component>),
    Resource(ResourceAddress, ResourceManager),
    Bucket(BucketId, Bucket),
    Proof(ProofId, Proof),
    Vault(VaultId, Option<ComponentAddress>, Vault),
}

impl BorrowedSNodeState {
    fn return_borrowed_state<'r, 'l, L: ReadableSubstateStore>(
        self,
        frame: &mut CallFrame<'r, 'l, L>,
    ) {
        match self {
            BorrowedSNodeState::AuthZone(auth_zone) => {
                frame.auth_zone = Some(auth_zone);
            }
            BorrowedSNodeState::Worktop(worktop) => {
                frame.worktop = Some(worktop);
            }
            BorrowedSNodeState::Scrypto(actor, _, _, component_state) => {
                if let Some(component_address) = actor.component_address() {
                    frame.track.return_borrowed_global_mut_value(
                        component_address,
                        component_state.unwrap(),
                    );
                }
            }
            BorrowedSNodeState::Resource(resource_address, resource_manager) => {
                frame
                    .track
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
                    frame
                        .track
                        .return_borrowed_global_mut_value((component_address, vault_id), vault);
                } else {
                    frame.owned_snodes.return_borrowed_vault_mut(vault);
                }
            }
        }
    }
}

pub enum StaticSNodeState {
    Package,
    Resource,
    System,
}

pub enum LoadedSNodeState {
    Static(StaticSNodeState),
    Consumed(Option<ConsumedSNodeState>),
    Borrowed(BorrowedSNodeState),
}

impl LoadedSNodeState {
    fn to_snode_state(&mut self) -> SNodeState {
        match self {
            Static(static_state) => match static_state {
                StaticSNodeState::Package => SNodeState::PackageStatic,
                StaticSNodeState::Resource => SNodeState::ResourceStatic,
                StaticSNodeState::System => SNodeState::SystemStatic,
            },
            Consumed(ref mut to_consume) => match to_consume.take().unwrap() {
                ConsumedSNodeState::Proof(proof) => SNodeState::Proof(proof),
                ConsumedSNodeState::Bucket(bucket) => SNodeState::Bucket(bucket),
            },
            Borrowed(ref mut borrowed) => match borrowed {
                BorrowedSNodeState::AuthZone(s) => SNodeState::AuthZoneRef(s),
                BorrowedSNodeState::Worktop(s) => SNodeState::Worktop(s),
                BorrowedSNodeState::Scrypto(info, code, export_name, s) => {
                    SNodeState::Scrypto(info.clone(), code.clone(), export_name.clone(), s.as_mut())
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

pub enum SNodeState<'a> {
    SystemStatic,
    Transaction(&'a mut TransactionProcessor),
    PackageStatic,
    AuthZoneRef(&'a mut AuthZone),
    Worktop(&'a mut Worktop),
    Scrypto(ScryptoActorInfo, Vec<u8>, String, Option<&'a mut Component>),
    ResourceStatic,
    ResourceRef(ResourceAddress, &'a mut ResourceManager),
    BucketRef(BucketId, &'a mut Bucket),
    Bucket(Bucket),
    ProofRef(ProofId, &'a mut Proof),
    Proof(Proof),
    VaultRef(VaultId, Option<ComponentAddress>, &'a mut Vault),
}

#[derive(Debug)]
struct ComponentState<'a> {
    component_address: ComponentAddress,
    component: &'a mut Component,
    initial_loaded_object_refs: ComponentObjectRefs,
    snode_refs: ComponentObjectRefs,
}

///TODO: Remove
#[derive(Debug)]
enum LazyMapState {
    Uncommitted { root: LazyMapId },
    Committed { component_address: ComponentAddress },
}

impl<'s, S: ReadableSubstateStore> Track<'s, S> {
    fn insert_objects_into_component(
        &mut self,
        new_objects: ComponentObjects,
        component_address: ComponentAddress,
    ) {
        for (vault_id, vault) in new_objects.vaults {
            self.create_uuid_value_2((component_address, vault_id), vault);
        }
        for (lazy_map_id, unclaimed) in new_objects.lazy_maps {
            self.create_key_space(component_address, lazy_map_id);
            for (k, v) in unclaimed.lazy_map {
                let parent_address = Address::LazyMap(component_address, lazy_map_id);
                self.set_key_value(parent_address, k, Some(v));
            }

            for (child_lazy_map_id, child_lazy_map) in unclaimed.descendent_lazy_maps {
                self.create_key_space(component_address, child_lazy_map_id);
                for (k, v) in child_lazy_map {
                    let parent_address = Address::LazyMap(component_address, child_lazy_map_id);
                    self.set_key_value(parent_address, k, Some(v));
                }
            }
            for (vault_id, vault) in unclaimed.descendent_vaults {
                self.create_uuid_value_2((component_address, vault_id), vault);
            }
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum MoveMethod {
    AsReturn,
    AsArgument,
}

/// A call frame is the basic unit that forms a transaction call stack. It keeps track of the
/// owned objects by this function.
///
/// A call frame can be either native or wasm (when the callee is a blueprint or component).
///
/// Radix Engine manages the lifecycle of call frames and enforces the call and move semantics.
pub struct CallFrame<'r, 'l, L: ReadableSubstateStore> {
    /// The call depth
    depth: usize,
    /// Whether to show trace messages
    trace: bool,
    /// Transactional state updates
    track: &'r mut Track<'l, L>,

    /// Owned Snodes
    buckets: HashMap<BucketId, Bucket>,
    proofs: HashMap<ProofId, Proof>,
    owned_snodes: ComponentObjects,

    /// Readable/Writable Snodes
    component: Option<ComponentState<'r>>,

    /// Referenced Snodes
    worktop: Option<Worktop>,
    auth_zone: Option<AuthZone>,

    /// The caller's auth zone
    caller_auth_zone: Option<&'r AuthZone>,
}

impl<'r, 'l, L: ReadableSubstateStore> CallFrame<'r, 'l, L> {
    /// Create a new call frame, which is not started.
    pub fn new(
        depth: usize,
        trace: bool,
        track: &'r mut Track<'l, L>,
        auth_zone: Option<AuthZone>,
        worktop: Option<Worktop>,
        buckets: HashMap<BucketId, Bucket>,
        proofs: HashMap<ProofId, Proof>,
    ) -> Self {
        Self {
            depth,
            trace,
            track,
            buckets,
            proofs,
            owned_snodes: ComponentObjects::new(),
            worktop,
            auth_zone,
            caller_auth_zone: None,
            component: None,
        }
    }

    fn new_bucket_id(&mut self) -> Result<BucketId, RuntimeError> {
        Ok(self.track.new_bucket_id())
    }

    fn new_proof_id(&mut self) -> Result<ProofId, RuntimeError> {
        Ok(self.track.new_proof_id())
    }

    /// Runs the given export within this process.
    pub fn run(
        &mut self,
        snode_ref: Option<SNodeRef>, // TODO: Remove, abstractions between invoke_snode() and run() are a bit messy right now
        snode: SNodeState<'r>,
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
        #[cfg(not(feature = "alloc"))]
        let now = std::time::Instant::now();
        re_info!(self, "Run started: snode_ref = {:?}", snode_ref);

        // Execution
        let output = match snode {
            SNodeState::SystemStatic => {
                System::static_main(call_data, self).map_err(RuntimeError::SystemError)
            }
            SNodeState::Transaction(transaction_process) => transaction_process.main(self),
            SNodeState::PackageStatic => {
                ValidatedPackage::static_main(call_data, self).map_err(RuntimeError::PackageError)
            }
            SNodeState::AuthZoneRef(auth_zone) => auth_zone
                .main(call_data, self)
                .map_err(RuntimeError::AuthZoneError),
            SNodeState::Worktop(worktop) => worktop
                .main(call_data, self)
                .map_err(RuntimeError::WorktopError),
            SNodeState::Scrypto(actor, code, export_name, component_state) => {
                let component_state = if let Some(component) = component_state {
                    let component_address = actor.component_address().unwrap().clone();
                    let data = ScryptoValue::from_slice(component.state()).unwrap();
                    let initial_loaded_object_refs = ComponentObjectRefs {
                        vault_ids: data.vault_ids.into_iter().collect(),
                        lazy_map_ids: data.lazy_map_ids.into_iter().collect(),
                    };
                    Some(ComponentState {
                        component_address,
                        component,
                        initial_loaded_object_refs,
                        snode_refs: ComponentObjectRefs::new(),
                    })
                } else {
                    None
                };
                self.component = component_state;

                let mut runtime = RadixEngineScryptoRuntime::new(actor, self);
                let mut engine = WasmiEngine::new();
                let module = engine.instantiate(&code);
                module
                    .invoke_export(&export_name, &call_data, &mut runtime)
                    .map_err(|e| match e {
                        // Flatten error code for more readable transaction receipt
                        InvokeError::RuntimeError(e) => e,
                        e @ _ => RuntimeError::InvokeError(e.into()),
                    })
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

        #[cfg(not(feature = "alloc"))]
        re_info!(
            self,
            "Run ended: time elapsed = {} ms",
            now.elapsed().as_millis()
        );
        #[cfg(feature = "alloc")]
        re_info!(self, "Run ended");

        Ok((output, moving_buckets, moving_proofs, moving_vaults))
    }

    /// Calls a function/method.
    pub fn invoke_snode(
        &mut self,
        snode_ref: SNodeRef,
        call_data: ScryptoValue,
    ) -> Result<ScryptoValue, RuntimeError> {
        let function = if let Value::Enum { name, .. } = &call_data.dom {
            name.clone()
        } else {
            return Err(RuntimeError::InvalidInvocation);
        };

        // Authorization and state load
        let (mut loaded_snode, method_auths) = match &snode_ref {
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
                        let substate_value = self
                            .track
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
                                package.code().to_vec(),
                                export_name.clone(),
                                None,
                            )),
                            vec![],
                        ))
                    }
                    ScryptoActor::Component(component_address) => {
                        let component: Component = self
                            .track
                            .borrow_global_mut_value(component_address.clone())
                            .map_err(|e| match e {
                                TrackError::NotFound => {
                                    RuntimeError::ComponentNotFound(component_address.clone())
                                }
                                TrackError::Reentrancy => {
                                    RuntimeError::ComponentReentrancy(component_address.clone())
                                }
                            })?
                            .into();
                        let package_address = component.package_address();
                        let blueprint_name = component.blueprint_name().to_string();
                        let export_name = format!("{}_main", blueprint_name);

                        let substate_value = self
                            .track
                            .read_value(package_address.clone())
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
                        Ok((
                            Borrowed(BorrowedSNodeState::Scrypto(
                                ScryptoActorInfo::component(
                                    package_address,
                                    blueprint_name,
                                    component_address.clone(),
                                ),
                                package.code().to_vec(),
                                export_name,
                                Some(component),
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
                let substate_value = self.track.read_value(resource_address.clone()).unwrap();
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
                let (component, vault) = if let Some(vault) =
                    self.owned_snodes.borrow_vault_mut(vault_id)
                {
                    (None, vault)
                } else if let Some(ComponentState {
                    component_address,
                    snode_refs,
                    ..
                }) = &self.component
                {
                    if !snode_refs.vault_ids.contains(vault_id) {
                        return Err(RuntimeError::VaultNotFound(*vault_id));
                    }
                    let vault: Vault = self
                        .track
                        .borrow_global_mut_value((*component_address, *vault_id))
                        .map_err(|e| match e {
                            TrackError::NotFound => RuntimeError::VaultNotFound(vault_id.clone()),
                            TrackError::Reentrancy => panic!("Vault logic is causing reentrancy"),
                        })?
                        .into();
                    (Some(*component_address), vault)
                } else {
                    panic!("Should never get here");
                };

                let resource_address = vault.resource_address();
                let substate_value = self.track.read_value(resource_address.clone()).unwrap();
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

        // Execution

        // Figure out what buckets and proofs to move from this process
        let mut moving_buckets = HashMap::new();
        let mut moving_proofs = HashMap::new();
        self.process_call_data(&call_data)?;
        moving_buckets.extend(self.send_buckets(&call_data.bucket_ids)?);
        moving_proofs.extend(self.send_proofs(&call_data.proof_ids, MoveMethod::AsArgument)?);

        let process_auth_zone = if matches!(
            loaded_snode,
            Borrowed(BorrowedSNodeState::Scrypto(_, _, _, _))
        ) {
            Some(AuthZone::new())
        } else {
            None
        };

        let snode = loaded_snode.to_snode_state();

        // start a new process
        let mut frame = CallFrame::new(
            self.depth + 1,
            self.trace,
            self.track,
            process_auth_zone,
            None,
            moving_buckets,
            moving_proofs,
        );
        if let Some(auth_zone) = &self.auth_zone {
            frame.caller_auth_zone = Option::Some(auth_zone);
        }

        // invoke the main function
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

    /// Checks resource leak.
    fn check_resource(&self) -> Result<(), RuntimeError> {
        re_debug!(self, "Resource check started");
        let mut success = true;

        for (bucket_id, bucket) in &self.buckets {
            re_warn!(self, "Dangling bucket: {}, {:?}", bucket_id, bucket);
            success = false;
        }
        for (vault_id, vault) in &self.owned_snodes.vaults {
            re_warn!(self, "Dangling vault: {:?}, {:?}", vault_id, vault);
            success = false;
        }
        for (lazy_map_id, lazy_map) in &self.owned_snodes.lazy_maps {
            re_warn!(self, "Dangling lazy map: {:?}, {:?}", lazy_map_id, lazy_map);
            success = false;
        }

        if let Some(worktop) = &self.worktop {
            if !worktop.is_empty() {
                re_warn!(self, "Resource worktop is not empty");
                success = false;
            }
        }

        re_debug!(self, "Resource check ended");
        if success {
            Ok(())
        } else {
            Err(RuntimeError::ResourceCheckFailure)
        }
    }

    /// Logs a message to the console.
    #[allow(unused_variables)]
    pub fn log(&self, level: Level, msg: String) {
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
            re_debug!(self, "Moving bucket: {}, {:?}", bucket_id, bucket);
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
            re_debug!(self, "Moving proof: {}, {:?}", proof_id, proof);
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
}

impl<'r, 'l, L: ReadableSubstateStore> SystemApi for CallFrame<'r, 'l, L> {
    fn invoke_snode(
        &mut self,
        snode_ref: SNodeRef,
        call_data: ScryptoValue,
    ) -> Result<ScryptoValue, RuntimeError> {
        self.invoke_snode(snode_ref, call_data)
    }

    fn get_non_fungible(
        &mut self,
        non_fungible_address: &NonFungibleAddress,
    ) -> Option<NonFungible> {
        let parent_address = Address::NonFungibleSet(non_fungible_address.resource_address());
        let key = non_fungible_address.non_fungible_id().to_vec();
        if let SubstateValue::NonFungible(non_fungible) =
            self.track.read_key_value(parent_address, key)
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
        self.track.set_key_value(parent_address, key, non_fungible)
    }

    fn borrow_global_mut_resource_manager(
        &mut self,
        resource_address: ResourceAddress,
    ) -> Result<ResourceManager, RuntimeError> {
        self.track
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
            .return_borrowed_global_mut_value(resource_address, resource_manager)
    }

    fn create_proof(&mut self, proof: Proof) -> Result<ProofId, RuntimeError> {
        let proof_id = self.new_proof_id()?;
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
        let bucket_id = self.new_bucket_id()?;
        self.buckets.insert(bucket_id, Bucket::new(container));
        Ok(bucket_id)
    }

    fn create_vault(&mut self, container: ResourceContainer) -> Result<VaultId, RuntimeError> {
        let vault_id = self.track.new_vault_id();
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
        self.track.create_uuid_value(resource_manager).into()
    }

    fn create_package(&mut self, package: ValidatedPackage) -> PackageAddress {
        self.track.create_uuid_value(package).into()
    }

    fn create_component(&mut self, component: Component) -> Result<ComponentAddress, RuntimeError> {
        let data = Self::process_entry_data(component.state())?;
        let new_objects = self.owned_snodes.take(data)?;
        let address = self.track.create_uuid_value(component);
        self.track
            .insert_objects_into_component(new_objects, address.clone().into());
        Ok(address.into())
    }

    fn read_component_state(&mut self, addr: ComponentAddress) -> Result<Vec<u8>, RuntimeError> {
        if let Some(ComponentState {
            component,
            initial_loaded_object_refs,
            component_address,
            snode_refs,
        }) = &mut self.component
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
            component,
            initial_loaded_object_refs,
            component_address,
            ..
        }) = &mut self.component
        {
            if addr.eq(component_address) {
                let mut new_set = Self::process_entry_data(&state)?;
                new_set.remove(&initial_loaded_object_refs)?;
                let new_objects = self.owned_snodes.take(new_set)?;
                self.track
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
        }) = &mut self.component
        {
            if snode_refs.lazy_map_ids.contains(&lazy_map_id) {
                let substate_value = self
                    .track
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
                None => match &self.component {
                    Some(ComponentState {
                        component_address,
                        snode_refs,
                        ..
                    }) => {
                        if !snode_refs.lazy_map_ids.contains(&lazy_map_id) {
                            return Err(RuntimeError::LazyMapNotFound(lazy_map_id));
                        }
                        let old_substate_value = self.track.read_key_value(
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
                self.track.set_key_value(
                    Address::LazyMap(component_address, lazy_map_id),
                    key,
                    SubstateValue::LazyMapEntry(Some(value)),
                );
                self.track
                    .insert_objects_into_component(new_objects, component_address);
            }
        }

        Ok(())
    }

    fn get_component_info(
        &mut self,
        component_address: ComponentAddress,
    ) -> Result<(PackageAddress, String), RuntimeError> {
        let substate_value = self
            .track
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
        let lazy_map_id = self.track.new_lazy_map_id();
        self.owned_snodes
            .lazy_maps
            .insert(lazy_map_id, UnclaimedLazyMap::new());
        lazy_map_id
    }

    fn get_epoch(&mut self) -> u64 {
        self.track.current_epoch()
    }

    fn get_transaction_hash(&mut self) -> Hash {
        self.track.transaction_hash()
    }

    fn generate_uuid(&mut self) -> u128 {
        self.track.new_uuid()
    }

    fn emit_log(&mut self, level: Level, message: String) {
        self.track.add_log(level, message);
    }
}
