use colored::*;

use sbor::*;
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
    fn get_non_fungible(
        &mut self,
        non_fungible_address: &NonFungibleAddress,
    ) -> Option<&NonFungible>;

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

    fn create_resource(&mut self, resource_manager: ResourceManager) -> ResourceAddress;
}

pub enum SNodeState {
    Scrypto(ScryptoActorInfo, Option<Component>),
    ResourceStatic,
    ResourceRef(ResourceAddress, ResourceManager),
    BucketRef(BucketId, Bucket),
    Bucket(Bucket),
    VaultRef(VaultId),
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
        additional_object_refs: ComponentObjectRefs,
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
    process_owned_objects: ComponentObjects,
}

impl<'a> WasmProcess<'a> {
    fn check_resource(&self) -> bool {
        let mut success = true;

        for (vault_id, vault) in &self.process_owned_objects.vaults {
            re_warn!(self, "Dangling vault: {:?}, {:?}", vault_id, vault);
            success = false;
        }
        for (lazy_map_id, lazy_map) in &self.process_owned_objects.lazy_maps {
            re_warn!(self, "Dangling lazy map: {:?}, {:?}", lazy_map_id, lazy_map);
            success = false;
        }

        return success;
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
}

///TODO: Remove
#[derive(Debug)]
enum LazyMapState {
    Uncommitted { root: LazyMapId },
    Committed { component_address: ComponentAddress },
}

impl<'s, S: SubstateStore> Track<'s, S> {
    fn insert_objects_into_component(
        &mut self,
        new_objects: ComponentObjects,
        component_address: ComponentAddress,
    ) {
        for (vault_id, vault) in new_objects.vaults {
            self.put_vault(component_address, vault_id, vault);
        }
        for (lazy_map_id, unclaimed) in new_objects.lazy_maps {
            for (k, v) in unclaimed.lazy_map {
                self.put_lazy_map_entry(component_address, lazy_map_id, k, v);
            }
            for (child_lazy_map_id, child_lazy_map) in unclaimed.descendent_lazy_maps {
                for (k, v) in child_lazy_map {
                    self.put_lazy_map_entry(component_address, child_lazy_map_id, k, v);
                }
            }
            for (vault_id, vault) in unclaimed.descendent_vaults {
                self.put_vault(component_address, vault_id, vault);
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
pub struct Process<'r, 'l, L: SubstateStore> {
    /// The call depth
    depth: usize,
    /// Whether to show trace messages
    trace: bool,
    /// Transactional state updates
    track: &'r mut Track<'l, L>,

    /// Buckets owned by this process
    buckets: HashMap<BucketId, Bucket>,
    /// Bucket proofs
    proofs: HashMap<ProofId, Proof>,

    /// State for the given wasm process, empty only on the root process
    /// (root process cannot create components nor is a component itself)
    wasm_process_state: Option<WasmProcess<'r>>,

    /// ID allocator for buckets and proofs created within transaction.
    id_allocator: IdAllocator,
    /// Resources collected from previous returns or self.
    worktop: Worktop,
    /// Proofs collected from previous returns or self. Also used for system authorization.
    auth_zone: Vec<Proof>,
    /// The caller's auth zone
    caller_auth_zone: &'r [Proof],
}

impl<'r, 'l, L: SubstateStore> Process<'r, 'l, L> {
    /// Create a new process, which is not started.
    pub fn new(depth: usize, trace: bool, track: &'r mut Track<'l, L>) -> Self {
        Self {
            depth,
            trace,
            track,
            buckets: HashMap::new(),
            proofs: HashMap::new(),
            wasm_process_state: None,
            id_allocator: IdAllocator::new(IdSpace::Transaction),
            worktop: Worktop::new(),
            auth_zone: Vec::new(),
            caller_auth_zone: &[],
        }
    }

    fn new_bucket_id(&mut self) -> Result<BucketId, RuntimeError> {
        if self.depth == 0 {
            self.id_allocator
                .new_bucket_id()
                .map_err(RuntimeError::IdAllocatorError)
        } else {
            Ok(self.track.new_bucket_id())
        }
    }

    fn new_proof_id(&mut self) -> Result<ProofId, RuntimeError> {
        if self.depth == 0 {
            self.id_allocator
                .new_proof_id()
                .map_err(RuntimeError::IdAllocatorError)
        } else {
            Ok(self.track.new_proof_id())
        }
    }

    // (Transaction ONLY) Takes resource by amount from worktop and returns a bucket.
    pub fn take_from_worktop(
        &mut self,
        amount: Decimal,
        resource_address: ResourceAddress,
    ) -> Result<BucketId, RuntimeError> {
        re_debug!(
            self,
            "(Transaction) Taking from worktop: {}, {}",
            amount,
            resource_address
        );
        let new_bucket_id = self.new_bucket_id()?;
        let new_bucket = match self
            .worktop
            .take(amount, resource_address)
            .map_err(RuntimeError::WorktopError)?
        {
            Some(bucket) => bucket,
            None => self.new_empty_bucket(resource_address)?,
        };
        self.buckets.insert(new_bucket_id, new_bucket);
        Ok(new_bucket_id)
    }

    // (Transaction ONLY) Takes resource by non-fungible IDs from worktop and returns a bucket.
    pub fn take_non_fungibles_from_worktop(
        &mut self,
        ids: &BTreeSet<NonFungibleId>,
        resource_address: ResourceAddress,
    ) -> Result<BucketId, RuntimeError> {
        re_debug!(
            self,
            "(Transaction) Taking from worktop: {:?}, {}",
            ids,
            resource_address
        );
        let new_bucket_id = self.new_bucket_id()?;
        let new_bucket = match self
            .worktop
            .take_non_fungibles(ids, resource_address)
            .map_err(RuntimeError::WorktopError)?
        {
            Some(bucket) => bucket,
            None => self.new_empty_bucket(resource_address)?,
        };
        self.buckets.insert(new_bucket_id, new_bucket);
        Ok(new_bucket_id)
    }

    // (Transaction ONLY) Takes resource by resource address from worktop and returns a bucket.
    pub fn take_all_from_worktop(
        &mut self,
        resource_address: ResourceAddress,
    ) -> Result<BucketId, RuntimeError> {
        re_debug!(
            self,
            "(Transaction) Taking from worktop: ALL, {}",
            resource_address
        );
        let new_bucket_id = self.new_bucket_id()?;
        let new_bucket = match self
            .worktop
            .take_all(resource_address)
            .map_err(RuntimeError::WorktopError)?
        {
            Some(bucket) => bucket,
            None => self.new_empty_bucket(resource_address)?,
        };
        self.buckets.insert(new_bucket_id, new_bucket);
        Ok(new_bucket_id)
    }

    fn new_empty_bucket(
        &mut self,
        resource_address: ResourceAddress,
    ) -> Result<Bucket, RuntimeError> {
        let resource_manager = self
            .track
            .get_resource_manager(&resource_address)
            .ok_or(RuntimeError::ResourceManagerNotFound(resource_address))?;
        Ok(Bucket::new(ResourceContainer::new_empty(
            resource_address,
            resource_manager.resource_type(),
        )))
    }

    // (Transaction ONLY) Returns resource back to worktop.
    pub fn return_to_worktop(&mut self, bucket_id: BucketId) -> Result<ScryptoValue, RuntimeError> {
        re_debug!(
            self,
            "(Transaction) Returning to worktop: bucket_id = {}",
            bucket_id
        );

        let bucket = self
            .buckets
            .remove(&bucket_id)
            .ok_or(RuntimeError::BucketNotFound(bucket_id))?;
        self.worktop
            .put(bucket)
            .map_err(RuntimeError::WorktopError)?;
        Ok(ScryptoValue::from_value(&()))
    }

    // (Transaction ONLY) Assert worktop contains at least this amount.
    pub fn assert_worktop_contains(
        &mut self,
        resource_address: ResourceAddress,
    ) -> Result<ScryptoValue, RuntimeError> {
        if self.worktop.total_amount(resource_address).is_zero() {
            Err(RuntimeError::AssertionFailed)
        } else {
            Ok(ScryptoValue::from_value(&()))
        }
    }

    // (Transaction ONLY) Assert worktop contains at least this amount.
    pub fn assert_worktop_contains_by_amount(
        &mut self,
        amount: Decimal,
        resource_address: ResourceAddress,
    ) -> Result<ScryptoValue, RuntimeError> {
        if self.worktop.total_amount(resource_address) < amount {
            Err(RuntimeError::AssertionFailed)
        } else {
            Ok(ScryptoValue::from_value(&()))
        }
    }

    // (Transaction ONLY) Assert worktop contains at least this amount.
    pub fn assert_worktop_contains_by_ids(
        &mut self,
        ids: &BTreeSet<NonFungibleId>,
        resource_address: ResourceAddress,
    ) -> Result<ScryptoValue, RuntimeError> {
        if !self
            .worktop
            .total_ids(resource_address)
            .map_err(RuntimeError::WorktopError)?
            .is_superset(ids)
        {
            Err(RuntimeError::AssertionFailed)
        } else {
            Ok(ScryptoValue::from_value(&()))
        }
    }

    // Takes a proof from the auth zone.
    pub fn pop_from_auth_zone(&mut self) -> Result<ProofId, RuntimeError> {
        re_debug!(self, "Popping from auth zone");
        if self.auth_zone.is_empty() {
            return Err(RuntimeError::EmptyAuthZone);
        }

        let new_proof_id = self.new_proof_id()?;
        let proof = self.auth_zone.remove(self.auth_zone.len() - 1);
        self.proofs.insert(new_proof_id, proof);
        Ok(new_proof_id)
    }

    // Puts a proof onto the auth zone.
    pub fn push_to_auth_zone(&mut self, proof_id: ProofId) -> Result<(), RuntimeError> {
        re_debug!(self, "Pushing onto auth zone: proof_id = {}", proof_id);

        let proof = self
            .proofs
            .remove(&proof_id)
            .ok_or(RuntimeError::ProofNotFound(proof_id))?;

        if proof.is_restricted() {
            return Err(RuntimeError::CantMoveRestrictedProof(proof_id));
        }

        self.auth_zone.push(proof);
        Ok(())
    }

    // Creates a bucket proof.
    pub fn create_bucket_proof(&mut self, bucket_id: BucketId) -> Result<ProofId, RuntimeError> {
        re_debug!(self, "Creating proof: bucket_id = {}", bucket_id);

        let new_proof_id = self.new_proof_id()?;
        let bucket = self
            .buckets
            .get_mut(&bucket_id)
            .ok_or(RuntimeError::BucketNotFound(bucket_id))?;
        let new_proof = bucket
            .create_proof(ResourceContainerId::Bucket(bucket_id))
            .map_err(RuntimeError::ProofError)?;
        self.proofs.insert(new_proof_id, new_proof);

        Ok(new_proof_id)
    }

    pub fn create_bucket_proof_by_amount(
        &mut self,
        bucket_id: BucketId,
        amount: Decimal,
    ) -> Result<ProofId, RuntimeError> {
        re_debug!(self, "Creating proof: bucket_id = {}", bucket_id);

        let new_proof_id = self.new_proof_id()?;
        let bucket = self
            .buckets
            .get_mut(&bucket_id)
            .ok_or(RuntimeError::BucketNotFound(bucket_id))?;
        let new_proof = bucket
            .create_proof_by_amount(amount, ResourceContainerId::Bucket(bucket_id))
            .map_err(RuntimeError::ProofError)?;
        self.proofs.insert(new_proof_id, new_proof);

        Ok(new_proof_id)
    }

    pub fn create_bucket_proof_by_ids(
        &mut self,
        bucket_id: BucketId,
        ids: &BTreeSet<NonFungibleId>,
    ) -> Result<ProofId, RuntimeError> {
        re_debug!(self, "Creating bucket proof: bucket_id = {}", bucket_id);

        let new_proof_id = self.new_proof_id()?;
        let bucket = self
            .buckets
            .get_mut(&bucket_id)
            .ok_or(RuntimeError::BucketNotFound(bucket_id))?;
        let new_proof = bucket
            .create_proof_by_ids(ids, ResourceContainerId::Bucket(bucket_id))
            .map_err(RuntimeError::ProofError)?;
        self.proofs.insert(new_proof_id, new_proof);

        Ok(new_proof_id)
    }

    // Creates a vault proof.
    pub fn create_vault_proof(&mut self, vault_id: VaultId) -> Result<ProofId, RuntimeError> {
        re_debug!(self, "Creating vault proof: vault_id = {:?}", vault_id);

        let new_proof_id = self.new_proof_id()?;
        let vault = self.get_local_vault(&vault_id)?;
        let new_proof = vault
            .create_proof(ResourceContainerId::Vault(vault_id))
            .map_err(RuntimeError::ProofError)?;
        self.proofs.insert(new_proof_id, new_proof);

        Ok(new_proof_id)
    }

    pub fn create_vault_proof_by_amount(
        &mut self,
        vault_id: VaultId,
        amount: Decimal,
    ) -> Result<ProofId, RuntimeError> {
        re_debug!(self, "Creating vault proof: vault_id = {:?}", vault_id);

        let new_proof_id = self.new_proof_id()?;
        let vault = self.get_local_vault(&vault_id)?;
        let new_proof = vault
            .create_proof_by_amount(amount, ResourceContainerId::Vault(vault_id))
            .map_err(RuntimeError::ProofError)?;
        self.proofs.insert(new_proof_id, new_proof);

        Ok(new_proof_id)
    }

    pub fn create_vault_proof_by_ids(
        &mut self,
        vault_id: VaultId,
        ids: &BTreeSet<NonFungibleId>,
    ) -> Result<ProofId, RuntimeError> {
        re_debug!(self, "Creating vault proof: vault_id = {:?}", vault_id);

        let new_proof_id = self.new_proof_id()?;
        let vault = self.get_local_vault(&vault_id)?;
        let new_proof = vault
            .create_proof_by_ids(ids, ResourceContainerId::Vault(vault_id))
            .map_err(RuntimeError::ProofError)?;
        self.proofs.insert(new_proof_id, new_proof);

        Ok(new_proof_id)
    }

    // Creates a auth zone proof for all of the specified resource.
    pub fn create_auth_zone_proof(
        &mut self,
        resource_address: ResourceAddress,
    ) -> Result<ProofId, RuntimeError> {
        re_debug!(self, "Creating auth zone proof: ALL, {}", resource_address);

        let resource_manager = self
            .track
            .get_resource_manager(&resource_address)
            .ok_or(RuntimeError::ResourceManagerNotFound(resource_address))?;
        let resource_type = resource_manager.resource_type();

        let new_proof_id = self.new_proof_id()?;
        let new_proof = Proof::compose(&self.auth_zone, resource_address, resource_type)
            .map_err(RuntimeError::ProofError)?;
        self.proofs.insert(new_proof_id, new_proof);

        Ok(new_proof_id)
    }

    pub fn create_auth_zone_proof_by_amount(
        &mut self,
        amount: Decimal,
        resource_address: ResourceAddress,
    ) -> Result<ProofId, RuntimeError> {
        re_debug!(self, "Creating proof: {}, {}", amount, resource_address);

        let resource_manager = self
            .track
            .get_resource_manager(&resource_address)
            .ok_or(RuntimeError::ResourceManagerNotFound(resource_address))?;
        let resource_type = resource_manager.resource_type();

        let new_proof_id = self.new_proof_id()?;
        let new_proof =
            Proof::compose_by_amount(&self.auth_zone, amount, resource_address, resource_type)
                .map_err(RuntimeError::ProofError)?;
        self.proofs.insert(new_proof_id, new_proof);

        Ok(new_proof_id)
    }

    pub fn create_auth_zone_proof_by_ids(
        &mut self,
        ids: &BTreeSet<NonFungibleId>,
        resource_address: ResourceAddress,
    ) -> Result<ProofId, RuntimeError> {
        re_debug!(self, "Creating proof: {:?}, {}", ids, resource_address);

        let resource_manager = self
            .track
            .get_resource_manager(&resource_address)
            .ok_or(RuntimeError::ResourceManagerNotFound(resource_address))?;
        let resource_type = resource_manager.resource_type();

        let new_proof_id = self.new_proof_id()?;
        let new_proof =
            Proof::compose_by_ids(&self.auth_zone, ids, resource_address, resource_type)
                .map_err(RuntimeError::ProofError)?;
        self.proofs.insert(new_proof_id, new_proof);

        Ok(new_proof_id)
    }

    // Clone a proof.
    pub fn clone_proof(&mut self, proof_id: ProofId) -> Result<ProofId, RuntimeError> {
        re_debug!(self, "Cloning proof: proof_id = {}", proof_id);

        let new_proof_id = self.new_proof_id()?;
        let proof = self
            .proofs
            .get(&proof_id)
            .ok_or(RuntimeError::ProofNotFound(proof_id))?;
        let new_proof = proof.clone();
        self.proofs.insert(new_proof_id, new_proof);

        Ok(new_proof_id)
    }

    // Drop a proof.
    pub fn drop_proof(&mut self, proof_id: ProofId) -> Result<(), RuntimeError> {
        re_debug!(self, "Dropping proof: proof_id = {}", proof_id);

        let proof = self
            .proofs
            .remove(&proof_id)
            .ok_or(RuntimeError::ProofNotFound(proof_id))?;

        proof.drop();

        Ok(())
    }

    pub fn drop_all_named_proofs(&mut self) -> Result<(), RuntimeError> {
        let proof_ids: Vec<ProofId> = self.proofs.keys().cloned().collect();
        for proof_id in proof_ids {
            self.drop_proof(proof_id)?;
        }
        Ok(())
    }

    pub fn drop_all_auth_zone_proofs(&mut self) -> Result<(), RuntimeError> {
        loop {
            if let Some(proof) = self.auth_zone.pop() {
                proof.drop();
            } else {
                break;
            }
        }
        Ok(())
    }

    /// Drops all proofs owned by this process.
    pub fn drop_all_proofs(&mut self) -> Result<(), RuntimeError> {
        self.drop_all_named_proofs()?;
        self.drop_all_auth_zone_proofs()
    }

    /// (Transaction ONLY) Calls a method.
    pub fn call_method_with_all_resources(
        &mut self,
        component_address: ComponentAddress,
        method: &str,
    ) -> Result<ScryptoValue, RuntimeError> {
        re_debug!(
            self,
            "(Transaction) Calling method with all resources started"
        );

        // 1. Drop all proofs to unlock the buckets
        self.drop_all_proofs()?;

        // 2. Move collected resource to temp buckets
        for id in self.worktop.resource_addresses() {
            if let Some(bucket) = self
                .worktop
                .take_all(id)
                .map_err(RuntimeError::WorktopError)?
            {
                /*
                This is the only place that we don't follow the convention for bucket/proof ID generation.

                The reason is that the number of buckets to be created can't be determined statically, which
                makes it hard to verify transaction if we use the transaction ID allocator.
                */
                let bucket_id = self.track.new_bucket_id();
                self.buckets.insert(bucket_id, bucket);
            }
        }

        // 3. Call the method with all buckets
        let to_deposit: Vec<scrypto::resource::Bucket> = self
            .buckets
            .keys()
            .cloned()
            .map(|bucket_id| scrypto::resource::Bucket(bucket_id))
            .collect();

        let result = self.call(
            SNodeRef::Scrypto(ScryptoActor::Component(component_address)),
            method.to_owned(),
            vec![ScryptoValue::from_value(&to_deposit)],
        );

        re_debug!(
            self,
            "(Transaction) Calling method with all resources ended"
        );
        result
    }

    pub fn publish_package(&mut self, code: Vec<u8>) -> Result<PackageAddress, RuntimeError> {
        re_debug!(self, "Publishing a package");

        let package = Package::new(code).map_err(RuntimeError::WasmValidationError)?;
        let package_address = self.track.create_package(package);
        Ok(package_address)
    }

    /// (SYSTEM ONLY)  Creates a proof which references a virtual bucket
    pub fn create_virtual_proof(
        &mut self,
        bucket_id: BucketId,
        proof_id: ProofId,
        mut bucket: Bucket,
    ) -> Result<(), RuntimeError> {
        let proof = bucket
            .create_proof(ResourceContainerId::Bucket(bucket_id))
            .map_err(RuntimeError::ProofError)?;
        self.proofs.insert(proof_id, proof);
        Ok(())
    }

    /// Runs the given export within this process.
    pub fn run(
        &mut self,
        snode: &'r mut SNodeState,
        function: String,
        args: Vec<ScryptoValue>,
    ) -> Result<(ScryptoValue, HashMap<BucketId, Bucket>, HashMap<ProofId, Proof>), RuntimeError> {
        #[cfg(not(feature = "alloc"))]
        let now = std::time::Instant::now();
        re_info!(self, "Run started: function = {:?}", function);

        // Execution
        let output = match snode {
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
                        additional_object_refs: ComponentObjectRefs::new(),
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
                    process_owned_objects: ComponentObjects::new(),
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
                    .main(*resource_address, function.as_str(), args, self)
                    .map_err(RuntimeError::ResourceManagerError)?;

                Ok(return_value)
            }
            SNodeState::BucketRef(_, bucket) => {
                bucket.main(function.as_str(), args, self)
                    .map_err(RuntimeError::BucketError)
            },

            _ => Err(RuntimeError::IllegalSystemCall),
        }?;

        self.process_return_data(&output)?;

        // figure out what buckets and resources to return
        let moving_buckets = self.send_buckets(&output.bucket_ids)?;
        let moving_proofs = self.send_proofs(&output.proof_ids, MoveMethod::AsReturn)?;

        // drop proofs and check resource leak
        self.drop_all_proofs()?;
        self.check_resource()?;

        #[cfg(not(feature = "alloc"))]
        re_info!(
            self,
            "Run ended: time elapsed = {} ms",
            now.elapsed().as_millis()
        );
        #[cfg(feature = "alloc")]
        re_info!(self, "Run ended");

        Ok((output, moving_buckets, moving_proofs))
    }

    /// Calls a function/method.
    pub fn call(
        &mut self,
        mut snode_ref: SNodeRef,
        function: String,
        args: Vec<ScryptoValue>,
    ) -> Result<ScryptoValue, RuntimeError> {

        // Authorization and state load
        let (mut snode, method_auth) = match &mut snode_ref {
            SNodeRef::Scrypto(actor) => {
                match actor {
                    ScryptoActor::Blueprint(package_address, blueprint_name) => {
                        let export_name = format!("{}_main", blueprint_name);
                        Ok((
                            SNodeState::Scrypto(
                                ScryptoActorInfo::blueprint(
                                    package_address.clone(),
                                    blueprint_name.clone(),
                                    export_name.clone(),
                                ),
                                None,
                            ),
                            MethodAuthorization::Public,
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

                        let (_, method_auth) = component.method_authorization(&schema, &function);
                        Ok((
                            SNodeState::Scrypto(
                                ScryptoActorInfo::component(
                                    package_address,
                                    blueprint_name,
                                    export_name,
                                    component_address.clone(),
                                ),
                                Some(component),
                            ),
                            method_auth,
                        ))
                    }
                }
            }
            SNodeRef::ResourceStatic => {
                Ok((SNodeState::ResourceStatic, MethodAuthorization::Public))
            }
            SNodeRef::ResourceRef(resource_address) => {
                let resource_manager: ResourceManager = self
                    .track
                    .borrow_global_mut_resource_manager(resource_address.clone())?;
                let method_auth = resource_manager.get_auth(&function).clone();
                Ok((
                    SNodeState::ResourceRef(resource_address.clone(), resource_manager),
                    method_auth,
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
                    .get_resource_manager(&resource_address)
                    .unwrap()
                    .get_auth(&function);
                Ok((SNodeState::Bucket(bucket), method_auth.clone()))
            }
            SNodeRef::BucketRef(bucket_id) => {
                let bucket = self
                    .buckets
                    .remove(&bucket_id)
                    .ok_or(RuntimeError::BucketNotFound(bucket_id.clone()))?;
                let resource_address = bucket.resource_address();
                let method_auth = self
                    .track
                    .get_resource_manager(&resource_address)
                    .unwrap()
                    .get_auth(&function);
                Ok((SNodeState::BucketRef(bucket_id.clone(), bucket), method_auth.clone()))
            }
            SNodeRef::VaultRef(vault_id) => {
                let resource_address = self.get_local_vault(&vault_id)?.resource_address();
                let method_auth = self
                    .track
                    .get_resource_manager(&resource_address)
                    .unwrap()
                    .get_auth(&function);
                Ok((SNodeState::VaultRef(vault_id.clone()), method_auth.clone()))
            }
        }?;

        // Authorization check
        let proofs_vector = match &snode {
            // Same process auth check
            SNodeState::VaultRef(_) | SNodeState::BucketRef(_, _) => {
                vec![self.caller_auth_zone, &self.auth_zone]
            }
            // Extern call auth check
            _ => vec![self.auth_zone.as_slice()],
        };

        method_auth
            .check(&proofs_vector)
            .map_err(|e| RuntimeError::AuthorizationError(function.clone(), e))?;


        // Execution

        // Figure out what buckets and proofs to move from this process
        let mut moving_buckets = HashMap::new();
        let mut moving_proofs = HashMap::new();
        for arg in &args {
            self.process_call_data(arg)?;
            moving_buckets.extend(self.send_buckets(&arg.bucket_ids)?);
            moving_proofs.extend(self.send_proofs(&arg.proof_ids, MoveMethod::AsArgument)?);
        }

        let result = match snode {
            SNodeState::VaultRef(vault_id) => {
                let vault = self.get_local_vault(&vault_id)?;
                let maybe_bucket = vault
                    .main(function.as_str(), args)
                    .map_err(RuntimeError::VaultError)?;
                if let Some(bucket) = maybe_bucket {
                    let bucket_id = self.new_bucket_id()?;
                    self.buckets.insert(bucket_id, bucket);
                    Ok(ScryptoValue::from_value(&scrypto::resource::Bucket(
                        bucket_id,
                    )))
                } else {
                    Ok(ScryptoValue::from_value(&()))
                }
            }
            SNodeState::Bucket(bucket) => match function.as_str() {
                "burn" => bucket.drop(self).map_err(RuntimeError::BucketError),
                _ => Err(RuntimeError::IllegalSystemCall)
            }
            _ => {
                // start a new process
                let mut process = Process::new(self.depth + 1, self.trace, self.track);
                process.caller_auth_zone = &self.auth_zone;

                // move buckets and proofs to the new process.
                process.receive_buckets(moving_buckets)?;
                process.receive_proofs(moving_proofs)?;

                // invoke the main function
                let (result, moving_buckets, moving_proofs) = process.run(&mut snode, function, args)?;

                // move buckets and proofs to this process.
                self.receive_buckets(moving_buckets)?;
                self.receive_proofs(moving_proofs)?;

                // Return borrowed snodes
                match snode {
                    SNodeState::Scrypto(actor, component_state) => {
                        if let Some(component_address) = actor.component_address() {
                            self.track.return_borrowed_global_component(component_address, component_state.unwrap());
                        }
                    }
                    SNodeState::ResourceRef(resource_address, resource_manager) => {
                        self.track.return_borrowed_global_resource_manager(resource_address, resource_manager);
                    }
                    SNodeState::BucketRef(bucket_id, bucket) => {
                        self.buckets.insert(bucket_id, bucket);
                    }
                    _ => {}
                }

                Ok(result)
            }
        }?;

        Ok(result)
    }

    /// Calls a function.
    pub fn call_function(
        &mut self,
        package_address: PackageAddress,
        blueprint_name: &str,
        function: &str,
        args: Vec<ScryptoValue>,
    ) -> Result<ScryptoValue, RuntimeError> {
        re_debug!(self, "Call function started");
        let snode_ref = SNodeRef::Scrypto(ScryptoActor::Blueprint(
            package_address,
            blueprint_name.to_string(),
        ));
        let result = self.call(snode_ref, function.to_string(), args);
        re_debug!(self, "Call function ended");
        result
    }

    /// Calls a method.
    pub fn call_method(
        &mut self,
        component_address: ComponentAddress,
        method: &str,
        args: Vec<ScryptoValue>,
    ) -> Result<ScryptoValue, RuntimeError> {
        re_debug!(self, "Call method started");
        let snode_ref = SNodeRef::Scrypto(ScryptoActor::Component(component_address));
        let result = self.call(snode_ref, method.to_string(), args);
        re_debug!(self, "Call method ended");
        result
    }

    /// Calls the ABI generator of a blueprint.
    // TODO: Remove
    pub fn call_abi(
        &mut self,
        package_address: PackageAddress,
        blueprint_name: &str,
    ) -> Result<ScryptoValue, RuntimeError> {
        re_debug!(self, "Call abi started");

        let mut snode = SNodeState::Scrypto(
            ScryptoActorInfo::blueprint(
                package_address,
                blueprint_name.to_string(),
                format!("{}_abi", blueprint_name),
            ),
            None,
        );

        let mut process = Process::new(self.depth + 1, self.trace, self.track);
        let result = process.run(&mut snode, String::new(), Vec::new()).map(|(r,_,_)| r);

        re_debug!(self, "Call abi ended");
        result
    }

    /// Checks resource leak.
    pub fn check_resource(&self) -> Result<(), RuntimeError> {
        re_debug!(self, "Resource check started");
        let mut success = true;

        for (bucket_id, bucket) in &self.buckets {
            re_warn!(self, "Dangling bucket: {}, {:?}", bucket_id, bucket);
            success = false;
        }
        if !self.worktop.is_empty() {
            re_warn!(self, "Resource worktop is not empty");
            success = false;
        }
        if let Some(wasm_process) = &self.wasm_process_state {
            if !wasm_process.check_resource() {
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

    fn process_return_data(&mut self, validated: &ScryptoValue) -> Result<(), RuntimeError> {
        if !validated.lazy_map_ids.is_empty() {
            return Err(RuntimeError::LazyMapNotAllowed);
        }
        if !validated.vault_ids.is_empty() {
            return Err(RuntimeError::VaultNotAllowed);
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

        let mut lazy_map_ids = HashSet::new();
        for lazy_map_id in validated.lazy_map_ids {
            if lazy_map_ids.contains(&lazy_map_id) {
                return Err(RuntimeError::DuplicateLazyMap(lazy_map_id));
            }
            lazy_map_ids.insert(lazy_map_id);
        }

        let mut vault_ids = HashSet::new();
        for vault_id in validated.vault_ids {
            if vault_ids.contains(&vault_id) {
                return Err(RuntimeError::DuplicateVault(vault_id));
            }
            vault_ids.insert(vault_id);
        }

        // lazy map allowed
        // vaults allowed
        Ok(ComponentObjectRefs {
            lazy_map_ids,
            vault_ids,
        })
    }

    /// Sends buckets to another component/blueprint, either as argument or return
    fn send_buckets(
        &mut self,
        bucket_ids: &[BucketId],
    ) -> Result<HashMap<BucketId, Bucket>, RuntimeError> {
        let mut buckets = HashMap::new();
        for bucket_id in bucket_ids {
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

    /// Receives buckets from another component/blueprint, either as argument or return
    fn receive_buckets(&mut self, buckets: HashMap<BucketId, Bucket>) -> Result<(), RuntimeError> {
        if self.depth == 0 {
            // buckets are aggregated by worktop
            for (_, bucket) in buckets {
                self.worktop
                    .put(bucket)
                    .map_err(RuntimeError::WorktopError)?;
            }
        } else {
            // for component, received buckets go to the "buckets" areas.
            self.buckets.extend(buckets);
        }

        Ok(())
    }

    /// Sends proofs to another component/blueprint, either as argument or return
    fn send_proofs(
        &mut self,
        proof_ids: &[ProofId],
        method: MoveMethod,
    ) -> Result<HashMap<ProofId, Proof>, RuntimeError> {
        let mut proofs = HashMap::new();
        for proof_id in proof_ids {
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

    /// Receives proofs from another component/blueprint, either as argument or return
    fn receive_proofs(&mut self, proofs: HashMap<ProofId, Proof>) -> Result<(), RuntimeError> {
        if self.depth == 0 {
            // proofs are accumulated by auth worktop
            for (_, proof) in proofs {
                self.auth_zone.push(proof);
            }
        } else {
            // for component, received buckets go to the "proofs" areas.
            for (proof_id, proof) in proofs {
                self.proofs.insert(proof_id, proof);
            }
        }

        Ok(())
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

    fn handle_publish(
        &mut self,
        input: PublishPackageInput,
    ) -> Result<PublishPackageOutput, RuntimeError> {
        let package_address = self.publish_package(input.code)?;
        Ok(PublishPackageOutput { package_address })
    }

    fn handle_create_component(
        &mut self,
        input: CreateComponentInput,
    ) -> Result<CreateComponentOutput, RuntimeError> {
        let wasm_process = self
            .wasm_process_state
            .as_mut()
            .ok_or(RuntimeError::IllegalSystemCall)?;

        let data = Self::process_entry_data(&input.state)?;
        let new_objects = wasm_process.process_owned_objects.take(data)?;
        let package_address = wasm_process.vm.actor.package_address().clone();
        let component = Component::new(
            package_address,
            input.blueprint_name,
            input.authorization,
            input.state,
        );
        let component_address = self.track.create_component(component);
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
            .get_component(input.component_address)
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
            InterpreterState::Component { component, .. } => Ok(component.state()),
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
        match &mut wasm_process.interpreter_state {
            InterpreterState::Component {
                ref mut component,
                component_address,
                initial_loaded_object_refs,
                ..
            } => {
                let mut new_set = Self::process_entry_data(&input.state)?;
                new_set.remove(&initial_loaded_object_refs)?;
                let new_objects = wasm_process.process_owned_objects.take(new_set)?;
                self.track
                    .insert_objects_into_component(new_objects, *component_address);

                // TODO: Verify that process_owned_objects is empty

                component.set_state(input.state);
                Ok(())
            }
            _ => Err(RuntimeError::IllegalSystemCall),
        }?;

        Ok(PutComponentStateOutput {})
    }

    fn handle_create_lazy_map(
        &mut self,
        _input: CreateLazyMapInput,
    ) -> Result<CreateLazyMapOutput, RuntimeError> {
        let wasm_process = self
            .wasm_process_state
            .as_mut()
            .ok_or(RuntimeError::IllegalSystemCall)?;
        let lazy_map_id = self.track.new_lazy_map_id();
        wasm_process
            .process_owned_objects
            .lazy_maps
            .insert(lazy_map_id, UnclaimedLazyMap::new());
        Ok(CreateLazyMapOutput { lazy_map_id })
    }

    fn handle_get_lazy_map_entry(
        &mut self,
        input: GetLazyMapEntryInput,
    ) -> Result<GetLazyMapEntryOutput, RuntimeError> {
        let wasm_process = self
            .wasm_process_state
            .as_mut()
            .ok_or(RuntimeError::IllegalSystemCall)?;
        let entry = match wasm_process
            .process_owned_objects
            .get_lazy_map_entry(&input.lazy_map_id, &input.key)
        {
            None => match &mut wasm_process.interpreter_state {
                InterpreterState::Component {
                    initial_loaded_object_refs,
                    additional_object_refs,
                    component_address,
                    ..
                } => {
                    if !initial_loaded_object_refs
                        .lazy_map_ids
                        .contains(&input.lazy_map_id)
                        && !additional_object_refs
                            .lazy_map_ids
                            .contains(&input.lazy_map_id)
                    {
                        return Err(RuntimeError::LazyMapNotFound(input.lazy_map_id));
                    }
                    let value = self.track.get_lazy_map_entry(
                        *component_address,
                        &input.lazy_map_id,
                        &input.key,
                    );
                    if value.is_some() {
                        let map_entry_objects =
                            Self::process_entry_data(&value.as_ref().unwrap()).unwrap();
                        additional_object_refs.extend(map_entry_objects);
                    }

                    Ok(value)
                }
                _ => Err(RuntimeError::LazyMapNotFound(input.lazy_map_id)),
            },
            Some((_, value)) => Ok(value),
        }?;

        Ok(GetLazyMapEntryOutput { value: entry })
    }

    fn handle_put_lazy_map_entry(
        &mut self,
        input: PutLazyMapEntryInput,
    ) -> Result<PutLazyMapEntryOutput, RuntimeError> {
        let wasm_process = self
            .wasm_process_state
            .as_mut()
            .ok_or(RuntimeError::IllegalSystemCall)?;
        let (old_value, lazy_map_state) = match wasm_process
            .process_owned_objects
            .get_lazy_map_entry(&input.lazy_map_id, &input.key)
        {
            None => match &wasm_process.interpreter_state {
                InterpreterState::Component {
                    initial_loaded_object_refs,
                    additional_object_refs,
                    component_address,
                    ..
                } => {
                    if !initial_loaded_object_refs
                        .lazy_map_ids
                        .contains(&input.lazy_map_id)
                        && !additional_object_refs
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

        let new_objects = wasm_process
            .process_owned_objects
            .take(new_entry_object_refs)?;

        match lazy_map_state {
            Uncommitted { root } => {
                wasm_process.process_owned_objects.insert_lazy_map_entry(
                    &input.lazy_map_id,
                    input.key,
                    input.value,
                );
                wasm_process
                    .process_owned_objects
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

    fn handle_create_vault(
        &mut self,
        input: CreateEmptyVaultInput,
    ) -> Result<CreateEmptyVaultOutput, RuntimeError> {
        let wasm_process = self
            .wasm_process_state
            .as_mut()
            .ok_or(RuntimeError::IllegalSystemCall)?;
        let definition = self
            .track
            .get_resource_manager(&input.resource_address)
            .ok_or(RuntimeError::ResourceManagerNotFound(
                input.resource_address,
            ))?;

        let new_vault = Vault::new(ResourceContainer::new_empty(
            input.resource_address,
            definition.resource_type(),
        ));
        let vault_id = self.track.new_vault_id();
        wasm_process
            .process_owned_objects
            .vaults
            .insert(vault_id, new_vault);

        Ok(CreateEmptyVaultOutput { vault_id })
    }

    fn get_local_vault(&mut self, vault_id: &VaultId) -> Result<&mut Vault, RuntimeError> {
        let wasm_process = self
            .wasm_process_state
            .as_mut()
            .ok_or(RuntimeError::IllegalSystemCall)?;
        match wasm_process.process_owned_objects.get_vault_mut(vault_id) {
            Some(vault) => Ok(vault),
            None => match &wasm_process.interpreter_state {
                InterpreterState::Component {
                    component_address,
                    initial_loaded_object_refs,
                    additional_object_refs,
                    ..
                } => {
                    if !initial_loaded_object_refs.vault_ids.contains(vault_id)
                        && !additional_object_refs.vault_ids.contains(vault_id)
                    {
                        return Err(RuntimeError::VaultNotFound(*vault_id));
                    }
                    let vault = self.track.get_vault_mut(component_address, vault_id);
                    Ok(vault)
                }
                _ => Err(RuntimeError::VaultNotFound(*vault_id)),
            },
        }
    }

    fn handle_put_into_vault(
        &mut self,
        input: PutIntoVaultInput,
    ) -> Result<PutIntoVaultOutput, RuntimeError> {
        // TODO: restrict access

        let bucket = self
            .buckets
            .remove(&input.bucket_id)
            .ok_or(RuntimeError::BucketNotFound(input.bucket_id))?;

        self.get_local_vault(&input.vault_id)?
            .put(bucket)
            .map_err(|e| RuntimeError::VaultError(VaultError::ResourceContainerError(e)))?;

        Ok(PutIntoVaultOutput {})
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

        let result = self.call(input.snode_ref, input.function, validated_args)?;
        Ok(InvokeSNodeOutput { rtn: result.raw })
    }

    fn handle_get_non_fungible_ids_in_vault(
        &mut self,
        input: GetNonFungibleIdsInVaultInput,
    ) -> Result<GetNonFungibleIdsInVaultOutput, RuntimeError> {
        let vault = self.get_local_vault(&input.vault_id)?;
        let non_fungible_ids = vault
            .total_ids()
            .map_err(|e| RuntimeError::VaultError(VaultError::ResourceContainerError(e)))?
            .into_iter()
            .collect();

        Ok(GetNonFungibleIdsInVaultOutput { non_fungible_ids })
    }

    fn handle_get_vault_amount(
        &mut self,
        input: GetVaultAmountInput,
    ) -> Result<GetVaultAmountOutput, RuntimeError> {
        let vault = self.get_local_vault(&input.vault_id)?;

        Ok(GetVaultAmountOutput {
            amount: vault.total_amount(),
        })
    }

    fn handle_get_vault_resource_address(
        &mut self,
        input: GetVaultResourceAddressInput,
    ) -> Result<GetVaultResourceAddressOutput, RuntimeError> {
        let vault = self.get_local_vault(&input.vault_id)?;

        Ok(GetVaultResourceAddressOutput {
            resource_address: vault.resource_address(),
        })
    }

    fn handle_create_bucket(
        &mut self,
        input: CreateEmptyBucketInput,
    ) -> Result<CreateEmptyBucketOutput, RuntimeError> {
        let definition = self
            .track
            .get_resource_manager(&input.resource_address)
            .ok_or(RuntimeError::ResourceManagerNotFound(
                input.resource_address,
            ))?;

        let new_bucket = Bucket::new(ResourceContainer::new_empty(
            input.resource_address,
            definition.resource_type(),
        ));
        let bucket_id = self.new_bucket_id()?;
        self.buckets.insert(bucket_id, new_bucket);

        Ok(CreateEmptyBucketOutput { bucket_id })
    }

    fn handle_create_bucket_proof(
        &mut self,
        input: CreateBucketProofInput,
    ) -> Result<CreateBucketProofOutput, RuntimeError> {
        Ok(CreateBucketProofOutput {
            proof_id: self.create_bucket_proof(input.bucket_id)?,
        })
    }

    fn handle_create_vault_proof(
        &mut self,
        input: CreateVaultProofInput,
    ) -> Result<CreateVaultProofOutput, RuntimeError> {
        Ok(CreateVaultProofOutput {
            proof_id: self.create_vault_proof(input.vault_id)?,
        })
    }

    fn handle_create_vault_proof_by_amount(
        &mut self,
        input: CreateVaultProofByAmountInput,
    ) -> Result<CreateVaultProofByAmountOutput, RuntimeError> {
        Ok(CreateVaultProofByAmountOutput {
            proof_id: self.create_vault_proof_by_amount(input.vault_id, input.amount)?,
        })
    }

    fn handle_create_vault_proof_by_ids(
        &mut self,
        input: CreateVaultProofByIdsInput,
    ) -> Result<CreateVaultProofByIdsOutput, RuntimeError> {
        Ok(CreateVaultProofByIdsOutput {
            proof_id: self.create_vault_proof_by_ids(input.vault_id, &input.ids)?,
        })
    }

    fn handle_create_auth_zone_proof(
        &mut self,
        input: CreateAuthZoneProofInput,
    ) -> Result<CreateAuthZoneProofOutput, RuntimeError> {
        Ok(CreateAuthZoneProofOutput {
            proof_id: self.create_auth_zone_proof(input.resource_address)?,
        })
    }

    fn handle_create_auth_zone_proof_by_amount(
        &mut self,
        input: CreateAuthZoneProofByAmountInput,
    ) -> Result<CreateAuthZoneProofByAmountOutput, RuntimeError> {
        Ok(CreateAuthZoneProofByAmountOutput {
            proof_id: self
                .create_auth_zone_proof_by_amount(input.amount, input.resource_address)?,
        })
    }

    fn handle_create_auth_zone_proof_by_ids(
        &mut self,
        input: CreateAuthZoneProofByIdsInput,
    ) -> Result<CreateAuthZoneProofByIdsOutput, RuntimeError> {
        Ok(CreateAuthZoneProofByIdsOutput {
            proof_id: self.create_auth_zone_proof_by_ids(&input.ids, input.resource_address)?,
        })
    }

    fn handle_drop_proof(
        &mut self,
        input: DropProofInput,
    ) -> Result<DropProofOutput, RuntimeError> {
        self.drop_proof(input.proof_id)?;

        Ok(DropProofOutput {})
    }

    fn handle_get_proof_amount(
        &mut self,
        input: GetProofAmountInput,
    ) -> Result<GetProofAmountOutput, RuntimeError> {
        let proof = self
            .proofs
            .get(&input.proof_id)
            .ok_or(RuntimeError::ProofNotFound(input.proof_id))?;

        Ok(GetProofAmountOutput {
            amount: proof.total_amount(),
        })
    }

    fn handle_get_proof_resource_address(
        &mut self,
        input: GetProofResourceAddressInput,
    ) -> Result<GetProofResourceAddressOutput, RuntimeError> {
        let proof = self
            .proofs
            .get(&input.proof_id)
            .ok_or(RuntimeError::ProofNotFound(input.proof_id))?;

        Ok(GetProofResourceAddressOutput {
            resource_address: proof.resource_address(),
        })
    }

    fn handle_get_non_fungible_ids_in_proof(
        &mut self,
        input: GetNonFungibleIdsInProofInput,
    ) -> Result<GetNonFungibleIdsInProofOutput, RuntimeError> {
        let proof = self
            .proofs
            .get(&input.proof_id)
            .ok_or(RuntimeError::ProofNotFound(input.proof_id))?;

        Ok(GetNonFungibleIdsInProofOutput {
            non_fungible_ids: proof
                .total_ids()
                .map_err(RuntimeError::ProofError)?
                .into_iter()
                .collect(),
        })
    }

    fn handle_clone_proof(
        &mut self,
        input: CloneProofInput,
    ) -> Result<CloneProofOutput, RuntimeError> {
        Ok(CloneProofOutput {
            proof_id: self.clone_proof(input.proof_id)?,
        })
    }

    fn handle_push_to_auth_zone(
        &mut self,
        input: PushToAuthZoneInput,
    ) -> Result<PushToAuthZoneOutput, RuntimeError> {
        self.push_to_auth_zone(input.proof_id)
            .map(|_| PushToAuthZoneOutput {})
    }

    fn handle_pop_from_auth_zone(
        &mut self,
        _input: PopFromAuthZoneInput,
    ) -> Result<PopFromAuthZoneOutput, RuntimeError> {
        self.pop_from_auth_zone()
            .map(|proof_id| PopFromAuthZoneOutput { proof_id })
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

impl<'r, 'l, L: SubstateStore> SystemApi for Process<'r, 'l, L> {
    fn get_non_fungible(
        &mut self,
        non_fungible_address: &NonFungibleAddress,
    ) -> Option<&NonFungible> {
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

    fn create_bucket(&mut self, container: ResourceContainer) -> Result<BucketId, RuntimeError> {
        let bucket_id = self.new_bucket_id()?;
        self.buckets.insert(bucket_id, Bucket::new(container));
        Ok(bucket_id)
    }

    fn take_bucket(&mut self, bucket_id: BucketId) -> Result<Bucket, RuntimeError> {
        self
            .buckets
            .remove(&bucket_id)
            .ok_or(RuntimeError::BucketNotFound(bucket_id))
    }

    fn create_resource(&mut self, resource_manager: ResourceManager) -> ResourceAddress {
        self.track.create_resource_manager(resource_manager)
    }
}

impl<'r, 'l, L: SubstateStore> Externals for Process<'r, 'l, L> {
    fn invoke_index(
        &mut self,
        index: usize,
        args: RuntimeArgs,
    ) -> Result<Option<RuntimeValue>, Trap> {
        match index {
            ENGINE_FUNCTION_INDEX => {
                let operation: u32 = args.nth_checked(0)?;
                match operation {
                    PUBLISH_PACKAGE => self.handle(args, Self::handle_publish),

                    CREATE_COMPONENT => self.handle(args, Self::handle_create_component),
                    GET_COMPONENT_INFO => self.handle(args, Self::handle_get_component_info),
                    GET_COMPONENT_STATE => self.handle(args, Self::handle_get_component_state),
                    PUT_COMPONENT_STATE => self.handle(args, Self::handle_put_component_state),

                    CREATE_LAZY_MAP => self.handle(args, Self::handle_create_lazy_map),
                    GET_LAZY_MAP_ENTRY => self.handle(args, Self::handle_get_lazy_map_entry),
                    PUT_LAZY_MAP_ENTRY => self.handle(args, Self::handle_put_lazy_map_entry),

                    CREATE_EMPTY_VAULT => self.handle(args, Self::handle_create_vault),
                    PUT_INTO_VAULT => self.handle(args, Self::handle_put_into_vault),
                    GET_VAULT_AMOUNT => self.handle(args, Self::handle_get_vault_amount),
                    GET_VAULT_RESOURCE_ADDRESS => {
                        self.handle(args, Self::handle_get_vault_resource_address)
                    }
                    GET_NON_FUNGIBLE_IDS_IN_VAULT => {
                        self.handle(args, Self::handle_get_non_fungible_ids_in_vault)
                    }

                    CREATE_EMPTY_BUCKET => self.handle(args, Self::handle_create_bucket),

                    CREATE_BUCKET_PROOF => self.handle(args, Self::handle_create_bucket_proof),
                    CREATE_VAULT_PROOF => self.handle(args, Self::handle_create_vault_proof),
                    CREATE_VAULT_PROOF_BY_AMOUNT => {
                        self.handle(args, Self::handle_create_vault_proof_by_amount)
                    }
                    CREATE_VAULT_PROOF_BY_IDS => {
                        self.handle(args, Self::handle_create_vault_proof_by_ids)
                    }
                    CREATE_AUTH_ZONE_PROOF => {
                        self.handle(args, Self::handle_create_auth_zone_proof)
                    }
                    CREATE_AUTH_ZONE_PROOF_BY_AMOUNT => {
                        self.handle(args, Self::handle_create_auth_zone_proof_by_amount)
                    }
                    CREATE_AUTH_ZONE_PROOF_BY_IDS => {
                        self.handle(args, Self::handle_create_auth_zone_proof_by_ids)
                    }
                    DROP_PROOF => self.handle(args, Self::handle_drop_proof),
                    GET_PROOF_AMOUNT => self.handle(args, Self::handle_get_proof_amount),
                    GET_PROOF_RESOURCE_ADDRESS => {
                        self.handle(args, Self::handle_get_proof_resource_address)
                    }
                    GET_NON_FUNGIBLE_IDS_IN_PROOF => {
                        self.handle(args, Self::handle_get_non_fungible_ids_in_proof)
                    }
                    CLONE_PROOF => self.handle(args, Self::handle_clone_proof),
                    PUSH_TO_AUTH_ZONE => self.handle(args, Self::handle_push_to_auth_zone),
                    POP_FROM_AUTH_ZONE => self.handle(args, Self::handle_pop_from_auth_zone),

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
