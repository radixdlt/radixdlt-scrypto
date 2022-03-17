use colored::*;
use sbor::*;
use scrypto::buffer::*;
use scrypto::engine::api::*;
use scrypto::engine::types::*;
use scrypto::prelude::NonFungibleAddress;
use scrypto::rust::borrow::ToOwned;
use scrypto::rust::collections::*;
use scrypto::rust::fmt;
use scrypto::rust::format;
use scrypto::rust::string::String;
use scrypto::rust::vec;
use scrypto::rust::vec::Vec;
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

/// Represents an interpreter instance.
pub struct Interpreter {
    invocation: Invocation,
    module: ModuleRef,
    memory: MemoryRef,
}

/// Keeps invocation information.
#[derive(Debug, Clone)]
pub struct Invocation {
    actor: Actor,
    package_id: PackageId,
    export_name: String,
    function: String,
    args: Vec<ValidatedData>,
}

/// Qualitative states for a WASM process
#[derive(Debug)]
enum InterpreterState {
    Blueprint,
    Component {
        component_id: ComponentId,
        state: Vec<u8>,
        initial_loaded_object_refs: ComponentObjectRefs,
        additional_object_refs: ComponentObjectRefs,
    },
}

/// Top level state machine for a process. Empty currently only
/// refers to the initial process since it doesn't run on a wasm interpreter (yet)
#[allow(dead_code)]
struct WasmProcess {
    /// The call depth
    depth: usize,
    trace: bool,
    vm: Interpreter,
    interpreter_state: InterpreterState,
    process_owned_objects: ComponentObjects,
}

impl WasmProcess {
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
    Committed { component_id: ComponentId },
}

impl<'s, S: SubstateStore> Track<'s, S> {
    fn insert_objects_into_component(
        &mut self,
        new_objects: ComponentObjects,
        component_id: ComponentId,
    ) {
        for (vault_id, vault) in new_objects.vaults {
            self.put_vault(component_id, vault_id, vault);
        }
        for (lazy_map_id, unclaimed) in new_objects.lazy_maps {
            for (k, v) in unclaimed.lazy_map {
                self.put_lazy_map_entry(component_id, lazy_map_id, k, v);
            }
            for (child_lazy_map_id, child_lazy_map) in unclaimed.descendent_lazy_maps {
                for (k, v) in child_lazy_map {
                    self.put_lazy_map_entry(component_id, child_lazy_map_id, k, v);
                }
            }
            for (vault_id, vault) in unclaimed.descendent_vaults {
                self.put_vault(component_id, vault_id, vault);
            }
        }
    }
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
    /// The buckets that will be moved to another process SHORTLY.
    moving_buckets: HashMap<BucketId, Bucket>,
    /// The proofs that will be moved to another process SHORTLY.
    moving_proofs: HashMap<ProofId, Proof>,

    /// State for the given wasm process, empty only on the root process
    /// (root process cannot create components nor is a component itself)
    wasm_process_state: Option<WasmProcess>,

