use colored::*;

use sbor::*;
use sbor::path::SborPath;
use scrypto::buffer::*;
use scrypto::core::{SNodeRef, ScryptoActor};
use scrypto::engine::api::*;
use scrypto::engine::types::*;
use scrypto::rust::borrow::ToOwned;
use scrypto::rust::collections::*;
use scrypto::rust::fmt;
use scrypto::rust::format;
use scrypto::rust::string::String;
use scrypto::rust::string::ToString;
use scrypto::rust::vec;
use scrypto::rust::vec::Vec;
use scrypto::values::*;
use wasmi::*;

use crate::engine::process::LazyMapState::{Committed, Uncommitted};
use crate::engine::*;
use crate::engine::process::LoadedSNodeState::{Borrowed, Consumed, Static};
use crate::errors::*;
use crate::ledger::*;
use crate::model::*;

macro_rules! re_trace {
    ($proc:expr, $($args: expr),+) => {
        if $proc.trace {
            $proc.log(Level::Trace, format!($($args),+));
        }
    };
}

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

pub trait SystemApi {
    fn invoke_snode(
        &mut self,
        snode_ref: SNodeRef,
        function: String,
        args: Vec<ScryptoValue>,
    ) -> Result<ScryptoValue, RuntimeError>;

    fn get_non_fungible(
        &mut self,
        non_fungible_address: &NonFungibleAddress,
    ) -> Option<NonFungible>;

    fn set_non_fungible(
        &mut self,
        non_fungible_address: NonFungibleAddress,
        non_fungible: Option<NonFungible>,
    );

    fn borrow_global_mut_resource_manager(
        &mut self,
        resource_address: ResourceAddress,
    ) -> Result<ResourceManager, RuntimeError>;

    fn return_borrowed_global_resource_manager(
        &mut self,
        resource_address: ResourceAddress,
        resource_manager: ResourceManager,
    );

    fn create_bucket(&mut self, container: ResourceContainer) -> Result<BucketId, RuntimeError>;

    fn take_bucket(&mut self, bucket_id: BucketId) -> Result<Bucket, RuntimeError>;

    fn create_vault(&mut self, container: ResourceContainer) -> Result<VaultId, RuntimeError>;

    fn create_proof(&mut self, proof: Proof) -> Result<ProofId, RuntimeError>;

    fn take_proof(&mut self, proof_id: ProofId) -> Result<Proof, RuntimeError>;

    fn create_resource(&mut self, resource_manager: ResourceManager) -> ResourceAddress;

    fn create_package(&mut self, package: Package) -> PackageAddress;
}

pub enum ConsumedSNodeState {
    Bucket(Bucket),
    Proof(Proof),
}

pub enum BorrowedSNodeState {
    AuthZone(AuthZone),
    Worktop(Worktop),
    Scrypto(ScryptoActorInfo, Option<Component>),
    Resource(ResourceAddress, ResourceManager),
    Bucket(BucketId, Bucket),
    Proof(ProofId, Proof),
    Vault(VaultId, Option<ComponentAddress>, Vault),
}

impl BorrowedSNodeState {
    fn return_borrowed_state<'r, 'l, L:ReadableSubstateStore>(self, process: &mut Process<'r, 'l, L>) {
        match self {
            BorrowedSNodeState::AuthZone(auth_zone) => {
                process.auth_zone = Some(auth_zone);
            }
            BorrowedSNodeState::Worktop(worktop) => {
                process.worktop = Some(worktop);
            }
            BorrowedSNodeState::Scrypto(actor, component_state) => {
                if let Some(component_address) = actor.component_address() {
                    process.track.return_borrowed_global_component(
                        component_address,
                        component_state.unwrap(),
                    );
                }
            }
            BorrowedSNodeState::Resource(resource_address, resource_manager) => {
                process.track.return_borrowed_global_resource_manager(
                    resource_address,
                    resource_manager,
                );
            }
            BorrowedSNodeState::Bucket(bucket_id, bucket) => {
                process.buckets.insert(bucket_id, bucket);
            }
            BorrowedSNodeState::Proof(proof_id, proof) => {
                process.proofs.insert(proof_id, proof);
            }
            BorrowedSNodeState::Vault(vault_id, maybe_component_address, vault) => {
                if let Some(component_address) = maybe_component_address {
                    process.track.return_borrowed_vault(&component_address, &vault_id, vault);
                } else {
                    process.owned_snodes.return_borrowed_vault_mut(vault);
                }
            }
        }
    }
}

pub enum StaticSNodeState {
    Package,
    Resource,
}

pub enum LoadedSNodeState {
    Static(StaticSNodeState),
    Consumed(Option<ConsumedSNodeState>),
    Borrowed(BorrowedSNodeState),
}

impl LoadedSNodeState {
    fn to_snode_state(&mut self) -> SNodeState {
        match self {
            Static(static_state) => {
                match static_state {
                    StaticSNodeState::Package => SNodeState::PackageStatic,
                    StaticSNodeState::Resource => SNodeState::ResourceStatic,
                }
            }
            Consumed(ref mut to_consume) => {
                match to_consume.take().unwrap() {
                    ConsumedSNodeState::Proof(proof) => SNodeState::Proof(proof),
                    ConsumedSNodeState::Bucket(bucket) => SNodeState::Bucket(bucket),
                }
            }
            Borrowed(ref mut borrowed) => {
                match borrowed {
                    BorrowedSNodeState::AuthZone(s) => SNodeState::AuthZoneRef(s),
                    BorrowedSNodeState::Worktop(s) => SNodeState::Worktop(s),
                    BorrowedSNodeState::Scrypto(info, s) => SNodeState::Scrypto(info.clone(), s.as_mut()),
                    BorrowedSNodeState::Resource(addr, s) => SNodeState::ResourceRef(*addr, s),
                    BorrowedSNodeState::Bucket(id, s) => SNodeState::BucketRef(*id, s),
                    BorrowedSNodeState::Proof(id, s) => SNodeState::ProofRef(*id, s),
                    BorrowedSNodeState::Vault(id, addr, s) => SNodeState::VaultRef(*id, addr.clone(), s),
                }
            }
        }
    }
}