    /// ID allocator for buckets and proofs created within transaction.
    id_allocator: IdAllocator,
    /// Resources collected from previous returns or self.
    worktop: Worktop,
    /// Proofs collected from previous returns or self. Also used for system authorization.
    auth_worktop: Vec<Proof>,
    /// The caller's auth worktop
    caller_auth_worktop: &'r [Proof],
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
            moving_buckets: HashMap::new(),
            moving_proofs: HashMap::new(),
            wasm_process_state: None,
            id_allocator: IdAllocator::new(IdSpace::Transaction),
            worktop: Worktop::new(),
            auth_worktop: Vec::new(),
            caller_auth_worktop: &[],
        }
    }

    // (Transaction ONLY) Takes resource from worktop and returns a bucket.
    pub fn take_from_worktop(
        &mut self,
        resource: ResourceSpecifier,
    ) -> Result<ValidatedData, RuntimeError> {
        re_debug!(self, "(Transaction) Taking from worktop: {:?}", resource);
        let new_bucket_id = self
            .id_allocator
            .new_bucket_id()
            .map_err(RuntimeError::IdAllocatorError)?;
        let bucket = match resource {
            ResourceSpecifier::Some(amount, resource_def_id) => match amount {
                Amount::Fungible { amount } => self.worktop.take(amount, resource_def_id),
                Amount::NonFungible { ids } => {
                    self.worktop.take_non_fungibles(&ids, resource_def_id)
                }
            }
            .map_err(RuntimeError::WorktopError)?,
            ResourceSpecifier::All(resource_def_id) => {
                match self
                    .worktop
                    .take_all(resource_def_id)
                    .map_err(RuntimeError::WorktopError)?
                {
                    Some(bucket) => bucket,
                    None => {
                        let resource_def = self
                            .track
                            .get_resource_def(&resource_def_id)
                            .ok_or(RuntimeError::ResourceDefNotFound(resource_def_id))?;
                        Bucket::new(ResourceContainer::new_empty(
                            resource_def_id,
                            resource_def.resource_type(),
                        ))
                    }
                }
            }
        };
        self.buckets.insert(new_bucket_id, bucket);
        Ok(ValidatedData::from_value(&scrypto::resource::Bucket(
            new_bucket_id,
        )))
    }

    // (Transaction ONLY) Returns resource back to worktop.
    pub fn return_to_worktop(
        &mut self,
        bucket_id: BucketId,
    ) -> Result<ValidatedData, RuntimeError> {
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
        Ok(ValidatedData::from_value(&()))
    }

    // (Transaction ONLY) Assert worktop contains at least this amount.
    pub fn assert_worktop_contains(
        &mut self,
        amount: Decimal,
        resource_def_id: ResourceDefId,
    ) -> Result<ValidatedData, RuntimeError> {
        re_debug!(
            self,
            "(Transaction) Asserting worktop contains: amount = {}, resource_def_id = {}",
            amount,
            resource_def_id
        );

        if !self.worktop.contains(amount, resource_def_id) {
            Err(RuntimeError::AssertionFailed)
        } else {
            Ok(ValidatedData::from_value(&()))
        }
    }

    // Takes a proof from the auth worktop.
    pub fn pop_from_auth_worktop(&mut self) -> Result<ProofId, RuntimeError> {
        re_debug!(self, "Popping from auth worktop");
        if self.auth_worktop.is_empty() {
            return Err(RuntimeError::EmptyAuthWorkTop);
        }

        let new_proof_id = self
            .id_allocator
            .new_proof_id()
            .map_err(RuntimeError::IdAllocatorError)?;
        let proof = self.auth_worktop.remove(self.auth_worktop.len() - 1);
        self.proofs.insert(new_proof_id, proof);
        Ok(new_proof_id)
    }

    // Puts a proof onto the auth worktop.
    pub fn push_onto_auth_worktop(&mut self, proof_id: ProofId) -> Result<(), RuntimeError> {
        re_debug!(self, "Pushing onto auth worktop: proof_id = {}", proof_id);

        let proof = self
            .proofs
            .remove(&proof_id)
            .ok_or(RuntimeError::ProofNotFound(proof_id))?;

        self.auth_worktop.push(proof);

        Ok(())
    }

    // Creates a proof.
    pub fn create_bucket_proof(&mut self, bucket_id: BucketId) -> Result<ProofId, RuntimeError> {
        re_debug!(self, "Creating proof: bucket_id = {}", bucket_id);

        let bucket = self
            .buckets
            .get_mut(&bucket_id)
            .ok_or(RuntimeError::BucketNotFound(bucket_id))?;

        let new_proof_id = if self.depth == 0 {
            self.id_allocator
                .new_proof_id()
                .map_err(RuntimeError::IdAllocatorError)?
        } else {
            self.track.new_proof_id()
        };
        let new_proof = Proof::new(bucket.refer_container()).map_err(RuntimeError::ProofError)?;
        self.proofs.insert(new_proof_id, new_proof);

        Ok(new_proof_id)
    }

    // Clone a proof.
    pub fn clone_proof(&mut self, proof_id: ProofId) -> Result<ProofId, RuntimeError> {
        re_debug!(self, "Cloning proof: proof_id = {}", proof_id);

        let proof = self
            .proofs
            .get(&proof_id)
            .ok_or(RuntimeError::ProofNotFound(proof_id))?;

        let new_proof_id = if self.depth == 0 {
            self.id_allocator
                .new_proof_id()
                .map_err(RuntimeError::IdAllocatorError)?
        } else {
            self.track.new_proof_id()
        };
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

        proof.settle();

        Ok(())
    }

    /// (Transaction ONLY) Calls a method.
    pub fn call_method_with_all_resources(
        &mut self,
        component_id: ComponentId,
        method: &str,
    ) -> Result<ValidatedData, RuntimeError> {
        re_debug!(
            self,
            "(Transaction) Calling method with all resources started"
        );

        // 1. Drop all proofs to unlock the buckets
        self.drop_all_proofs()?;

        // 2. Move collected resource to temp buckets
        for id in self.worktop.resource_def_ids() {
            if let Some(bucket) = self
                .worktop
                .take_all(id)
                .map_err(RuntimeError::WorktopError)?
            {
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
        let invocation = self.prepare_call_method(
            component_id,
            method,
            vec![ValidatedData::from_slice(&scrypto_encode(&to_deposit)).unwrap()],
        )?;
        let result = self.call(invocation);

        re_debug!(
            self,
            "(Transaction) Calling method with all resources ended"
        );
        result
    }

    /// (Transaction ONLY) Publishes a package.
    pub fn publish_package(&mut self, code: Vec<u8>) -> Result<ValidatedData, RuntimeError> {
        re_debug!(self, "(Transaction) Publishing a package");

        validate_module(&code).map_err(RuntimeError::WasmValidationError)?;
        let package_id = self.track.create_package(Package::new(code));

        Ok(ValidatedData::from_slice(&scrypto_encode(&package_id)).unwrap())
    }

    /// (SYSTEM ONLY)  Creates a proof which references a virtual bucket
    pub fn create_virtual_proof(&mut self, proof_id: ProofId, bucket: Bucket) {
        let proof = Proof::new(bucket.refer_container()).unwrap();
        self.proofs.insert(proof_id, proof);
    }

    /// Moves buckets and proofs into this process.
    pub fn move_in_resources(
        &mut self,
        buckets: HashMap<BucketId, Bucket>,
        proofs: HashMap<ProofId, Proof>,
    ) -> Result<(), RuntimeError> {
        if self.depth == 0 {
            assert!(proofs.is_empty());

            for (_, bucket) in buckets {
                self.worktop
                    .put(bucket)
                    .map_err(RuntimeError::WorktopError)?;
            }
        } else {
            self.proofs.extend(proofs);
            self.buckets.extend(buckets);
        }

        Ok(())
    }

    /// Moves all marked buckets and proofs from this process.
    pub fn move_out_resources(&mut self) -> (HashMap<BucketId, Bucket>, HashMap<ProofId, Proof>) {
        let buckets = self.moving_buckets.drain().collect();
        let proofs = self.moving_proofs.drain().collect();
        (buckets, proofs)
    }

    /// Runs the given export within this process.
    pub fn run(&mut self, invocation: Invocation) -> Result<ValidatedData, RuntimeError> {
        #[cfg(not(feature = "alloc"))]
        let now = std::time::Instant::now();
        re_info!(
            self,
            "Run started: package = {}, export = {}",
            invocation.package_id,
            invocation.export_name
        );

        // Load the code
        let (module, memory) = self
            .track
            .load_module(invocation.package_id)
            .ok_or(RuntimeError::PackageNotFound(invocation.package_id))?;
        let vm = Interpreter {
            invocation: invocation.clone(),
            module: module.clone(),
            memory,
        };
        self.wasm_process_state = Some(WasmProcess {
            depth: self.depth,
            trace: self.trace,
            vm,
            interpreter_state: match invocation.actor {
                Actor::Blueprint(..) => Ok(InterpreterState::Blueprint),
                Actor::Component(component_id) => {
                    let component = self.track.get_component(component_id.clone()).unwrap();
                    component.check_auth(&invocation.function, self.caller_auth_worktop)?;

                    let initial_loaded_object_refs =
                        Self::process_entry_data(component.state()).unwrap();
                    let state = component.state().to_vec();
                    let component = InterpreterState::Component {
                        state,
                        component_id,
                        initial_loaded_object_refs,
                        additional_object_refs: ComponentObjectRefs::new(),
                    };
                    Ok(component)
                }
            }?,
            process_owned_objects: ComponentObjects::new(),
        });

        // run the main function
        let result = module.invoke_export(invocation.export_name.as_str(), &[], self);
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

        // move resource based on return data
        let output = match rtn {
            RuntimeValue::I32(ptr) => {
                let data = ValidatedData::from_slice(&self.read_bytes(ptr)?)
                    .map_err(RuntimeError::DataValidationError)?;
                self.process_call_data(&data, false)?;
                data
            }
            _ => {
                return Err(RuntimeError::InvalidReturnType);
            }
        };

        #[cfg(not(feature = "alloc"))]
        re_info!(
            self,
            "Run ended: time elapsed = {} ms",
            now.elapsed().as_millis()
        );
        #[cfg(feature = "alloc")]
        re_info!(self, "Run ended");

        Ok(output)
    }

    /// Prepares a function call.
    pub fn prepare_call_function(
        &mut self,
        package_id: PackageId,
        blueprint_name: &str,
        function: &str,
        args: Vec<ValidatedData>,
    ) -> Result<Invocation, RuntimeError> {
        Ok(Invocation {
            actor: Actor::Blueprint(package_id, blueprint_name.to_owned()),
            package_id: package_id,
            export_name: format!("{}_main", blueprint_name),
            function: function.to_owned(),
            args,
        })
    }

    /// Prepares a method call.
    pub fn prepare_call_method(
        &mut self,
        component_id: ComponentId,
        method: &str,
        args: Vec<ValidatedData>,
    ) -> Result<Invocation, RuntimeError> {
        let component = self
            .track
            .get_component(component_id)
            .ok_or(RuntimeError::ComponentNotFound(component_id))?
            .clone();
        let mut args_with_self =
            vec![ValidatedData::from_slice(&scrypto_encode(&component_id)).unwrap()];
        args_with_self.extend(args);

        Ok(Invocation {
            actor: Actor::Component(component_id),
            package_id: component.package_id(),
            export_name: format!("{}_main", component.blueprint_name()),
            function: method.to_owned(),
            args: args_with_self,
        })
    }

    /// Prepares an ABI call.
    pub fn prepare_call_abi(
        &mut self,
        package_id: PackageId,
        blueprint_name: &str,
    ) -> Result<Invocation, RuntimeError> {
        Ok(Invocation {
            actor: Actor::Blueprint(package_id, blueprint_name.to_owned()),
            package_id: package_id,
            export_name: format!("{}_abi", blueprint_name),
            function: String::new(),
            args: Vec::new(),
        })
    }

    /// Calls a function/method.
    pub fn call(&mut self, invocation: Invocation) -> Result<ValidatedData, RuntimeError> {
        // move resource
        for arg in &invocation.args {
            self.process_call_data(arg, true)?;
        }
        let (buckets_out, proofs_out) = self.move_out_resources();
        let mut process = Process::new(self.depth + 1, self.trace, self.track);
        process.caller_auth_worktop = &self.auth_worktop;
        process.move_in_resources(buckets_out, proofs_out)?;

        // run the function
        let result = process.run(invocation)?;
        process.drop_all_proofs()?;
        process.check_resource()?;

        // move resource
        let (buckets_in, proofs_in) = process.move_out_resources();
        self.move_in_resources(buckets_in, proofs_in)?;

        Ok(result)
    }

    /// Calls a function.
    pub fn call_function(
        &mut self,
        package_id: PackageId,
        blueprint_name: &str,
        function: &str,
        args: Vec<ValidatedData>,
    ) -> Result<ValidatedData, RuntimeError> {
        re_debug!(self, "Call function started");
        let invocation = self.prepare_call_function(package_id, blueprint_name, function, args)?;
        let result = self.call(invocation);
        re_debug!(self, "Call function ended");
        result
    }

    /// Calls a method.
    pub fn call_method(
        &mut self,
        component_id: ComponentId,
        method: &str,
        args: Vec<ValidatedData>,
    ) -> Result<ValidatedData, RuntimeError> {
        re_debug!(self, "Call method started");
        let invocation = self.prepare_call_method(component_id, method, args)?;
        let result = self.call(invocation);
        re_debug!(self, "Call method ended");
        result
    }

    /// Calls the ABI generator of a blueprint.
    pub fn call_abi(
        &mut self,
        package_id: PackageId,
        blueprint_name: &str,
    ) -> Result<ValidatedData, RuntimeError> {
        re_debug!(self, "Call abi started");
        let invocation = self.prepare_call_abi(package_id, blueprint_name)?;
        let result = self.call(invocation);
        re_debug!(self, "Call abi ended");
        result
    }

    /// Drops all proofs owned by this process.
    pub fn drop_all_proofs(&mut self) -> Result<(), RuntimeError> {
        let proof_ids: Vec<ProofId> = self.proofs.keys().cloned().collect();
        for proof_id in proof_ids {
            self.proofs
                .remove(&proof_id)
                .ok_or(RuntimeError::ProofNotFound(proof_id))?;
        }
        Ok(())
    }

    /// Checks resource leak.
    pub fn check_resource(&self) -> Result<(), RuntimeError> {
        re_debug!(self, "Resource check started");
        let mut success = true;

        for (bucket_id, bucket) in &self.buckets {
            re_warn!(self, "Dangling bucket: {}, {:?}", bucket_id, bucket);
            success = false;
        }
        for resource_def_id in self.worktop.resource_def_ids() {
            if let Some(container) = self.worktop.borrow_container(resource_def_id) {
                let total_amount = container.total_amount();
                if !total_amount.is_zero() {
                    re_warn!(
                        self,
                        "Dangling resource on worktop: {}, {:?}",
                        resource_def_id,
                        total_amount
                    );
                    success = false;
                }
            }
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

    fn process_call_data(
        &mut self,
        validated: &ValidatedData,
        is_argument: bool,
    ) -> Result<(), RuntimeError> {
        self.move_buckets(&validated.bucket_ids)?;
        if is_argument {
            self.move_proofs(&validated.proof_ids)?;
        } else {
            if !validated.proof_ids.is_empty() {
                return Err(RuntimeError::ProofNotAllowed);
            }
        }
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
            ValidatedData::from_slice(data).map_err(RuntimeError::DataValidationError)?;
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

    fn process_non_fungible_data(&mut self, data: &[u8]) -> Result<ValidatedData, RuntimeError> {
        let validated =
            ValidatedData::from_slice(data).map_err(RuntimeError::DataValidationError)?;
        if !validated.bucket_ids.is_empty() {
            return Err(RuntimeError::BucketNotAllowed);
        }
        if !validated.proof_ids.is_empty() {
            return Err(RuntimeError::ProofNotAllowed);
        }
        if !validated.lazy_map_ids.is_empty() {
            return Err(RuntimeError::LazyMapNotAllowed);
        }
        if !validated.vault_ids.is_empty() {
            return Err(RuntimeError::VaultNotAllowed);
        }
        Ok(validated)
    }

    /// Remove transient buckets from this process
    fn move_buckets(&mut self, buckets: &[BucketId]) -> Result<(), RuntimeError> {
        for bucket_id in buckets {
            let bucket = self
                .buckets
                .remove(bucket_id)
                .ok_or(RuntimeError::BucketNotFound(*bucket_id))?;
            re_debug!(self, "Moving bucket: {}, {:?}", bucket_id, bucket);
            if bucket.is_locked() {
                return Err(RuntimeError::CantMoveLockedBucket);
            }
            self.moving_buckets.insert(*bucket_id, bucket);
        }
        Ok(())
    }

    /// Remove transient buckets from this process
    fn move_proofs(&mut self, proofs: &[ProofId]) -> Result<(), RuntimeError> {
        for proof_id in proofs {
            let proof = self
                .proofs
                .remove(proof_id)
                .ok_or(RuntimeError::ProofNotFound(*proof_id))?;
            re_debug!(self, "Moving proof: {}, {:?}", proof_id, proof);
            self.moving_proofs.insert(*proof_id, proof);
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

    /// Read a byte array from wasm instance.
    fn read_bytes(&mut self, ptr: i32) -> Result<Vec<u8>, RuntimeError> {
        let wasm_process = self.wasm_process_state.as_ref().unwrap();
        // read length
        let len: u32 = wasm_process
            .vm
            .memory
            .get_value(ptr as u32)
            .map_err(|_| RuntimeError::MemoryAccessError)?;

        // SECURITY: meter before allocating memory
        let mut data = vec![0u8; len as usize];
        wasm_process
            .vm
            .memory
            .get_into((ptr + 4) as u32, &mut data)
            .map_err(|_| RuntimeError::MemoryAccessError)?;

        // free the buffer
        wasm_process
            .vm
            .module
            .invoke_export(
                "scrypto_free",
                &[RuntimeValue::I32(ptr as i32)],
                &mut NopExternals,
            )
            .map_err(|_| RuntimeError::MemoryAccessError)?;

        Ok(data.to_vec())
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
        validate_module(&input.code).map_err(RuntimeError::WasmValidationError)?;

        let package_id = self.track.create_package(Package::new(input.code));

        Ok(PublishPackageOutput { package_id })
    }

    fn handle_call_function(
        &mut self,
        input: CallFunctionInput,
    ) -> Result<CallFunctionOutput, RuntimeError> {
        let mut validated_args = Vec::new();
        for arg in input.args {
            validated_args
                .push(ValidatedData::from_slice(&arg).map_err(RuntimeError::DataValidationError)?);
        }

        re_debug!(
            self,
            "CALL started: package_id = {}, blueprint_name = {}, function = {}, args = {:?}",
            input.package_id,
            input.blueprint_name,
            input.function,
            validated_args
        );

        let invocation = self.prepare_call_function(
            input.package_id,
            &input.blueprint_name,
            input.function.as_str(),
            validated_args,
        )?;
        let result = self.call(invocation);

        re_debug!(self, "CALL finished");
        Ok(CallFunctionOutput { rtn: result?.raw })
    }

    fn handle_call_method(
        &mut self,
        input: CallMethodInput,
    ) -> Result<CallMethodOutput, RuntimeError> {
        let mut validated_args = Vec::new();
        for arg in input.args {
            validated_args
                .push(ValidatedData::from_slice(&arg).map_err(RuntimeError::DataValidationError)?);
        }

        re_debug!(
            self,
            "CALL started: component = {}, method = {}, args = {:?}",
            input.component_id,
            input.method,
            validated_args
        );

        let invocation =
            self.prepare_call_method(input.component_id, input.method.as_str(), validated_args)?;
        let result = self.call(invocation);

        re_debug!(self, "CALL finished");
        Ok(CallMethodOutput { rtn: result?.raw })
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
        let component = Component::new(
            wasm_process.vm.invocation.package_id,
            input.blueprint_name,
            input.state,
            input.sys_auth,
        );
        let component_id = self.track.create_component(component);
        self.track
            .insert_objects_into_component(new_objects, component_id);

        Ok(CreateComponentOutput { component_id })
    }

    fn handle_get_component_info(
        &mut self,
        input: GetComponentInfoInput,
    ) -> Result<GetComponentInfoOutput, RuntimeError> {
        let component = self
            .track
            .get_component(input.component_id)
            .ok_or(RuntimeError::ComponentNotFound(input.component_id))?;

        Ok(GetComponentInfoOutput {
            package_id: component.package_id(),
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
        let return_state = match &wasm_process.interpreter_state {
            InterpreterState::Component { state, .. } => Ok(state),
            _ => Err(RuntimeError::IllegalSystemCall),
        }?;
        let state = return_state.to_vec();
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
        match &wasm_process.interpreter_state {
            InterpreterState::Component {
                component_id,
                initial_loaded_object_refs,
                ..
            } => {
                let mut new_set = Self::process_entry_data(&input.state)?;
                new_set.remove(&initial_loaded_object_refs)?;
                let new_objects = wasm_process.process_owned_objects.take(new_set)?;
                self.track
                    .insert_objects_into_component(new_objects, *component_id);

                // TODO: Verify that process_owned_objects is empty

                let component = self.track.get_component_mut(*component_id).unwrap();
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
                    component_id,
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
                        *component_id,
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
                    component_id,
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
                        *component_id,
                        &input.lazy_map_id,
                        &input.key,
                    );
                    Ok((
                        old_value,
                        Committed {
                            component_id: *component_id,
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
            Committed { component_id } => {
                self.track.put_lazy_map_entry(
                    component_id,
                    input.lazy_map_id,
                    input.key,
                    input.value,
                );
                self.track
                    .insert_objects_into_component(new_objects, component_id);
            }
        }

        Ok(PutLazyMapEntryOutput {})
    }

    fn allocate_resource(
        &mut self,
        resource_def_id: ResourceDefId,
        mint_params: MintParams,
    ) -> Result<ResourceContainer, RuntimeError> {
        let resource_def = self
            .track
            .get_resource_def_mut(&resource_def_id)
            .ok_or(RuntimeError::ResourceDefNotFound(resource_def_id))?;
        match mint_params {
            MintParams::Fungible { amount } => {
                // Notify resource manager
                resource_def
                    .mint(&Amount::Fungible { amount })
                    .map_err(RuntimeError::ResourceDefError)?;

                // Allocate fungible
                Ok(ResourceContainer::new_fungible(
                    resource_def_id,
                    resource_def.resource_type().divisibility(),
                    amount,
                ))
            }
            MintParams::NonFungible { entries } => {
                // Notify resource manager
                resource_def
                    .mint(&Amount::NonFungible {
                        ids: entries.keys().cloned().collect(),
                    })
                    .map_err(RuntimeError::ResourceDefError)?;

                // Allocate non-fungibles
                let mut ids = BTreeSet::new();
                for (id, data) in entries {
                    let non_fungible_address = NonFungibleAddress::new(resource_def_id, id.clone());
                    if self.track.get_non_fungible(&non_fungible_address).is_some() {
                        return Err(RuntimeError::NonFungibleAlreadyExists(non_fungible_address));
                    }

                    let immutable_data = self.process_non_fungible_data(&data.0)?;
                    let mutable_data = self.process_non_fungible_data(&data.1)?;

                    self.track.put_non_fungible(
                        non_fungible_address,
                        NonFungible::new(immutable_data.raw, mutable_data.raw),
                    );
                    ids.insert(id);
                }

                Ok(ResourceContainer::new_non_fungible(resource_def_id, ids))
            }
        }
    }

    fn handle_create_resource(
        &mut self,
        input: CreateResourceInput,
    ) -> Result<CreateResourceOutput, RuntimeError> {
        let resource_def = ResourceDef::new(
            input.resource_type,
            input.metadata,
            input.flags,
            input.mutable_flags,
            input.authorities,
        )
        .map_err(RuntimeError::ResourceDefError)?;

        let resource_def_id = self.track.create_resource_def(resource_def);
        re_debug!(self, "New resource definition: {}", resource_def_id);

        let bucket_id = if let Some(mint_params) = input.mint_params {
            let bucket = Bucket::new(self.allocate_resource(resource_def_id, mint_params)?);
            let bucket_id = self.track.new_bucket_id();
            self.buckets.insert(bucket_id, bucket);
            Some(bucket_id)
        } else {
            None
        };

        Ok(CreateResourceOutput {
            resource_def_id,
            bucket_id,
        })
    }

    fn handle_get_resource_metadata(
        &mut self,
        input: GetResourceMetadataInput,
    ) -> Result<GetResourceMetadataOutput, RuntimeError> {
        let resource_def = self
            .track
            .get_resource_def(&input.resource_def_id)
            .ok_or(RuntimeError::ResourceDefNotFound(input.resource_def_id))?;

        Ok(GetResourceMetadataOutput {
            metadata: resource_def.metadata().clone(),
        })
    }

    fn handle_get_resource_total_supply(
        &mut self,
        input: GetResourceTotalSupplyInput,
    ) -> Result<GetResourceTotalSupplyOutput, RuntimeError> {
        let resource_def = self
            .track
            .get_resource_def(&input.resource_def_id)
            .ok_or(RuntimeError::ResourceDefNotFound(input.resource_def_id))?;

        Ok(GetResourceTotalSupplyOutput {
            total_supply: resource_def.total_supply(),
        })
    }

    fn handle_get_resource_flags(
        &mut self,
        input: GetResourceFlagsInput,
    ) -> Result<GetResourceFlagsOutput, RuntimeError> {
        let resource_def = self
            .track
            .get_resource_def(&input.resource_def_id)
            .ok_or(RuntimeError::ResourceDefNotFound(input.resource_def_id))?;

        Ok(GetResourceFlagsOutput {
            flags: resource_def.flags(),
        })
    }

    fn handle_get_resource_mutable_flags(
        &mut self,
        input: GetResourceMutableFlagsInput,
    ) -> Result<GetResourceMutableFlagsOutput, RuntimeError> {
        let resource_def = self
            .track
            .get_resource_def(&input.resource_def_id)
            .ok_or(RuntimeError::ResourceDefNotFound(input.resource_def_id))?;

        Ok(GetResourceMutableFlagsOutput {
            mutable_flags: resource_def.mutable_flags(),
        })
    }

    fn handle_get_resource_type(
        &mut self,
        input: GetResourceTypeInput,
    ) -> Result<GetResourceTypeOutput, RuntimeError> {
        let resource_def = self
            .track
            .get_resource_def(&input.resource_def_id)
            .ok_or(RuntimeError::ResourceDefNotFound(input.resource_def_id))?;

        Ok(GetResourceTypeOutput {
            resource_type: resource_def.resource_type(),
        })
    }

    fn handle_get_non_fungible_data(
        &mut self,
        input: GetNonFungibleDataInput,
    ) -> Result<GetNonFungibleDataOutput, RuntimeError> {
        let non_fungible = self
            .track
            .get_non_fungible(&input.non_fungible_address)
            .ok_or(RuntimeError::NonFungibleNotFound(
                input.non_fungible_address,
            ))?;

        Ok(GetNonFungibleDataOutput {
            immutable_data: non_fungible.immutable_data(),
            mutable_data: non_fungible.mutable_data(),
        })
    }

    fn handle_non_fungible_exists(
        &mut self,
        input: NonFungibleExistsInput,
    ) -> Result<NonFungibleExistsOutput, RuntimeError> {
        let non_fungible = self.track.get_non_fungible(&input.non_fungible_address);

        Ok(NonFungibleExistsOutput {
            non_fungible_exists: non_fungible.is_some(),
        })
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
            .get_resource_def(&input.resource_def_id)
            .ok_or(RuntimeError::ResourceDefNotFound(input.resource_def_id))?;

        let new_vault = Vault::new(ResourceContainer::new_empty(
            input.resource_def_id,
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
                    component_id,
                    initial_loaded_object_refs,
                    additional_object_refs,
                    ..
                } => {
                    if !initial_loaded_object_refs.vault_ids.contains(vault_id)
                        && !additional_object_refs.vault_ids.contains(vault_id)
                    {
                        return Err(RuntimeError::VaultNotFound(*vault_id));
                    }
                    let vault = self.track.get_vault_mut(component_id, vault_id);
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
            .map_err(RuntimeError::VaultError)?;

        Ok(PutIntoVaultOutput {})
    }

    fn check_resource_auth(
        &mut self,
        resource_def_id: &ResourceDefId,
        transition: ResourceControllerMethod,
    ) -> Result<(), RuntimeError> {
        let resource_def = self
            .track
            .get_resource_def(&resource_def_id)
            .ok_or(RuntimeError::ResourceDefNotFound(resource_def_id.clone()))?;
        resource_def
            .check_auth(
                transition,
                vec![self.caller_auth_worktop, &self.auth_worktop],
            )
            .map_err(RuntimeError::ResourceDefError)
    }

    fn handle_update_resource_flags(
        &mut self,
        input: UpdateResourceFlagsInput,
    ) -> Result<UpdateResourceFlagsOutput, RuntimeError> {
        // Auth
        self.check_resource_auth(
            &input.resource_def_id,
            ResourceControllerMethod::UpdateFlags,
        )?;

        // State Update
        let resource_def = self
            .track
            .get_resource_def_mut(&input.resource_def_id)
            .ok_or(RuntimeError::ResourceDefNotFound(input.resource_def_id))?;
        resource_def
            .update_flags(input.new_flags)
            .map_err(RuntimeError::ResourceDefError)?;

        Ok(UpdateResourceFlagsOutput {})
    }

    fn handle_update_resource_mutable_flags(
        &mut self,
        input: UpdateResourceMutableFlagsInput,
    ) -> Result<UpdateResourceMutableFlagsOutput, RuntimeError> {
        // Auth
        self.check_resource_auth(
            &input.resource_def_id,
            ResourceControllerMethod::UpdateMutableFlags,
        )?;

        // State Update
        let resource_def = self
            .track
            .get_resource_def_mut(&input.resource_def_id)
            .ok_or(RuntimeError::ResourceDefNotFound(input.resource_def_id))?;
        resource_def
            .update_mutable_flags(input.new_mutable_flags)
            .map_err(RuntimeError::ResourceDefError)?;

        Ok(UpdateResourceMutableFlagsOutput {})
    }

    fn handle_update_resource_metadata(
        &mut self,
        input: UpdateResourceMetadataInput,
    ) -> Result<UpdateResourceMetadataOutput, RuntimeError> {
        // Auth
        self.check_resource_auth(
            &input.resource_def_id,
            ResourceControllerMethod::UpdateMetadata,
        )?;

        // State update
        let resource_def = self
            .track
            .get_resource_def_mut(&input.resource_def_id)
            .ok_or(RuntimeError::ResourceDefNotFound(input.resource_def_id))?;
        resource_def
            .update_metadata(input.new_metadata)
            .map_err(RuntimeError::ResourceDefError)?;

        Ok(UpdateResourceMetadataOutput {})
    }

    fn handle_update_non_fungible_mutable_data(
        &mut self,
        input: UpdateNonFungibleMutableDataInput,
    ) -> Result<UpdateNonFungibleMutableDataOutput, RuntimeError> {
        // Auth
        let resource_def_id = input.non_fungible_address.resource_def_id();
        self.check_resource_auth(
            &resource_def_id,
            ResourceControllerMethod::UpdateNonFungibleMutableData,
        )?;

        // update state
        let data = self.process_non_fungible_data(&input.new_mutable_data)?;
        self.track
            .get_non_fungible_mut(&input.non_fungible_address)
            .ok_or(RuntimeError::NonFungibleNotFound(
                input.non_fungible_address,
            ))?
            .set_mutable_data(data.raw);

        Ok(UpdateNonFungibleMutableDataOutput {})
    }

    fn handle_mint_resource(
        &mut self,
        input: MintResourceInput,
    ) -> Result<MintResourceOutput, RuntimeError> {
        // Auth
        self.check_resource_auth(&input.resource_def_id, ResourceControllerMethod::Mint)?;

        // wrap resource into a bucket
        let bucket = Bucket::new(self.allocate_resource(input.resource_def_id, input.mint_params)?);
        let bucket_id = self.track.new_bucket_id();
        self.buckets.insert(bucket_id, bucket);

        Ok(MintResourceOutput { bucket_id })
    }

    fn handle_burn_resource(
        &mut self,
        input: BurnResourceInput,
    ) -> Result<BurnResourceOutput, RuntimeError> {
        // Auth
        let bucket = self
            .buckets
            .remove(&input.bucket_id)
            .ok_or(RuntimeError::BucketNotFound(input.bucket_id))?;
        self.check_resource_auth(&bucket.resource_def_id(), ResourceControllerMethod::Burn)?;

        // Burn
        let resource_def = self
            .track
            .get_resource_def_mut(&bucket.resource_def_id())
            .ok_or(RuntimeError::ResourceDefNotFound(bucket.resource_def_id()))?;

        resource_def
            .burn(&bucket.liquid_amount())
            .map_err(RuntimeError::ResourceDefError)?;
        Ok(BurnResourceOutput {})
    }

    fn handle_take_from_vault(
        &mut self,
        input: TakeFromVaultInput,
    ) -> Result<TakeFromVaultOutput, RuntimeError> {
        let resource_def_id = self.get_local_vault(&input.vault_id)?.resource_def_id();
        self.check_resource_auth(&resource_def_id, ResourceControllerMethod::TakeFromVault)?;

        let new_bucket = self
            .get_local_vault(&input.vault_id)?
            .take(input.amount)
            .map_err(RuntimeError::VaultError)?;

        let bucket_id = self.track.new_bucket_id();
        self.buckets.insert(bucket_id, new_bucket);

        Ok(TakeFromVaultOutput { bucket_id })
    }

    fn handle_take_non_fungible_from_vault(
        &mut self,
        input: TakeNonFungibleFromVaultInput,
    ) -> Result<TakeNonFungibleFromVaultOutput, RuntimeError> {
        let resource_def_id = self.get_local_vault(&input.vault_id)?.resource_def_id();
        self.check_resource_auth(&resource_def_id, ResourceControllerMethod::TakeFromVault)?;

        let new_bucket = self
            .get_local_vault(&input.vault_id)?
            .take_non_fungible(&input.non_fungible_id)
            .map_err(RuntimeError::VaultError)?;

        let bucket_id = self.track.new_bucket_id();
        self.buckets.insert(bucket_id, new_bucket);

        Ok(TakeNonFungibleFromVaultOutput { bucket_id })
    }

    fn handle_get_non_fungible_ids_in_vault(
        &mut self,
        input: GetNonFungibleIdsInVaultInput,
    ) -> Result<GetNonFungibleIdsInVaultOutput, RuntimeError> {
        let vault = self.get_local_vault(&input.vault_id)?;
        let non_fungible_ids = vault
            .liquid_amount()
            .as_non_fungible_ids()
            .map_err(RuntimeError::AmountError)?
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
            amount: vault.liquid_amount().as_quantity(),
        })
    }

    fn handle_get_vault_resource_def_id(
        &mut self,
        input: GetVaultResourceDefIdInput,
    ) -> Result<GetVaultResourceDefIdOutput, RuntimeError> {
        let vault = self.get_local_vault(&input.vault_id)?;

        Ok(GetVaultResourceDefIdOutput {
            resource_def_id: vault.resource_def_id(),
        })
    }

    fn handle_create_bucket(
        &mut self,
        input: CreateEmptyBucketInput,
    ) -> Result<CreateEmptyBucketOutput, RuntimeError> {
        let definition = self
            .track
            .get_resource_def(&input.resource_def_id)
            .ok_or(RuntimeError::ResourceDefNotFound(input.resource_def_id))?;

        let new_bucket = Bucket::new(ResourceContainer::new_empty(
            input.resource_def_id,
            definition.resource_type(),
        ));
        let bucket_id = self.track.new_bucket_id();
        self.buckets.insert(bucket_id, new_bucket);

        Ok(CreateEmptyBucketOutput { bucket_id })
    }

    fn handle_put_into_bucket(
        &mut self,
        input: PutIntoBucketInput,
    ) -> Result<PutIntoBucketOutput, RuntimeError> {
        let other = self
            .buckets
            .remove(&input.other)
            .ok_or(RuntimeError::BucketNotFound(input.other))?;

        self.buckets
            .get_mut(&input.bucket_id)
            .ok_or(RuntimeError::BucketNotFound(input.bucket_id))?
            .put(other)
            .map_err(RuntimeError::BucketError)?;

        Ok(PutIntoBucketOutput {})
    }

    fn handle_take_from_bucket(
        &mut self,
        input: TakeFromBucketInput,
    ) -> Result<TakeFromBucketOutput, RuntimeError> {
        let new_bucket = self
            .buckets
            .get_mut(&input.bucket_id)
            .ok_or(RuntimeError::BucketNotFound(input.bucket_id))?
            .take(input.amount)
            .map_err(RuntimeError::BucketError)?;
        let bucket_id = self.track.new_bucket_id();
        self.buckets.insert(bucket_id, new_bucket);

        Ok(TakeFromBucketOutput { bucket_id })
    }

    fn handle_get_bucket_amount(
        &mut self,
        input: GetBucketAmountInput,
    ) -> Result<GetBucketAmountOutput, RuntimeError> {
        let amount = self
            .buckets
            .get(&input.bucket_id)
            .map(|b| b.liquid_amount().as_quantity())
            .ok_or(RuntimeError::BucketNotFound(input.bucket_id))?;

        Ok(GetBucketAmountOutput { amount })
    }

    fn handle_get_bucket_resource_def_id(
        &mut self,
        input: GetBucketResourceDefIdInput,
    ) -> Result<GetBucketResourceDefIdOutput, RuntimeError> {
        let resource_def_id = self
            .buckets
            .get(&input.bucket_id)
            .map(|b| b.resource_def_id())
            .ok_or(RuntimeError::BucketNotFound(input.bucket_id))?;

        Ok(GetBucketResourceDefIdOutput { resource_def_id })
    }

    fn handle_take_non_fungible_from_bucket(
        &mut self,
        input: TakeNonFungibleFromBucketInput,
    ) -> Result<TakeNonFungibleFromBucketOutput, RuntimeError> {
        let new_bucket = self
            .buckets
            .get_mut(&input.bucket_id)
            .ok_or(RuntimeError::BucketNotFound(input.bucket_id))?
            .take_non_fungible(&input.non_fungible_id)
            .map_err(RuntimeError::BucketError)?;
        let bucket_id = self.track.new_bucket_id();
        self.buckets.insert(bucket_id, new_bucket);

        Ok(TakeNonFungibleFromBucketOutput { bucket_id })
    }

    fn handle_get_non_fungible_ids_in_bucket(
        &mut self,
        input: GetNonFungibleIdsInBucketInput,
    ) -> Result<GetNonFungibleIdsInBucketOutput, RuntimeError> {
        let bucket = self
            .buckets
            .get(&input.bucket_id)
            .ok_or(RuntimeError::BucketNotFound(input.bucket_id))?;

        Ok(GetNonFungibleIdsInBucketOutput {
            non_fungible_ids: bucket
                .liquid_amount()
                .as_non_fungible_ids()
                .map_err(RuntimeError::AmountError)?
                .into_iter()
                .collect(),
        })
    }

    fn handle_create_bucket_proof(
        &mut self,
        input: CreateBucketProofInput,
    ) -> Result<CreateBucketProofOutput, RuntimeError> {
        Ok(CreateBucketProofOutput {
            proof_id: self.create_bucket_proof(input.bucket_id)?,
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
            amount: proof.total_amount().as_quantity(),
        })
    }

    fn handle_get_proof_resource_def_id(
        &mut self,
        input: GetProofResourceDefIdInput,
    ) -> Result<GetProofResourceDefIdOutput, RuntimeError> {
        let proof = self
            .proofs
            .get(&input.proof_id)
            .ok_or(RuntimeError::ProofNotFound(input.proof_id))?;

        Ok(GetProofResourceDefIdOutput {
            resource_def_id: proof.resource_def_id(),
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
                .total_amount()
                .as_non_fungible_ids()
                .map_err(RuntimeError::AmountError)?
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

    fn handle_push_onto_auth_worktop(
        &mut self,
        input: PushOntoAuthWorktopInput,
    ) -> Result<PushOntoAuthWorktopOutput, RuntimeError> {
        self.push_onto_auth_worktop(input.proof_id)
            .map(|_| PushOntoAuthWorktopOutput {})
    }

    fn handle_pop_from_auth_worktop(
        &mut self,
        _input: PopFromAuthWorktopInput,
    ) -> Result<PopFromAuthWorktopOutput, RuntimeError> {
        self.pop_from_auth_worktop()
            .map(|proof_id| PopFromAuthWorktopOutput { proof_id })
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
            function: wasm_process.vm.invocation.function.clone(),
            args: wasm_process
                .vm
                .invocation
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
        Ok(GetActorOutput {
            actor: wasm_process.vm.invocation.actor.clone(),
        })
    }

    //============================
    // SYSTEM CALL HANDLERS END
    //============================
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
                    CALL_FUNCTION => self.handle(args, Self::handle_call_function),
                    CALL_METHOD => self.handle(args, Self::handle_call_method),

                    CREATE_COMPONENT => self.handle(args, Self::handle_create_component),
                    GET_COMPONENT_INFO => self.handle(args, Self::handle_get_component_info),
                    GET_COMPONENT_STATE => self.handle(args, Self::handle_get_component_state),
                    PUT_COMPONENT_STATE => self.handle(args, Self::handle_put_component_state),

                    CREATE_LAZY_MAP => self.handle(args, Self::handle_create_lazy_map),
                    GET_LAZY_MAP_ENTRY => self.handle(args, Self::handle_get_lazy_map_entry),
                    PUT_LAZY_MAP_ENTRY => self.handle(args, Self::handle_put_lazy_map_entry),

                    CREATE_RESOURCE => self.handle(args, Self::handle_create_resource),
                    GET_RESOURCE_TYPE => self.handle(args, Self::handle_get_resource_type),
                    GET_RESOURCE_METADATA => self.handle(args, Self::handle_get_resource_metadata),
                    GET_RESOURCE_TOTAL_SUPPLY => {
                        self.handle(args, Self::handle_get_resource_total_supply)
                    }
                    GET_RESOURCE_FLAGS => self.handle(args, Self::handle_get_resource_flags),
                    UPDATE_RESOURCE_FLAGS => self.handle(args, Self::handle_update_resource_flags),
                    GET_RESOURCE_MUTABLE_FLAGS => {
                        self.handle(args, Self::handle_get_resource_mutable_flags)
                    }
                    UPDATE_RESOURCE_MUTABLE_FLAGS => {
                        self.handle(args, Self::handle_update_resource_mutable_flags)
                    }
                    MINT_RESOURCE => self.handle(args, Self::handle_mint_resource),
                    BURN_RESOURCE => self.handle(args, Self::handle_burn_resource),
                    UPDATE_NON_FUNGIBLE_MUTABLE_DATA => {
                        self.handle(args, Self::handle_update_non_fungible_mutable_data)
                    }
                    GET_NON_FUNGIBLE_DATA => self.handle(args, Self::handle_get_non_fungible_data),
                    NON_FUNGIBLE_EXISTS => self.handle(args, Self::handle_non_fungible_exists),
                    UPDATE_RESOURCE_METADATA => {
                        self.handle(args, Self::handle_update_resource_metadata)
                    }

                    CREATE_EMPTY_VAULT => self.handle(args, Self::handle_create_vault),
                    PUT_INTO_VAULT => self.handle(args, Self::handle_put_into_vault),
                    TAKE_FROM_VAULT => self.handle(args, Self::handle_take_from_vault),
                    GET_VAULT_AMOUNT => self.handle(args, Self::handle_get_vault_amount),
                    GET_VAULT_RESOURCE_DEF_ID => {
                        self.handle(args, Self::handle_get_vault_resource_def_id)
                    }
                    TAKE_NON_FUNGIBLE_FROM_VAULT => {
                        self.handle(args, Self::handle_take_non_fungible_from_vault)
                    }
                    GET_NON_FUNGIBLE_IDS_IN_VAULT => {
                        self.handle(args, Self::handle_get_non_fungible_ids_in_vault)
                    }

                    CREATE_EMPTY_BUCKET => self.handle(args, Self::handle_create_bucket),
                    PUT_INTO_BUCKET => self.handle(args, Self::handle_put_into_bucket),
                    TAKE_FROM_BUCKET => self.handle(args, Self::handle_take_from_bucket),
                    GET_BUCKET_AMOUNT => self.handle(args, Self::handle_get_bucket_amount),
                    GET_BUCKET_RESOURCE_DEF_ID => {
                        self.handle(args, Self::handle_get_bucket_resource_def_id)
                    }
                    TAKE_NON_FUNGIBLE_FROM_BUCKET => {
                        self.handle(args, Self::handle_take_non_fungible_from_bucket)
                    }
                    GET_NON_FUNGIBLE_IDS_IN_BUCKET => {
                        self.handle(args, Self::handle_get_non_fungible_ids_in_bucket)
                    }

                    CREATE_BUCKET_PROOF => self.handle(args, Self::handle_create_bucket_proof),
                    DROP_PROOF => self.handle(args, Self::handle_drop_proof),
                    GET_PROOF_AMOUNT => self.handle(args, Self::handle_get_proof_amount),
                    GET_PROOF_RESOURCE_DEF_ID => {
                        self.handle(args, Self::handle_get_proof_resource_def_id)
                    }
                    GET_NON_FUNGIBLE_IDS_IN_PROOF => {
                        self.handle(args, Self::handle_get_non_fungible_ids_in_proof)
                    }
                    CLONE_PROOF => self.handle(args, Self::handle_clone_proof),
                    PUSH_ONTO_AUTH_WORKTOP => {
                        self.handle(args, Self::handle_push_onto_auth_worktop)
                    }
                    POP_FROM_AUTH_WORKTOP => self.handle(args, Self::handle_pop_from_auth_worktop),

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