pub enum SNodeState<'a> {
    Transaction(&'a mut TransactionProcess),
    PackageStatic,
    AuthZoneRef(&'a mut AuthZone),
    Worktop(&'a mut Worktop),
    Scrypto(ScryptoActorInfo, Option<&'a mut Component>),
    ResourceStatic,
    ResourceRef(ResourceAddress, &'a mut ResourceManager),
    BucketRef(BucketId, &'a mut Bucket),
    Bucket(Bucket),
    ProofRef(ProofId, &'a mut Proof),
    Proof(Proof),
    VaultRef(VaultId, Option<ComponentAddress>, &'a mut Vault),
}

/// Represents an interpreter instance.
pub struct Interpreter {
    actor: ScryptoActorInfo,
    function: String,
    args: Vec<ScryptoValue>,
    module: ModuleRef,
    memory: MemoryRef,
}

/// Qualitative states for a WASM process
#[derive(Debug)]
enum InterpreterState<'a> {
    Blueprint,
    Component {
        component_address: ComponentAddress,
        component: &'a mut Component,
        initial_loaded_object_refs: ComponentObjectRefs,
    },
}

/// Top level state machine for a process. Empty currently only
/// refers to the initial process since it doesn't run on a wasm interpreter (yet)
#[allow(dead_code)]
struct WasmProcess<'a> {
    /// The call depth
    depth: usize,
    trace: bool,
    vm: Interpreter,
    interpreter_state: InterpreterState<'a>,
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
            self.insert_new_vault(component_address, vault_id, vault);
        }
        for (lazy_map_id, unclaimed) in new_objects.lazy_maps {
            self.insert_new_lazy_map(component_address, lazy_map_id);
            for (k, v) in unclaimed.lazy_map {
                self.put_lazy_map_entry(component_address, lazy_map_id, k, v);
            }

            for (child_lazy_map_id, child_lazy_map) in unclaimed.descendent_lazy_maps {
                self.insert_new_lazy_map(component_address, child_lazy_map_id);
                for (k, v) in child_lazy_map {
                    self.put_lazy_map_entry(component_address, child_lazy_map_id, k, v);
                }
            }
            for (vault_id, vault) in unclaimed.descendent_vaults {
                self.insert_new_vault(component_address, vault_id, vault);
            }
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum MoveMethod {
    AsReturn,
    AsArgument,
}

/// A process keeps track of resource movements and code execution.
pub struct Process<'r, 'l, L: ReadableSubstateStore> {
    /// The call depth
    depth: usize,
    /// Whether to show trace messages
    trace: bool,
    /// Transactional state updates
    track: &'r mut Track<'l, L>,

    /// Process Owned Snodes
    buckets: HashMap<BucketId, Bucket>,
    proofs: HashMap<ProofId, Proof>,
    owned_snodes: ComponentObjects,

    /// Referenced Snodes
    snode_refs: ComponentObjectRefs,
    worktop: Option<Worktop>,
    auth_zone: Option<AuthZone>,

    /// The caller's auth zone
    caller_auth_zone: Option<&'r AuthZone>,

    /// State for the given wasm process, empty only on the root process
    /// (root process cannot create components nor is a component itself)
    wasm_process_state: Option<WasmProcess<'r>>,
}

impl<'r, 'l, L: ReadableSubstateStore> Process<'r, 'l, L> {
    /// Create a new process, which is not started.
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
            snode_refs: ComponentObjectRefs::new(),
            caller_auth_zone: None,
            wasm_process_state: None,
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
        function: String,
        args: Vec<ScryptoValue>,
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
        re_info!(self, "Run started: function = {:?}", function);

        // Execution
        let output = match snode {
            SNodeState::Transaction(transaction_process) => {
                transaction_process.main(self)
            }
            SNodeState::PackageStatic => {
                Package::static_main(&function, args, self).map_err(RuntimeError::PackageError)
            }
            SNodeState::AuthZoneRef(auth_zone) => {
                auth_zone
                    .main(function.as_str(), args, self)
                    .map_err(RuntimeError::AuthZoneError)
            }
            SNodeState::Worktop(worktop) => {
                worktop
                    .main(function.as_str(), args, self)
                    .map_err(RuntimeError::WorktopError)
            }
            SNodeState::Scrypto(actor, component_state) => {
                let package = self.track.get_package(actor.package_address()).ok_or(
                    RuntimeError::PackageNotFound(actor.package_address().clone()),
                )?;

                if !package.contains_blueprint(actor.blueprint_name()) {
                    return Err(RuntimeError::BlueprintNotFound(
                        actor.package_address().clone(),
                        actor.blueprint_name().to_string(),
                    ));
                }

                let (module, memory) = package.load_module().unwrap();

                let (interpreter_state, args) = if let Some(component) = component_state {
                    let component_address = actor.component_address().unwrap().clone();
                    let data = ScryptoValue::from_slice(component.state()).unwrap();
                    let initial_loaded_object_refs = ComponentObjectRefs {
                        vault_ids: data.vault_ids.into_iter().collect(),
                        lazy_map_ids: data.lazy_map_ids.into_iter().collect(),
                    };
                    let istate = InterpreterState::Component {
                        component_address,
                        component,
                        initial_loaded_object_refs,
                    };
                    let mut args_with_self = vec![ScryptoValue::from_value(&component_address)];
                    args_with_self.extend(args);

                    (istate, args_with_self)
                } else {
                    (InterpreterState::Blueprint, args)
                };

                self.wasm_process_state = Some(WasmProcess {
                    depth: self.depth,
                    trace: self.trace,
                    vm: Interpreter {
                        function,
                        args,
                        actor: actor.clone(),
                        module: module.clone(),
                        memory,
                    },
                    interpreter_state,
                });

                // Execution
                let result = module.invoke_export(actor.export_name(), &[], self);

                // Return value
                re_debug!(self, "Invoke result: {:?}", result);
                let rtn = result
                    .map_err(|e| {
                        match e.into_host_error() {
                            // Pass-through runtime errors
                            Some(host_error) => *host_error.downcast::<RuntimeError>().unwrap(),
                            None => RuntimeError::InvokeError,
                        }
                    })?
                    .ok_or(RuntimeError::NoReturnData)?;
                match rtn {
                    RuntimeValue::I32(ptr) => self.read_return_value(ptr as u32),
                    _ => Err(RuntimeError::InvalidReturnType),
                }
            }
            SNodeState::ResourceStatic => {
                ResourceManager::static_main(function.as_str(), args, self)
                    .map_err(RuntimeError::ResourceManagerError)
            }
            SNodeState::ResourceRef(resource_address, resource_manager) => {
                let return_value = resource_manager
                    .main(resource_address, function.as_str(), args, self)
                    .map_err(RuntimeError::ResourceManagerError)?;

                Ok(return_value)
            }
            SNodeState::BucketRef(bucket_id, bucket) => bucket
                .main(bucket_id, function.as_str(), args, self)
                .map_err(RuntimeError::BucketError),
            SNodeState::Bucket(bucket) => {
                match function.as_str() {
                    "burn" => bucket.drop(self).map_err(RuntimeError::BucketError),
                    _ => Err(RuntimeError::IllegalSystemCall),
                }
            },
            SNodeState::ProofRef(_, proof) => proof
                .main(function.as_str(), args, self)
                .map_err(RuntimeError::ProofError),
            SNodeState::Proof(proof) => {
                proof.main_consume(function.as_str())
                    .map_err(RuntimeError::ProofError)
            }
            SNodeState::VaultRef(vault_id, _, vault) =>
                vault
                    .main(vault_id, function.as_str(), args, self)
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
            self.invoke_snode(SNodeRef::AuthZoneRef, "clear".to_string(), vec![])?;
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
        function: String,
        args: Vec<ScryptoValue>,
    ) -> Result<ScryptoValue, RuntimeError> {
        // Authorization and state load
        let (mut loaded_snode, method_auths) = match &snode_ref {
            SNodeRef::PackageStatic => Ok((Static(StaticSNodeState::Package), vec![])),
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
                        let export_name = format!("{}_main", blueprint_name);
                        Ok((
                            Borrowed(BorrowedSNodeState::Scrypto(
                                ScryptoActorInfo::blueprint(
                                    package_address.clone(),
                                    blueprint_name.clone(),
                                    export_name.clone(),
                                ),
                                None,
                            )),
                            vec![],
                        ))
                    }
                    ScryptoActor::Component(component_address) => {
                        let component = self
                            .track
                            .borrow_global_mut_component(component_address.clone())?;
                        let package_address = component.package_address();
                        let blueprint_name = component.blueprint_name().to_string();
                        let export_name = format!("{}_main", blueprint_name);

                        let package = self
                            .track
                            .get_package(&package_address)
                            .ok_or(RuntimeError::PackageNotFound(package_address))?;
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
                                    export_name,
                                    component_address.clone(),
                                ),
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
                    .borrow_global_mut_resource_manager(resource_address.clone())?;
                let method_auth = resource_manager.get_auth(&function, &args).clone();
                Ok((
                    Borrowed(BorrowedSNodeState::Resource(resource_address.clone(), resource_manager)),
                    vec![method_auth],
                ))
            }
            SNodeRef::Bucket(bucket_id) => {
                let bucket = self
                    .buckets
                    .remove(&bucket_id)
                    .ok_or(RuntimeError::BucketNotFound(bucket_id.clone()))?;
                let resource_address = bucket.resource_address();
                let method_auth = self
                    .track
                    .borrow_resource_manager(&resource_address)
                    .unwrap()
                    .get_auth(&function, &args);
                Ok((Consumed(Some(ConsumedSNodeState::Bucket(bucket))), vec![method_auth.clone()]))
            }
            SNodeRef::BucketRef(bucket_id) => {
                let bucket = self
                    .buckets
                    .remove(&bucket_id)
                    .ok_or(RuntimeError::BucketNotFound(bucket_id.clone()))?;
                let resource_address = bucket.resource_address();
                let method_auth = self
                    .track
                    .borrow_resource_manager(&resource_address)
                    .unwrap()
                    .get_auth(&function, &args);
                Ok((
                    Borrowed(BorrowedSNodeState::Bucket(bucket_id.clone(), bucket)),
                    vec![method_auth.clone()],
                ))
            }
            SNodeRef::ProofRef(proof_id) => {
                let proof = self.proofs.remove(&proof_id).ok_or(RuntimeError::ProofNotFound(proof_id.clone()))?;
                Ok((Borrowed(BorrowedSNodeState::Proof(proof_id.clone(), proof)), vec![]))
            }
            SNodeRef::Proof(proof_id) => {
                let proof = self.proofs.remove(&proof_id).ok_or(RuntimeError::ProofNotFound(proof_id.clone()))?;
                Ok((Consumed(Some(ConsumedSNodeState::Proof(proof))), vec![]))
            }
            SNodeRef::VaultRef(vault_id) => {
                let (component, vault) = if let Some(vault) = self.owned_snodes.borrow_vault_mut(vault_id) {
                    (None, vault)
                } else if !self.snode_refs.vault_ids.contains(vault_id) {
                    return Err(RuntimeError::VaultNotFound(*vault_id));
                } else if let Some(WasmProcess { interpreter_state: InterpreterState::Component { component_address, .. }, .. }) = &self.wasm_process_state {
                    let vault = self.track.borrow_vault_mut(component_address, vault_id);
                    (Some(*component_address), vault)
                } else {
                    panic!("Should never get here");
                };

                let resource_address = vault.resource_address();
                let method_auth = self
                    .track
                    .borrow_resource_manager(&resource_address)
                    .unwrap()
                    .get_auth(&function, &args);
                Ok((
                    Borrowed(BorrowedSNodeState::Vault(vault_id.clone(), component, vault)),
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
                Borrowed(BorrowedSNodeState::Resource(_, _)) | Borrowed(BorrowedSNodeState::Vault(_, _, _))
                | Borrowed(BorrowedSNodeState::Bucket(_, _)) | Consumed(Some(ConsumedSNodeState::Bucket(_))) => {
                    if let Some(auth_zone) = self.caller_auth_zone {
                        auth_zones.push(auth_zone);
                    }
                }
                // Extern call auth check
                _ => { }
            };

            for method_auth in method_auths {
                method_auth
                    .check(&auth_zones)
                    .map_err(|error| RuntimeError::AuthorizationError {
                        function: function.clone(),
                        authorization: method_auth,
                        error
                    })?;
            }
        }

        // Execution

        // Figure out what buckets and proofs to move from this process
        let mut moving_buckets = HashMap::new();
        let mut moving_proofs = HashMap::new();
        for arg in &args {
            self.process_call_data(arg)?;
            moving_buckets.extend(self.send_buckets(&arg.bucket_ids)?);
            moving_proofs.extend(self.send_proofs(&arg.proof_ids, MoveMethod::AsArgument)?);
        }

        let process_auth_zone = if matches!(loaded_snode, Borrowed(BorrowedSNodeState::Scrypto(_, _))) {
            Some(AuthZone::new())
        } else {
            None
        };

        let snode = loaded_snode.to_snode_state();

        // start a new process
        let mut process = Process::new(
            self.depth + 1,
            self.trace,
            self.track,
            process_auth_zone,
            None,
            moving_buckets,
            moving_proofs,
        );
        if let Some(auth_zone) = &self.auth_zone {
            process.caller_auth_zone = Option::Some(auth_zone);
        }

        // invoke the main function
        let (result, received_buckets, received_proofs, received_vaults) =
            process.run(Some(snode_ref), snode, function, args)?;

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

    /// Calls the ABI generator of a blueprint.
    // TODO: Remove
    pub fn call_abi(
        &mut self,
        package_address: PackageAddress,
        blueprint_name: &str,
    ) -> Result<ScryptoValue, RuntimeError> {
        re_debug!(self, "Call abi started");

        let snode = SNodeState::Scrypto(
            ScryptoActorInfo::blueprint(
                package_address,
                blueprint_name.to_string(),
                format!("{}_abi", blueprint_name),
            ),
            None,
        );

        let mut process = Process::new(self.depth + 1, self.trace, self.track, None, None, HashMap::new(), HashMap::new());
        let result = process
            .run(None, snode, String::new(), Vec::new())
            .map(|(r, _, _, _)| r);

        re_debug!(self, "Call abi ended");
        result
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

    fn process_return_data(&mut self, from: Option<SNodeRef>, validated: &ScryptoValue) -> Result<(), RuntimeError> {
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
            let vault = self.owned_snodes.vaults
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

    /// Send a byte array to wasm instance.
    fn send_bytes(&mut self, bytes: &[u8]) -> Result<i32, RuntimeError> {
        let wasm_process = self.wasm_process_state.as_ref().unwrap();
        let result = wasm_process.vm.module.invoke_export(
            "scrypto_alloc",
            &[RuntimeValue::I32((bytes.len()) as i32)],
            &mut NopExternals,
        );

        if let Ok(Some(RuntimeValue::I32(ptr))) = result {
            if wasm_process.vm.memory.set((ptr + 4) as u32, bytes).is_ok() {
                return Ok(ptr);
            }
        }

        Err(RuntimeError::MemoryAllocError)
    }

    fn read_return_value(&mut self, ptr: u32) -> Result<ScryptoValue, RuntimeError> {
        let wasm_process = self.wasm_process_state.as_ref().unwrap();
        // read length
        let len: u32 = wasm_process
            .vm
            .memory
            .get_value(ptr)
            .map_err(|_| RuntimeError::MemoryAccessError)?;

        let start = ptr.checked_add(4).ok_or(RuntimeError::MemoryAccessError)?;
        let end = start
            .checked_add(len)
            .ok_or(RuntimeError::MemoryAccessError)?;
        let range = start as usize..end as usize;
        let direct = wasm_process.vm.memory.direct_access();
        let buffer = direct.as_ref();

        if end > buffer.len().try_into().unwrap() {
            return Err(RuntimeError::MemoryAccessError);
        }

        ScryptoValue::from_slice(&buffer[range]).map_err(RuntimeError::ParseScryptoValueError)
    }

    /// Handles a system call.
    fn handle<I: Decode + fmt::Debug, O: Encode + fmt::Debug>(
        &mut self,
        args: RuntimeArgs,
        handler: fn(&mut Self, input: I) -> Result<O, RuntimeError>,
    ) -> Result<Option<RuntimeValue>, Trap> {
        let wasm_process = self.wasm_process_state.as_mut().unwrap();
        let op: u32 = args.nth_checked(0)?;
        let input_ptr: u32 = args.nth_checked(1)?;
        let input_len: u32 = args.nth_checked(2)?;
        // SECURITY: bill before allocating memory
        let mut input_bytes = vec![0u8; input_len as usize];
        wasm_process
            .vm
            .memory
            .get_into(input_ptr, &mut input_bytes)
            .map_err(|_| Trap::from(RuntimeError::MemoryAccessError))?;
        let input: I = scrypto_decode(&input_bytes)
            .map_err(|e| Trap::from(RuntimeError::InvalidRequestData(e)))?;
        if input_len <= 1024 {
            re_trace!(self, "{:?}", input);
        } else {
            re_trace!(self, "Large request: op = {:02x}, len = {}", op, input_len);
        }

        let output: O = handler(self, input).map_err(Trap::from)?;
        let output_bytes = scrypto_encode(&output);
        let output_ptr = self.send_bytes(&output_bytes).map_err(Trap::from)?;
        if output_bytes.len() <= 1024 {
            re_trace!(self, "{:?}", output);
        } else {
            re_trace!(
                self,
                "Large response: op = {:02x}, len = {}",
                op,
                output_bytes.len()
            );
        }

        Ok(Some(RuntimeValue::I32(output_ptr)))
    }

    //============================
    // SYSTEM CALL HANDLERS START
    //============================

    fn handle_create_component(
        &mut self,
        input: CreateComponentInput,
    ) -> Result<CreateComponentOutput, RuntimeError> {
        let data = Self::process_entry_data(&input.state)?;
        let new_objects = self.owned_snodes.take(data)?;

        let wasm_process = self
            .wasm_process_state
            .as_mut()
            .ok_or(RuntimeError::IllegalSystemCall)?;
        let package_address = wasm_process.vm.actor.package_address().clone();
        let component = Component::new(
            package_address,
            input.blueprint_name,
            input.access_rules_list,
            input.state,
        );
        let component_address = self.track.create_uuid_value(component).into();
        self.track
            .insert_objects_into_component(new_objects, component_address);

        Ok(CreateComponentOutput { component_address })
    }

    fn handle_get_component_info(
        &mut self,
        input: GetComponentInfoInput,
    ) -> Result<GetComponentInfoOutput, RuntimeError> {
        let component = self
            .track
            .borrow_component(input.component_address)
            .ok_or(RuntimeError::ComponentNotFound(input.component_address))?;

        Ok(GetComponentInfoOutput {
            package_address: component.package_address(),
            blueprint_name: component.blueprint_name().to_owned(),
        })
    }

    fn handle_get_component_state(
        &mut self,
        _: GetComponentStateInput,
    ) -> Result<GetComponentStateOutput, RuntimeError> {
        let wasm_process = self
            .wasm_process_state
            .as_mut()
            .ok_or(RuntimeError::IllegalSystemCall)?;
        let component_state = match &wasm_process.interpreter_state {
            InterpreterState::Component { component, initial_loaded_object_refs, .. } => {
                self.snode_refs.extend(initial_loaded_object_refs.clone());
                Ok(component.state())
            },
            _ => Err(RuntimeError::IllegalSystemCall),
        }?;
        let state = component_state.to_vec();
        Ok(GetComponentStateOutput { state })
    }

    fn handle_put_component_state(
        &mut self,
        input: PutComponentStateInput,
    ) -> Result<PutComponentStateOutput, RuntimeError> {
        let wasm_process = self
            .wasm_process_state
            .as_mut()
            .ok_or(RuntimeError::IllegalSystemCall)?;
        let (component, new_set, component_address) = match &mut wasm_process.interpreter_state {
            InterpreterState::Component {
                ref mut component,
                component_address,
                initial_loaded_object_refs,
                ..
            } => {
                let mut new_set = Self::process_entry_data(&input.state)?;
                new_set.remove(&initial_loaded_object_refs)?;
                Ok((component, new_set, component_address))
            }
            _ => Err(RuntimeError::IllegalSystemCall),
        }?;

        let new_objects = self.owned_snodes.take(new_set)?;
        self.track.insert_objects_into_component(new_objects, *component_address);

        // TODO: Verify that process_owned_objects is empty

        component.set_state(input.state);

        Ok(PutComponentStateOutput {})
    }

    fn handle_create_lazy_map(
        &mut self,
        _input: CreateLazyMapInput,
    ) -> Result<CreateLazyMapOutput, RuntimeError> {
        let lazy_map_id = self.track.new_lazy_map_id();
        self
            .owned_snodes
            .lazy_maps
            .insert(lazy_map_id, UnclaimedLazyMap::new());
        Ok(CreateLazyMapOutput { lazy_map_id })
    }

    fn handle_get_lazy_map_entry(
        &mut self,
        input: GetLazyMapEntryInput,
    ) -> Result<GetLazyMapEntryOutput, RuntimeError> {
        if let Some((_, value)) = self
            .owned_snodes
            .get_lazy_map_entry(&input.lazy_map_id, &input.key) {
            return Ok(GetLazyMapEntryOutput { value });
        }

        if !self.snode_refs.lazy_map_ids.contains(&input.lazy_map_id) {
            return Err(RuntimeError::LazyMapNotFound(input.lazy_map_id));
        }

        if let Some(WasmProcess { interpreter_state: InterpreterState::Component { component_address, .. }, .. }) = &self.wasm_process_state {
            let value = self.track.get_lazy_map_entry(
                *component_address,
                &input.lazy_map_id,
                &input.key,
            );
            if value.is_some() {
                let map_entry_objects =
                    Self::process_entry_data(&value.as_ref().unwrap()).unwrap();
                self.snode_refs.extend(map_entry_objects);
            }

            return Ok(GetLazyMapEntryOutput { value });
        }

        panic!("Should not get here.");
    }

    fn handle_put_lazy_map_entry(
        &mut self,
        input: PutLazyMapEntryInput,
    ) -> Result<PutLazyMapEntryOutput, RuntimeError> {
        let wasm_process = self
            .wasm_process_state
            .as_mut()
            .ok_or(RuntimeError::IllegalSystemCall)?;
        let (old_value, lazy_map_state) = match self
            .owned_snodes
            .get_lazy_map_entry(&input.lazy_map_id, &input.key)
        {
            None => match &wasm_process.interpreter_state {
                InterpreterState::Component {
                    component_address,
                    ..
                } => {
                    if !self.snode_refs
                            .lazy_map_ids
                            .contains(&input.lazy_map_id)
                    {
                        return Err(RuntimeError::LazyMapNotFound(input.lazy_map_id));
                    }
                    let old_value = self.track.get_lazy_map_entry(
                        *component_address,
                        &input.lazy_map_id,
                        &input.key,
                    );
                    Ok((
                        old_value,
                        Committed {
                            component_address: *component_address,
                        },
                    ))
                }
                _ => Err(RuntimeError::LazyMapNotFound(input.lazy_map_id)),
            },
            Some((root, value)) => Ok((value, Uncommitted { root })),
        }?;
        let mut new_entry_object_refs = Self::process_entry_data(&input.value)?;
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

        let new_objects = self
            .owned_snodes
            .take(new_entry_object_refs)?;

        match lazy_map_state {
            Uncommitted { root } => {
                self.owned_snodes.insert_lazy_map_entry(
                    &input.lazy_map_id,
                    input.key,
                    input.value,
                );
                self
                    .owned_snodes
                    .insert_objects_into_map(new_objects, &root);
            }
            Committed { component_address } => {
                self.track.put_lazy_map_entry(
                    component_address,
                    input.lazy_map_id,
                    input.key,
                    input.value,
                );
                self.track
                    .insert_objects_into_component(new_objects, component_address);
            }
        }

        Ok(PutLazyMapEntryOutput {})
    }

    fn handle_invoke_snode(
        &mut self,
        input: InvokeSNodeInput,
    ) -> Result<InvokeSNodeOutput, RuntimeError> {
        let mut validated_args = Vec::new();
        for arg in input.args {
            validated_args.push(
                ScryptoValue::from_slice(&arg).map_err(RuntimeError::ParseScryptoValueError)?,
            );
        }

        let result = self.invoke_snode(input.snode_ref, input.function, validated_args)?;
        Ok(InvokeSNodeOutput { rtn: result.raw })
    }

    fn handle_emit_log(&mut self, input: EmitLogInput) -> Result<EmitLogOutput, RuntimeError> {
        self.track.add_log(input.level, input.message);

        Ok(EmitLogOutput {})
    }

    fn handle_get_call_data(
        &mut self,
        _input: GetCallDataInput,
    ) -> Result<GetCallDataOutput, RuntimeError> {
        let wasm_process = self
            .wasm_process_state
            .as_ref()
            .ok_or(RuntimeError::InterpreterNotStarted)?;
        Ok(GetCallDataOutput {
            function: wasm_process.vm.function.clone(),
            args: wasm_process
                .vm
                .args
                .iter()
                .cloned()
                .map(|v| v.raw)
                .collect(),
        })
    }

    fn handle_get_transaction_hash(
        &mut self,
        _input: GetTransactionHashInput,
    ) -> Result<GetTransactionHashOutput, RuntimeError> {
        Ok(GetTransactionHashOutput {
            transaction_hash: self.track.transaction_hash(),
        })
    }

    fn handle_get_current_epoch(
        &mut self,
        _input: GetCurrentEpochInput,
    ) -> Result<GetCurrentEpochOutput, RuntimeError> {
        Ok(GetCurrentEpochOutput {
            current_epoch: self.track.current_epoch(),
        })
    }

    fn handle_generate_uuid(
        &mut self,
        _input: GenerateUuidInput,
    ) -> Result<GenerateUuidOutput, RuntimeError> {
        Ok(GenerateUuidOutput {
            uuid: self.track.new_uuid(),
        })
    }

    fn handle_get_actor(&mut self, _input: GetActorInput) -> Result<GetActorOutput, RuntimeError> {
        let wasm_process = self
            .wasm_process_state
            .as_ref()
            .ok_or(RuntimeError::InterpreterNotStarted)?;

        return Ok(GetActorOutput {
            actor: wasm_process.vm.actor.clone(),
        });
    }

    //============================
    // SYSTEM CALL HANDLERS END
    //============================
}

impl<'r, 'l, L: ReadableSubstateStore> SystemApi for Process<'r, 'l, L> {
    fn invoke_snode(
        &mut self,
        snode_ref: SNodeRef,
        function: String,
        args: Vec<ScryptoValue>,
    ) -> Result<ScryptoValue, RuntimeError> {
        self.invoke_snode(snode_ref, function, args)
    }

    fn get_non_fungible(
        &mut self,
        non_fungible_address: &NonFungibleAddress,
    ) -> Option<NonFungible> {
        self.track.get_non_fungible(non_fungible_address)
    }

    fn set_non_fungible(
        &mut self,
        non_fungible_address: NonFungibleAddress,
        non_fungible: Option<NonFungible>,
    ) {
        self.track
            .set_non_fungible(non_fungible_address, non_fungible)
    }

    fn borrow_global_mut_resource_manager(
        &mut self,
        resource_address: ResourceAddress,
    ) -> Result<ResourceManager, RuntimeError> {
        self.track
            .borrow_global_mut_resource_manager(resource_address)
    }

    fn return_borrowed_global_resource_manager(
        &mut self,
        resource_address: ResourceAddress,
        resource_manager: ResourceManager,
    ) {
        self.track
            .return_borrowed_global_resource_manager(resource_address, resource_manager)
    }

    fn create_proof(&mut self, proof: Proof) -> Result<ProofId, RuntimeError> {
        let proof_id = self.new_proof_id()?;
        self.proofs.insert(proof_id, proof);
        Ok(proof_id)
    }

    fn take_proof(&mut self, proof_id: ProofId) -> Result<Proof, RuntimeError> {
        let proof = self.proofs
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
        self
            .owned_snodes
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

    fn create_package(&mut self, package: Package) -> PackageAddress {
        self.track.create_uuid_value(package).into()
    }
}

impl<'r, 'l, L: ReadableSubstateStore> Externals for Process<'r, 'l, L> {
    fn invoke_index(
        &mut self,
        index: usize,
        args: RuntimeArgs,
    ) -> Result<Option<RuntimeValue>, Trap> {
        match index {
            ENGINE_FUNCTION_INDEX => {
                let operation: u32 = args.nth_checked(0)?;
                match operation {
                    CREATE_COMPONENT => self.handle(args, Self::handle_create_component),
                    GET_COMPONENT_INFO => self.handle(args, Self::handle_get_component_info),
                    GET_COMPONENT_STATE => self.handle(args, Self::handle_get_component_state),
                    PUT_COMPONENT_STATE => self.handle(args, Self::handle_put_component_state),

                    CREATE_LAZY_MAP => self.handle(args, Self::handle_create_lazy_map),
                    GET_LAZY_MAP_ENTRY => self.handle(args, Self::handle_get_lazy_map_entry),
                    PUT_LAZY_MAP_ENTRY => self.handle(args, Self::handle_put_lazy_map_entry),

                    INVOKE_SNODE => self.handle(args, Self::handle_invoke_snode),

                    EMIT_LOG => self.handle(args, Self::handle_emit_log),
                    GET_CALL_DATA => self.handle(args, Self::handle_get_call_data),
                    GET_TRANSACTION_HASH => self.handle(args, Self::handle_get_transaction_hash),
                    GET_CURRENT_EPOCH => self.handle(args, Self::handle_get_current_epoch),
                    GENERATE_UUID => self.handle(args, Self::handle_generate_uuid),
                    GET_ACTOR => self.handle(args, Self::handle_get_actor),

                    _ => Err(RuntimeError::InvalidRequestCode(operation).into()),
                }
            }
            _ => Err(RuntimeError::HostFunctionNotFound(index).into()),
        }
    }
}
