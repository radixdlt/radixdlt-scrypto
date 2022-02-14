use colored::*;
use sbor::*;
use scrypto::buffer::*;
use scrypto::engine::api::*;
use scrypto::engine::types::*;
use scrypto::rust::borrow::ToOwned;
use scrypto::rust::collections::*;
use scrypto::rust::fmt;
use scrypto::rust::format;
use scrypto::rust::rc::Rc;
use scrypto::rust::string::String;
use scrypto::rust::vec;
use scrypto::rust::vec::Vec;
use wasmi::*;

use crate::engine::process::LazyMapState::{Committed, Uncommitted};
use crate::engine::*;
use crate::errors::*;
use crate::ledger::*;
use crate::model::*;
use crate::transaction::*;

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

enum LazyMapState {
    Uncommitted { root: LazyMapId },
    Committed { component_ref: ComponentRef },
}

enum ComponentState {
    Empty,
    Loaded {
        component_ref: ComponentRef,
        component_data: ValidatedData,
    },
    Saved,
}

#[derive(Debug)]
struct UnclaimedLazyMap {
    lazy_map: LazyMap,
    /// All descendents (not just direct children) of the unclaimed lazy map
    descendent_lazy_maps: HashMap<LazyMapId, LazyMap>,
    descendent_vaults: HashMap<VaultId, Vault>,
}

impl UnclaimedLazyMap {
    fn merge(&mut self, unclaimed_lazy_map: UnclaimedLazyMap, lazy_map_id: LazyMapId) {
        self.descendent_lazy_maps
            .insert(lazy_map_id, unclaimed_lazy_map.lazy_map);

        for (lazy_map_id, lazy_map) in unclaimed_lazy_map.descendent_lazy_maps {
            self.descendent_lazy_maps.insert(lazy_map_id, lazy_map);
        }
        for (vault_id, vault) in unclaimed_lazy_map.descendent_vaults {
            self.descendent_vaults.insert(vault_id, vault);
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
    /// Buckets owned by this process (but LOCKED because there is a reference to it)
    buckets_locked: HashMap<BucketId, BucketRef>,
    /// Bucket references
    bucket_refs: HashMap<BucketRefId, BucketRef>,
    /// The buckets that will be moved to another process SHORTLY.
    moving_buckets: HashMap<BucketId, Bucket>,
    /// The bucket refs that will be moved to another process SHORTLY.
    moving_bucket_refs: HashMap<BucketRefId, BucketRef>,

    /// Vaults which haven't been assigned to a component or lazy map yet.
    unclaimed_vaults: HashMap<VaultId, Vault>,
    /// Lazy maps which haven't been assigned to a component or lazy map yet.
    /// Keeps track of vault and lazy map descendents.
    unclaimed_lazy_maps: HashMap<LazyMapId, UnclaimedLazyMap>,

    /// Components which have been loaded and possibly updated in the lifetime of this process.
    component_state: ComponentState,

    /// A WASM interpreter
    vm: Option<Interpreter>,
    /// ID allocator for buckets and bucket refs created within transaction.
    id_allocator: IdAllocator,
    /// Resources collected from previous CALLs returns.
    ///
    /// When the `depth == 0` (transaction), all returned resources from CALLs are coalesced
    /// into a map of unidentified buckets indexed by resource address, instead of moving back
    /// to `buckets`.
    ///
    /// Loop invariant: all buckets should be NON_EMPTY.
    worktop: HashMap<ResourceDefRef, Bucket>,
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
    package_ref: PackageRef,
    export_name: String,
    function: String,
    args: Vec<ValidatedData>,
}

impl<'r, 'l, L: SubstateStore> Process<'r, 'l, L> {
    /// Create a new process, which is not started.
    pub fn new(depth: usize, trace: bool, track: &'r mut Track<'l, L>) -> Self {
        Self {
            depth,
            trace,
            track,
            buckets: HashMap::new(),
            buckets_locked: HashMap::new(),
            bucket_refs: HashMap::new(),
            moving_buckets: HashMap::new(),
            moving_bucket_refs: HashMap::new(),
            unclaimed_vaults: HashMap::new(),
            unclaimed_lazy_maps: HashMap::new(),
            component_state: ComponentState::Empty,
            vm: None,
            id_allocator: IdAllocator::new(IdSpace::Transaction),
            worktop: HashMap::new(),
        }
    }

    // (Transaction ONLY) Takes resource from worktop and returns a bucket.
    pub fn take_from_worktop(
        &mut self,
        resource_spec: ResourceSpecification,
    ) -> Result<ValidatedData, RuntimeError> {
        re_debug!(
            self,
            "(Transaction) Taking from worktop: {:?}",
            resource_spec
        );
        let resource_def_ref = resource_spec.resource_def_ref();
        let new_bucket_id = self
            .id_allocator
            .new_bucket_id()
            .map_err(RuntimeError::IdAllocatorError)?;
        let bucket = match self.worktop.remove(&resource_def_ref) {
            Some(mut bucket) => {
                let to_return = match resource_spec {
                    ResourceSpecification::Fungible { amount, .. } => bucket.take(amount),
                    ResourceSpecification::NonFungible { keys, .. } => {
                        bucket.take_non_fungibles(&keys)
                    }
                    ResourceSpecification::All { .. } => bucket.take(bucket.amount()),
                }
                .map_err(RuntimeError::BucketError)?;

                if !bucket.amount().is_zero() {
                    self.worktop.insert(resource_def_ref, bucket);
                }
                Ok(to_return)
            }
            None => Err(RuntimeError::BucketError(BucketError::InsufficientBalance)),
        }?;
        self.buckets.insert(new_bucket_id, bucket);
        Ok(ValidatedData::from_slice(&scrypto_encode(&new_bucket_id)).unwrap())
    }

    // (Transaction ONLY) Returns resource back to worktop.
    pub fn return_to_worktop(
        &mut self,
        bucket_id: BucketId,
    ) -> Result<ValidatedData, RuntimeError> {
        re_debug!(
            self,
            "(Transaction) Returning to worktop: bucket_id = {:?}",
            bucket_id
        );

        let bucket = self
            .buckets
            .remove(&bucket_id)
            .ok_or(RuntimeError::BucketNotFound(bucket_id))?;

        if !bucket.amount().is_zero() {
            if let Some(existing_bucket) = self.worktop.get_mut(&bucket.resource_def_ref()) {
                existing_bucket
                    .put(bucket)
                    .map_err(RuntimeError::BucketError)?;
            } else {
                self.worktop.insert(bucket.resource_def_ref(), bucket);
            }
        }
        Ok(ValidatedData::from_slice(&scrypto_encode(&())).unwrap())
    }

    // (Transaction ONLY) Assert worktop contains at least this amount.
    pub fn assert_worktop_contains(
        &mut self,
        amount: Decimal,
        resource_def_ref: ResourceDefRef,
    ) -> Result<ValidatedData, RuntimeError> {
        re_debug!(
            self,
            "(Transaction) Asserting worktop contains: amount = {:?}, resource_def_ref = {:?}",
            amount,
            resource_def_ref
        );

        let balance = match self.worktop.get(&resource_def_ref) {
            Some(bucket) => bucket.amount(),
            None => Decimal::zero(),
        };

        if balance < amount {
            re_warn!(
                self,
                "(Transaction) Assertion failed: required = {:?}, actual = {:?}, resource_def_ref = {:?}",
                amount,
                balance,
                resource_def_ref
            );
            Err(RuntimeError::AssertionFailed)
        } else {
            Ok(ValidatedData::from_slice(&scrypto_encode(&())).unwrap())
        }
    }

    // (Transaction ONLY) Creates a bucket ref.
    pub fn create_bucket_ref(
        &mut self,
        bucket_id: BucketId,
    ) -> Result<ValidatedData, RuntimeError> {
        re_debug!(
            self,
            "(Transaction) Creating bucket ref: bucket_id = {:?}",
            bucket_id
        );

        let new_bucket_ref_id = self
            .id_allocator
            .new_bucket_ref_id()
            .map_err(RuntimeError::IdAllocatorError)?;
        match self.buckets_locked.get_mut(&bucket_id) {
            Some(bucket_rc) => {
                // re-borrow
                self.bucket_refs
                    .insert(new_bucket_ref_id, bucket_rc.clone());
            }
            None => {
                // first time borrow
                let bucket = BucketRef::new(LockedBucket::new(
                    bucket_id,
                    self.buckets
                        .remove(&bucket_id)
                        .ok_or(RuntimeError::BucketNotFound(bucket_id))?,
                ));
                self.buckets_locked.insert(bucket_id, bucket.clone());
                self.bucket_refs.insert(new_bucket_ref_id, bucket);
            }
        };

        Ok(ValidatedData::from_slice(&scrypto_encode(&new_bucket_ref_id)).unwrap())
    }

    // (Transaction ONLY) Clone a bucket ref.
    pub fn clone_bucket_ref(
        &mut self,
        bucket_ref_id: BucketRefId,
    ) -> Result<ValidatedData, RuntimeError> {
        re_debug!(
            self,
            "(Transaction) Cloning bucket ref: bucket_ref_id = {:?}",
            bucket_ref_id
        );

        let new_bucket_ref_id = self
            .id_allocator
            .new_bucket_ref_id()
            .map_err(RuntimeError::IdAllocatorError)?;
        let bucket_ref = self
            .bucket_refs
            .get(&bucket_ref_id)
            .ok_or(RuntimeError::BucketRefNotFound(bucket_ref_id))?
            .clone();
        self.bucket_refs.insert(new_bucket_ref_id, bucket_ref);

        Ok(ValidatedData::from_slice(&scrypto_encode(&new_bucket_ref_id)).unwrap())
    }

    // (Transaction ONLY) Drop a bucket ref.
    pub fn drop_bucket_ref(
        &mut self,
        bucket_ref_id: BucketRefId,
    ) -> Result<ValidatedData, RuntimeError> {
        re_debug!(
            self,
            "(Transaction) Dropping bucket ref: bucket_ref_id = {:?}",
            bucket_ref_id
        );

        self.handle_drop_bucket_ref(DropBucketRefInput { bucket_ref_id })?;

        Ok(ValidatedData::from_slice(&scrypto_encode(&())).unwrap())
    }

    /// (Transaction ONLY) Calls a method.
    pub fn call_method_with_all_resources(
        &mut self,
        component_ref: ComponentRef,
        method: &str,
    ) -> Result<ValidatedData, RuntimeError> {
        re_debug!(
            self,
            "(Transaction) Calling method with all resources started"
        );
        // 1. Move collected resource to temp buckets
        for (_, bucket) in self.worktop.clone() {
            let bucket_id = self.track.new_bucket_id(); // this is unbounded
            self.buckets.insert(bucket_id, bucket);
        }
        self.worktop.clear();

        // 2. Drop all bucket refs to unlock the buckets
        self.drop_all_bucket_refs()?;

        // 3. Call the method with all buckets
        let to_deposit: Vec<scrypto::resource::Bucket> = self
            .buckets
            .keys()
            .cloned()
            .map(|bucket_id| scrypto::resource::Bucket(bucket_id))
            .collect();
        let invocation = self.prepare_call_method(
            component_ref,
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

    /// (SYSTEM ONLY)  Creates a bucket ref which references a virtual bucket
    pub fn create_virtual_bucket_ref(
        &mut self,
        bucket_id: BucketId,
        bucket_ref_id: BucketRefId,
        bucket: Bucket,
    ) {
        let locked_bucket = LockedBucket::new(bucket_id, bucket);
        let bucket_ref = BucketRef::new(locked_bucket);
        self.bucket_refs.insert(bucket_ref_id, bucket_ref);
    }

    /// Moves buckets and bucket refs into this process.
    pub fn move_in_resources(
        &mut self,
        buckets: HashMap<BucketId, Bucket>,
        bucket_refs: HashMap<BucketRefId, BucketRef>,
    ) -> Result<(), RuntimeError> {
        if self.depth == 0 {
            assert!(bucket_refs.is_empty());

            for (_, bucket) in buckets {
                if !bucket.amount().is_zero() {
                    let address = bucket.resource_def_ref();
                    if let Some(b) = self.worktop.get_mut(&address) {
                        b.put(bucket).unwrap();
                    } else {
                        self.worktop.insert(address, bucket);
                    }
                }
            }
        } else {
            self.bucket_refs.extend(bucket_refs);
            self.buckets.extend(buckets);
        }

        Ok(())
    }

    /// Moves all marked buckets and bucket refs from this process.
    pub fn move_out_resources(
        &mut self,
    ) -> (HashMap<BucketId, Bucket>, HashMap<BucketRefId, BucketRef>) {
        let buckets = self.moving_buckets.drain().collect();
        let bucket_refs = self.moving_bucket_refs.drain().collect();
        (buckets, bucket_refs)
    }

    /// Runs the given export within this process.
    pub fn run(&mut self, invocation: Invocation) -> Result<ValidatedData, RuntimeError> {
        #[cfg(not(feature = "alloc"))]
        let now = std::time::Instant::now();
        re_info!(
            self,
            "Run started: package = {:?}, export = {:?}",
            invocation.package_ref,
            invocation.export_name
        );

        // Load the code
        let (module, memory) = self
            .track
            .load_module(invocation.package_ref)
            .ok_or(RuntimeError::PackageNotFound(invocation.package_ref))?;
        let vm = Interpreter {
            invocation: invocation.clone(),
            module: module.clone(),
            memory,
        };
        self.vm = Some(vm);

        // run the main function
        let result = module.invoke_export(invocation.export_name.as_str(), &[], self);
        re_debug!(self, "Invoke result: {:?}", result);
        let rtn = result
            .map_err(RuntimeError::InvokeError)?
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
        package_ref: PackageRef,
        blueprint_name: &str,
        function: &str,
        args: Vec<ValidatedData>,
    ) -> Result<Invocation, RuntimeError> {
        Ok(Invocation {
            actor: Actor::Blueprint(package_ref, blueprint_name.to_owned()),
            package_ref: package_ref,
            export_name: format!("{}_main", blueprint_name),
            function: function.to_owned(),
            args,
        })
    }

    /// Prepares a method call.
    pub fn prepare_call_method(
        &mut self,
        component_ref: ComponentRef,
        method: &str,
        args: Vec<ValidatedData>,
    ) -> Result<Invocation, RuntimeError> {
        let component = self
            .track
            .get_component(component_ref)
            .ok_or(RuntimeError::ComponentNotFound(component_ref))?
            .clone();
        let mut args_with_self =
            vec![ValidatedData::from_slice(&scrypto_encode(&component_ref)).unwrap()];
        args_with_self.extend(args);

        Ok(Invocation {
            actor: Actor::Component(component_ref),
            package_ref: component.package_ref(),
            export_name: format!("{}_main", component.blueprint_name()),
            function: method.to_owned(),
            args: args_with_self,
        })
    }

    /// Prepares an ABI call.
    pub fn prepare_call_abi(
        &mut self,
        package_ref: PackageRef,
        blueprint_name: &str,
    ) -> Result<Invocation, RuntimeError> {
        Ok(Invocation {
            actor: Actor::Blueprint(package_ref, blueprint_name.to_owned()),
            package_ref: package_ref,
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
        let (buckets_out, bucket_refs_out) = self.move_out_resources();
        let mut process = Process::new(self.depth + 1, self.trace, self.track);
        process.move_in_resources(buckets_out, bucket_refs_out)?;

        // run the function
        let result = process.run(invocation)?;
        process.drop_all_bucket_refs()?;
        process.check_resource()?;

        // move resource
        let (buckets_in, bucket_refs_in) = process.move_out_resources();
        self.move_in_resources(buckets_in, bucket_refs_in)?;

        // scan locked buckets for some might have been unlocked by child processes
        let bucket_ids: Vec<BucketId> = self
            .buckets_locked
            .values()
            .filter(|v| Rc::strong_count(v) == 1)
            .map(|v| v.bucket_id())
            .collect();
        for bucket_id in bucket_ids {
            re_debug!(self, "Changing bucket {:?} to unlocked state", bucket_id);
            let bucket_rc = self.buckets_locked.remove(&bucket_id).unwrap();
            let bucket = Rc::try_unwrap(bucket_rc).unwrap();
            self.buckets.insert(bucket_id, bucket.into());
        }

        Ok(result)
    }

    /// Calls a function.
    pub fn call_function(
        &mut self,
        package_ref: PackageRef,
        blueprint_name: &str,
        function: &str,
        args: Vec<ValidatedData>,
    ) -> Result<ValidatedData, RuntimeError> {
        re_debug!(self, "Call function started");
        let invocation = self.prepare_call_function(package_ref, blueprint_name, function, args)?;
        let result = self.call(invocation);
        re_debug!(self, "Call function ended");
        result
    }

    /// Calls a method.
    pub fn call_method(
        &mut self,
        component_ref: ComponentRef,
        method: &str,
        args: Vec<ValidatedData>,
    ) -> Result<ValidatedData, RuntimeError> {
        re_debug!(self, "Call method started");
        let invocation = self.prepare_call_method(component_ref, method, args)?;
        let result = self.call(invocation);
        re_debug!(self, "Call method ended");
        result
    }

    /// Calls the ABI generator of a blueprint.
    pub fn call_abi(
        &mut self,
        package_ref: PackageRef,
        blueprint_name: &str,
    ) -> Result<ValidatedData, RuntimeError> {
        re_debug!(self, "Call abi started");
        let invocation = self.prepare_call_abi(package_ref, blueprint_name)?;
        let result = self.call(invocation);
        re_debug!(self, "Call abi ended");
        result
    }

    /// Drops all bucket refs owned by this process.
    pub fn drop_all_bucket_refs(&mut self) -> Result<(), RuntimeError> {
        let bucket_ref_ids: Vec<BucketRefId> = self.bucket_refs.keys().cloned().collect();
        for bucket_ref_id in bucket_ref_ids {
            self.handle_drop_bucket_ref(DropBucketRefInput { bucket_ref_id })?;
        }
        Ok(())
    }

    /// Checks resource leak.
    pub fn check_resource(&self) -> Result<(), RuntimeError> {
        re_debug!(self, "Resource check started");
        let mut success = true;

        for (bucket_id, bucket) in &self.buckets {
            re_warn!(self, "Dangling bucket: {:?}, {:?}", bucket_id, bucket);
            success = false;
        }
        for (bucket_id, bucket) in &self.buckets_locked {
            re_warn!(self, "Dangling bucket: {:?}, {:?}", bucket_id, bucket);
            success = false;
        }
        for (_, bucket) in &self.worktop {
            re_warn!(self, "Dangling resource: {:?}", bucket);
            success = false;
        }
        for (vault_id, vault) in &self.unclaimed_vaults {
            re_warn!(self, "Dangling vault: {:?}, {:?}", vault_id, vault);
            success = false;
        }
        for (lazy_map_id, lazy_map) in &self.unclaimed_lazy_maps {
            re_warn!(self, "Dangling lazy map: {:?}, {:?}", lazy_map_id, lazy_map);
            success = false;
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

    /// Return the actor
    fn actor(&self) -> Result<Actor, RuntimeError> {
        self.vm
            .as_ref()
            .ok_or(RuntimeError::InterpreterNotStarted)
            .map(|vm| vm.invocation.actor.clone())
    }

    /// Return the function name
    fn function(&self) -> Result<String, RuntimeError> {
        self.vm
            .as_ref()
            .ok_or(RuntimeError::InterpreterNotStarted)
            .map(|vm| vm.invocation.function.clone())
    }

    /// Return the function name
    fn args(&self) -> Result<Vec<Vec<u8>>, RuntimeError> {
        self.vm
            .as_ref()
            .ok_or(RuntimeError::InterpreterNotStarted)
            .map(|vm| vm.invocation.args.iter().cloned().map(|v| v.raw).collect())
    }

    /// Return the module ref
    fn module(&self) -> Result<ModuleRef, RuntimeError> {
        self.vm
            .as_ref()
            .ok_or(RuntimeError::InterpreterNotStarted)
            .map(|vm| vm.module.clone())
    }

    /// Return the memory ref
    fn memory(&self) -> Result<MemoryRef, RuntimeError> {
        self.vm
            .as_ref()
            .ok_or(RuntimeError::InterpreterNotStarted)
            .map(|vm| vm.memory.clone())
    }

    fn process_call_data(
        &mut self,
        validated: &ValidatedData,
        is_argument: bool,
    ) -> Result<(), RuntimeError> {
        self.move_buckets(&validated.bucket_ids)?;
        if is_argument {
            self.move_bucket_refs(&validated.bucket_ref_ids)?;
        } else {
            if !validated.bucket_ref_ids.is_empty() {
                return Err(RuntimeError::BucketRefNotAllowed);
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

    fn process_component_data(data: &[u8]) -> Result<ValidatedData, RuntimeError> {
        let validated =
            ValidatedData::from_slice(data).map_err(RuntimeError::DataValidationError)?;
        if !validated.bucket_ids.is_empty() {
            return Err(RuntimeError::BucketNotAllowed);
        }
        if !validated.bucket_ref_ids.is_empty() {
            return Err(RuntimeError::BucketRefNotAllowed);
        }
        // lazy map allowed
        // vaults allowed
        Ok(validated)
    }

    fn process_map_data(data: &[u8]) -> Result<ValidatedData, RuntimeError> {
        let validated =
            ValidatedData::from_slice(data).map_err(RuntimeError::DataValidationError)?;
        if !validated.bucket_ids.is_empty() {
            return Err(RuntimeError::BucketNotAllowed);
        }
        if !validated.bucket_ref_ids.is_empty() {
            return Err(RuntimeError::BucketRefNotAllowed);
        }
        // lazy map allowed
        // vaults allowed
        Ok(validated)
    }

    fn process_non_fungible_data(&mut self, data: &[u8]) -> Result<ValidatedData, RuntimeError> {
        let validated =
            ValidatedData::from_slice(data).map_err(RuntimeError::DataValidationError)?;
        if !validated.bucket_ids.is_empty() {
            return Err(RuntimeError::BucketNotAllowed);
        }
        if !validated.bucket_ref_ids.is_empty() {
            return Err(RuntimeError::BucketRefNotAllowed);
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
            re_debug!(self, "Moving bucket: {:?}, {:?}", bucket_id, bucket);
            self.moving_buckets.insert(*bucket_id, bucket);
        }
        Ok(())
    }

    /// Remove transient buckets from this process
    fn move_bucket_refs(&mut self, bucket_refs: &[BucketRefId]) -> Result<(), RuntimeError> {
        for bucket_ref_id in bucket_refs {
            let bucket_ref = self
                .bucket_refs
                .remove(bucket_ref_id)
                .ok_or(RuntimeError::BucketRefNotFound(*bucket_ref_id))?;
            re_debug!(
                self,
                "Moving bucket ref: {:?}, {:?}",
                bucket_ref_id,
                bucket_ref
            );
            self.moving_bucket_refs.insert(*bucket_ref_id, bucket_ref);
        }
        Ok(())
    }

    /// Send a byte array to wasm instance.
    fn send_bytes(&mut self, bytes: &[u8]) -> Result<i32, RuntimeError> {
        let result = self.module()?.invoke_export(
            "scrypto_alloc",
            &[RuntimeValue::I32((bytes.len()) as i32)],
            &mut NopExternals,
        );

        if let Ok(Some(RuntimeValue::I32(ptr))) = result {
            if self.memory()?.set((ptr + 4) as u32, bytes).is_ok() {
                return Ok(ptr);
            }
        }

        Err(RuntimeError::MemoryAllocError)
    }

    /// Read a byte array from wasm instance.
    fn read_bytes(&mut self, ptr: i32) -> Result<Vec<u8>, RuntimeError> {
        // read length
        let a = self
            .memory()?
            .get(ptr as u32, 4)
            .map_err(RuntimeError::MemoryAccessError)?;
        let len = u32::from_le_bytes([a[0], a[1], a[2], a[3]]);

        // read data
        let data = self
            .memory()?
            .get((ptr + 4) as u32, len as usize)
            .map_err(RuntimeError::MemoryAccessError)?;

        // free the buffer
        self.module()?
            .invoke_export(
                "scrypto_free",
                &[RuntimeValue::I32(ptr as i32)],
                &mut NopExternals,
            )
            .map_err(RuntimeError::MemoryAccessError)?;

        Ok(data)
    }

    /// Handles a system call.
    fn handle<I: Decode + fmt::Debug, O: Encode + fmt::Debug>(
        &mut self,
        args: RuntimeArgs,
        handler: fn(&mut Self, input: I) -> Result<O, RuntimeError>,
    ) -> Result<Option<RuntimeValue>, Trap> {
        let op: u32 = args.nth_checked(0)?;
        let input_ptr: u32 = args.nth_checked(1)?;
        let input_len: u32 = args.nth_checked(2)?;
        let input_bytes = self
            .memory()?
            .get(input_ptr, input_len as usize)
            .map_err(|e| Trap::from(RuntimeError::MemoryAccessError(e)))?;
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

    fn check_badge(
        &mut self,
        optional_bucket_ref_id: Option<BucketRefId>,
    ) -> Result<Option<ResourceDefRef>, RuntimeError> {
        if let Some(bucket_ref_id) = optional_bucket_ref_id {
            // retrieve bucket reference
            let bucket_ref = self
                .bucket_refs
                .get(&bucket_ref_id)
                .ok_or(RuntimeError::BucketRefNotFound(bucket_ref_id))?;

            // read amount
            if bucket_ref.bucket().amount().is_zero() {
                return Err(RuntimeError::EmptyBucketRef);
            }
            let resource_def_ref = bucket_ref.bucket().resource_def_ref();

            // drop bucket reference after use
            self.handle_drop_bucket_ref(DropBucketRefInput { bucket_ref_id })?;

            Ok(Some(resource_def_ref))
        } else {
            Ok(None)
        }
    }

    //============================
    // SYSTEM CALL HANDLERS START
    //============================

    fn handle_publish(
        &mut self,
        input: PublishPackageInput,
    ) -> Result<PublishPackageOutput, RuntimeError> {
        let package_ref = self.track.new_package_ref();

        if self.track.get_package(package_ref).is_some() {
            return Err(RuntimeError::PackageAlreadyExists(package_ref));
        }
        validate_module(&input.code).map_err(RuntimeError::WasmValidationError)?;

        re_debug!(self, "New package: {:?}", package_ref);
        self.track
            .put_package(package_ref, Package::new(input.code));

        Ok(PublishPackageOutput { package_ref })
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
            "CALL started: package_ref = {:?}, blueprint_name = {}, function = {:?}, args = {:?}",
            input.package_ref,
            input.blueprint_name,
            input.function,
            validated_args
        );

        let invocation = self.prepare_call_function(
            input.package_ref,
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
            "CALL started: component = {:?}, method = {:?}, args = {:?}",
            input.component_ref,
            input.method,
            validated_args
        );

        let invocation =
            self.prepare_call_method(input.component_ref, input.method.as_str(), validated_args)?;
        let result = self.call(invocation);

        re_debug!(self, "CALL finished");
        Ok(CallMethodOutput { rtn: result?.raw })
    }

    fn move_lazy_map_into_component(
        &mut self,
        unclaimed_lazy_map: UnclaimedLazyMap,
        lazy_map_id: LazyMapId,
        component_ref: ComponentRef,
    ) {
        re_debug!(
            self,
            "Lazy Map move: lazy_map = {:?}, to = component({:?}) ",
            lazy_map_id,
            component_ref
        );

        self.track
            .put_lazy_map(component_ref, lazy_map_id, unclaimed_lazy_map.lazy_map);
        for (child_lazy_map_id, child_lazy_map) in unclaimed_lazy_map.descendent_lazy_maps {
            self.track
                .put_lazy_map(component_ref, child_lazy_map_id, child_lazy_map);
        }
        for (vault_id, vault) in unclaimed_lazy_map.descendent_vaults {
            self.track.put_vault(component_ref, vault_id, vault);
        }
    }

    fn handle_create_component(
        &mut self,
        input: CreateComponentInput,
    ) -> Result<CreateComponentOutput, RuntimeError> {
        let component_ref = self.track.new_component_ref();

        if self.track.get_component(component_ref).is_some() {
            return Err(RuntimeError::ComponentAlreadyExists(component_ref));
        }

        let data = Self::process_component_data(&input.state)?;
        re_debug!(
            self,
            "New component: address = {:?}, state = {:?}",
            component_ref,
            data
        );

        for vault_id in data.vault_ids {
            let vault = self
                .unclaimed_vaults
                .remove(&vault_id)
                .ok_or(RuntimeError::VaultNotFound(vault_id))?;
            self.track.put_vault(component_ref, vault_id, vault);
        }

        for lazy_map_id in data.lazy_map_ids {
            let unclaimed_lazy_map = self
                .unclaimed_lazy_maps
                .remove(&lazy_map_id)
                .ok_or(RuntimeError::LazyMapNotFound(lazy_map_id))?;
            self.move_lazy_map_into_component(unclaimed_lazy_map, lazy_map_id, component_ref);
        }

        let component = Component::new(input.package_ref, input.blueprint_name, data.raw);
        self.track.put_component(component_ref, component);

        Ok(CreateComponentOutput { component_ref })
    }

    fn handle_get_component_info(
        &mut self,
        input: GetComponentInfoInput,
    ) -> Result<GetComponentInfoOutput, RuntimeError> {
        let component = self
            .track
            .get_component(input.component_ref)
            .ok_or(RuntimeError::ComponentNotFound(input.component_ref))?;

        Ok(GetComponentInfoOutput {
            package_ref: component.package_ref(),
            blueprint_name: component.blueprint_name().to_owned(),
        })
    }

    fn handle_get_component_state(
        &mut self,
        _: GetComponentStateInput,
    ) -> Result<GetComponentStateOutput, RuntimeError> {
        let component_ref = match self.component_state {
            ComponentState::Empty => match self.vm.as_ref().unwrap().invocation.actor {
                Actor::Component(component_ref) => Ok(component_ref),
                _ => Err(RuntimeError::IllegalSystemCall()),
            },
            ComponentState::Loaded { component_ref, .. } => {
                Err(RuntimeError::ComponentAlreadyLoaded(component_ref))
            }
            ComponentState::Saved => Err(RuntimeError::IllegalSystemCall()),
        }?;

        let component = self.track.get_component(component_ref).unwrap();
        let state = component.state();
        let component_data = Self::process_component_data(state).unwrap();
        self.component_state = ComponentState::Loaded {
            component_ref,
            component_data,
        };

        Ok(GetComponentStateOutput {
            state: state.to_owned(),
        })
    }

    fn handle_put_component_state(
        &mut self,
        input: PutComponentStateInput,
    ) -> Result<PutComponentStateOutput, RuntimeError> {
        let old_state = match &self.component_state {
            ComponentState::Empty => Err(RuntimeError::ComponentNotLoaded()),
            ComponentState::Loaded { component_data, .. } => Ok(component_data),
            ComponentState::Saved => Err(RuntimeError::IllegalSystemCall()),
        }?
        .to_owned();
        let component_ref = match self.vm.as_ref().unwrap().invocation.actor {
            Actor::Component(component_ref) => Ok(component_ref),
            _ => Err(RuntimeError::IllegalSystemCall()),
        }
        .unwrap();

        let new_state = Self::process_component_data(&input.state)?;
        re_debug!(self, "New component state: {:?}", new_state);

        // Only allow vaults to be added, never removed
        let mut old_vaults: HashSet<VaultId> = HashSet::from_iter(old_state.vault_ids.into_iter());
        for vault_id in new_state.vault_ids {
            if !old_vaults.remove(&vault_id) {
                let vault = self
                    .unclaimed_vaults
                    .remove(&vault_id)
                    .ok_or(RuntimeError::VaultNotFound(vault_id))?;
                self.track.put_vault(component_ref, vault_id, vault);
            }
        }
        old_vaults
            .into_iter()
            .try_for_each(|vault_id| Err(RuntimeError::VaultRemoved(vault_id)))?;

        // Only allow lazy maps to be added, never removed
        let mut old_lazy_maps: HashSet<LazyMapId> =
            HashSet::from_iter(old_state.lazy_map_ids.into_iter());
        for lazy_map_id in new_state.lazy_map_ids {
            if !old_lazy_maps.remove(&lazy_map_id) {
                let unclaimed_lazy_map = self
                    .unclaimed_lazy_maps
                    .remove(&lazy_map_id)
                    .ok_or(RuntimeError::LazyMapNotFound(lazy_map_id))?;
                self.move_lazy_map_into_component(unclaimed_lazy_map, lazy_map_id, component_ref);
            }
        }
        old_lazy_maps
            .into_iter()
            .try_for_each(|lazy_map_id| Err(RuntimeError::LazyMapRemoved(lazy_map_id)))?;

        let component = self.track.get_component_mut(component_ref).unwrap();
        component.set_state(new_state.raw);

        self.component_state = ComponentState::Saved;

        Ok(PutComponentStateOutput {})
    }

    fn handle_create_lazy_map(
        &mut self,
        _input: CreateLazyMapInput,
    ) -> Result<CreateLazyMapOutput, RuntimeError> {
        let lazy_map_id = self.track.new_lazy_map_id();
        self.unclaimed_lazy_maps.insert(
            lazy_map_id,
            UnclaimedLazyMap {
                lazy_map: LazyMap::new(),
                descendent_lazy_maps: HashMap::new(),
                descendent_vaults: HashMap::new(),
            },
        );

        Ok(CreateLazyMapOutput { lazy_map_id })
    }

    fn handle_get_lazy_map_entry(
        &mut self,
        input: GetLazyMapEntryInput,
    ) -> Result<GetLazyMapEntryOutput, RuntimeError> {
        let (lazy_map, _) = self.get_local_lazy_map_mut(input.lazy_map_id)?;
        let value = lazy_map.get_entry(&input.key);

        Ok(GetLazyMapEntryOutput {
            value: value.map(|e| e.to_vec()),
        })
    }

    fn handle_put_lazy_map_entry(
        &mut self,
        input: PutLazyMapEntryInput,
    ) -> Result<PutLazyMapEntryOutput, RuntimeError> {
        let key = Self::process_map_data(&input.key)?;
        let new_entry_state = Self::process_map_data(&input.value)?;
        re_debug!(self, "Map entry: {} => {}", key, new_entry_state);

        let (lazy_map, lazy_map_state) = self.get_local_lazy_map_mut(input.lazy_map_id)?;
        let (mut old_entry_vault_ids, mut old_entry_lazy_map_ids) = lazy_map
            .get_entry(&key.raw)
            .map_or((HashSet::new(), HashSet::new()), |e| {
                let data = Self::process_map_data(e).unwrap();
                let old_vaults = HashSet::from_iter(data.vault_ids.into_iter());
                let old_lazy_maps = HashSet::from_iter(data.lazy_map_ids.into_iter());
                (old_vaults, old_lazy_maps)
            });
        lazy_map.set_entry(key.raw, new_entry_state.raw);

        // Only allow vaults to be added, never removed
        for vault_id in new_entry_state.vault_ids {
            if !old_entry_vault_ids.remove(&vault_id) {
                let vault = self
                    .unclaimed_vaults
                    .remove(&vault_id)
                    .ok_or(RuntimeError::VaultNotFound(vault_id))?;
                match lazy_map_state {
                    Uncommitted { root } => {
                        let unclaimed_lazy_map = self.unclaimed_lazy_maps.get_mut(&root).unwrap();
                        unclaimed_lazy_map.descendent_vaults.insert(vault_id, vault);
                    }
                    Committed { component_ref } => {
                        self.track.put_vault(component_ref, vault_id, vault);
                    }
                }
            }
        }
        old_entry_vault_ids
            .into_iter()
            .try_for_each(|vault_id| Err(RuntimeError::VaultRemoved(vault_id)))?;

        // Only allow lazy maps to be added, never removed
        for lazy_map_id in new_entry_state.lazy_map_ids {
            if !old_entry_lazy_map_ids.remove(&lazy_map_id) {
                let child_lazy_map = self
                    .unclaimed_lazy_maps
                    .remove(&lazy_map_id)
                    .ok_or(RuntimeError::LazyMapNotFound(lazy_map_id))?;

                match lazy_map_state {
                    Uncommitted { root } => {
                        let unclaimed_lazy_map = self.unclaimed_lazy_maps.get_mut(&root).unwrap();
                        unclaimed_lazy_map.merge(child_lazy_map, lazy_map_id);
                    }
                    Committed { component_ref } => {
                        self.move_lazy_map_into_component(
                            child_lazy_map,
                            lazy_map_id,
                            component_ref,
                        );
                    }
                }
            }
        }
        old_entry_lazy_map_ids
            .into_iter()
            .try_for_each(|lazy_map_id| Err(RuntimeError::LazyMapRemoved(lazy_map_id)))?;

        Ok(PutLazyMapEntryOutput {})
    }

    fn allocate_resource(
        &mut self,
        resource_def_ref: ResourceDefRef,
        new_supply: Supply,
    ) -> Result<Resource, RuntimeError> {
        match new_supply {
            Supply::Fungible { amount } => Ok(Resource::Fungible { amount }),
            Supply::NonFungible { entries } => {
                let mut keys = BTreeSet::new();

                for (key, data) in entries {
                    if self
                        .track
                        .get_non_fungible(resource_def_ref, &key)
                        .is_some()
                    {
                        return Err(RuntimeError::NonFungibleAlreadyExists(
                            resource_def_ref,
                            key.clone(),
                        ));
                    }

                    let immutable_data = self.process_non_fungible_data(&data.0)?;
                    let mutable_data = self.process_non_fungible_data(&data.1)?;

                    self.track.put_non_fungible(
                        resource_def_ref,
                        &key,
                        NonFungible::new(immutable_data.raw, mutable_data.raw),
                    );
                    keys.insert(key.clone());
                }

                Ok(Resource::NonFungible { keys })
            }
        }
    }

    fn handle_create_resource(
        &mut self,
        input: CreateResourceInput,
    ) -> Result<CreateResourceOutput, RuntimeError> {
        // instantiate resource definition
        let resource_def_ref = self.track.new_resource_def_ref();
        if self.track.get_resource_def(resource_def_ref).is_some() {
            return Err(RuntimeError::ResourceDefAlreadyExists(resource_def_ref));
        }
        re_debug!(self, "New resource definition: {:?}", resource_def_ref);
        let definition = ResourceDef::new(
            input.resource_type,
            input.metadata,
            input.flags,
            input.mutable_flags,
            input.authorities,
            &input.initial_supply,
        )
        .map_err(RuntimeError::ResourceDefError)?;
        self.track.put_resource_def(resource_def_ref, definition);

        // allocate supply
        let bucket_id = if let Some(initial_supply) = input.initial_supply {
            let supply = self.allocate_resource(resource_def_ref, initial_supply)?;

            let bucket = Bucket::new(resource_def_ref, input.resource_type, supply);
            let bucket_id = self.track.new_bucket_id();
            self.buckets.insert(bucket_id, bucket);
            Some(bucket_id)
        } else {
            None
        };

        Ok(CreateResourceOutput {
            resource_def_ref,
            bucket_id,
        })
    }

    fn handle_get_resource_metadata(
        &mut self,
        input: GetResourceMetadataInput,
    ) -> Result<GetResourceMetadataOutput, RuntimeError> {
        let resource_def = self
            .track
            .get_resource_def(input.resource_def_ref)
            .ok_or(RuntimeError::ResourceDefNotFound(input.resource_def_ref))?;

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
            .get_resource_def(input.resource_def_ref)
            .ok_or(RuntimeError::ResourceDefNotFound(input.resource_def_ref))?;

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
            .get_resource_def(input.resource_def_ref)
            .ok_or(RuntimeError::ResourceDefNotFound(input.resource_def_ref))?;

        Ok(GetResourceFlagsOutput {
            flags: resource_def.flags(),
        })
    }

    fn handle_update_resource_flags(
        &mut self,
        input: UpdateResourceFlagsInput,
    ) -> Result<UpdateResourceFlagsOutput, RuntimeError> {
        let badge = self.check_badge(Some(input.auth))?;

        let resource_def = self
            .track
            .get_resource_def_mut(input.resource_def_ref)
            .ok_or(RuntimeError::ResourceDefNotFound(input.resource_def_ref))?;
        resource_def
            .update_flags(input.new_flags, badge)
            .map_err(RuntimeError::ResourceDefError)?;

        Ok(UpdateResourceFlagsOutput {})
    }

    fn handle_get_resource_mutable_flags(
        &mut self,
        input: GetResourceMutableFlagsInput,
    ) -> Result<GetResourceMutableFlagsOutput, RuntimeError> {
        let resource_def = self
            .track
            .get_resource_def(input.resource_def_ref)
            .ok_or(RuntimeError::ResourceDefNotFound(input.resource_def_ref))?;

        Ok(GetResourceMutableFlagsOutput {
            mutable_flags: resource_def.mutable_flags(),
        })
    }

    fn handle_update_resource_mutable_flags(
        &mut self,
        input: UpdateResourceMutableFlagsInput,
    ) -> Result<UpdateResourceMutableFlagsOutput, RuntimeError> {
        let badge = self.check_badge(Some(input.auth))?;

        let resource_def = self
            .track
            .get_resource_def_mut(input.resource_def_ref)
            .ok_or(RuntimeError::ResourceDefNotFound(input.resource_def_ref))?;
        resource_def
            .update_mutable_flags(input.new_mutable_flags, badge)
            .map_err(RuntimeError::ResourceDefError)?;

        Ok(UpdateResourceMutableFlagsOutput {})
    }

    fn handle_get_resource_type(
        &mut self,
        input: GetResourceTypeInput,
    ) -> Result<GetResourceTypeOutput, RuntimeError> {
        let resource_def = self
            .track
            .get_resource_def(input.resource_def_ref)
            .ok_or(RuntimeError::ResourceDefNotFound(input.resource_def_ref))?;

        Ok(GetResourceTypeOutput {
            resource_type: resource_def.resource_type(),
        })
    }

    fn handle_mint_resource(
        &mut self,
        input: MintResourceInput,
    ) -> Result<MintResourceOutput, RuntimeError> {
        let badge = self.check_badge(Some(input.auth))?;

        // allocate resource
        let resource = self.allocate_resource(input.resource_def_ref, input.new_supply)?;

        // mint resource
        let resource_def = self
            .track
            .get_resource_def_mut(input.resource_def_ref)
            .ok_or(RuntimeError::ResourceDefNotFound(input.resource_def_ref))?;
        resource_def
            .mint(&resource, badge)
            .map_err(RuntimeError::ResourceDefError)?;

        // wrap resource into a bucket
        let bucket = Bucket::new(
            input.resource_def_ref,
            resource_def.resource_type(),
            resource,
        );
        let bucket_id = self.track.new_bucket_id();
        self.buckets.insert(bucket_id, bucket);

        Ok(MintResourceOutput { bucket_id })
    }

    fn handle_burn_resource(
        &mut self,
        input: BurnResourceInput,
    ) -> Result<BurnResourceOutput, RuntimeError> {
        let badge = self.check_badge(input.auth)?;

        let bucket = self
            .buckets
            .remove(&input.bucket_id)
            .ok_or(RuntimeError::BucketNotFound(input.bucket_id))?;

        let resource_def = self
            .track
            .get_resource_def_mut(bucket.resource_def_ref())
            .ok_or(RuntimeError::ResourceDefNotFound(bucket.resource_def_ref()))?;

        resource_def
            .burn(bucket.resource(), badge)
            .map_err(RuntimeError::ResourceDefError)?;
        Ok(BurnResourceOutput {})
    }

    fn handle_update_non_fungible_mutable_data(
        &mut self,
        input: UpdateNonFungibleMutableDataInput,
    ) -> Result<UpdateNonFungibleMutableDataOutput, RuntimeError> {
        let badge = self.check_badge(Some(input.auth))?;

        // obtain authorization from resource definition
        let resource_def = self
            .track
            .get_resource_def(input.resource_def_ref)
            .ok_or(RuntimeError::ResourceDefNotFound(input.resource_def_ref))?;
        resource_def
            .check_update_non_fungible_mutable_data_auth(badge)
            .map_err(RuntimeError::ResourceDefError)?;
        // update state
        let data = self.process_non_fungible_data(&input.new_mutable_data)?;
        self.track
            .get_non_fungible_mut(input.resource_def_ref, &input.key)
            .ok_or(RuntimeError::NonFungibleNotFound(
                input.resource_def_ref,
                input.key.clone(),
            ))?
            .set_mutable_data(data.raw);

        Ok(UpdateNonFungibleMutableDataOutput {})
    }

    fn handle_get_non_fungible_data(
        &mut self,
        input: GetNonFungibleDataInput,
    ) -> Result<GetNonFungibleDataOutput, RuntimeError> {
        let non_fungible = self
            .track
            .get_non_fungible(input.resource_def_ref, &input.key)
            .ok_or(RuntimeError::NonFungibleNotFound(
                input.resource_def_ref,
                input.key.clone(),
            ))?;

        Ok(GetNonFungibleDataOutput {
            immutable_data: non_fungible.immutable_data(),
            mutable_data: non_fungible.mutable_data(),
        })
    }

    fn handle_update_resource_metadata(
        &mut self,
        input: UpdateResourceMetadataInput,
    ) -> Result<UpdateResourceMetadataOutput, RuntimeError> {
        let badge = self.check_badge(Some(input.auth))?;

        let resource_def = self
            .track
            .get_resource_def_mut(input.resource_def_ref)
            .ok_or(RuntimeError::ResourceDefNotFound(input.resource_def_ref))?;
        resource_def
            .update_metadata(input.new_metadata, badge)
            .map_err(RuntimeError::ResourceDefError)?;

        Ok(UpdateResourceMetadataOutput {})
    }

    fn handle_create_vault(
        &mut self,
        input: CreateEmptyVaultInput,
    ) -> Result<CreateEmptyVaultOutput, RuntimeError> {
        let definition = self
            .track
            .get_resource_def(input.resource_def_ref)
            .ok_or(RuntimeError::ResourceDefNotFound(input.resource_def_ref))?;

        let new_vault = Vault::new(Bucket::new(
            input.resource_def_ref,
            definition.resource_type(),
            match definition.resource_type() {
                ResourceType::Fungible { .. } => Resource::Fungible {
                    amount: Decimal::zero(),
                },
                ResourceType::NonFungible { .. } => Resource::NonFungible {
                    keys: BTreeSet::new(),
                },
            },
        ));
        let vault_id = self.track.new_vault_id();
        self.unclaimed_vaults.insert(vault_id, new_vault);

        Ok(CreateEmptyVaultOutput { vault_id })
    }

    fn get_local_lazy_map_mut(
        &mut self,
        lazy_map_id: LazyMapId,
    ) -> Result<(&mut LazyMap, LazyMapState), RuntimeError> {
        // TODO: Optimize to prevent iteration
        for (root, unclaimed) in self.unclaimed_lazy_maps.iter_mut() {
            if lazy_map_id.eq(root) {
                return Ok((&mut unclaimed.lazy_map, Uncommitted { root: root.clone() }));
            }

            let lazy_map = unclaimed.descendent_lazy_maps.get_mut(&lazy_map_id);
            if lazy_map.is_some() {
                return Ok((lazy_map.unwrap(), Uncommitted { root: root.clone() }));
            }
        }

        match self.vm.as_ref().unwrap().invocation.actor {
            Actor::Component(component_ref) => {
                match self.track.get_lazy_map_mut(component_ref, lazy_map_id) {
                    Some(lazy_map) => Ok((
                        lazy_map,
                        Committed {
                            component_ref: component_ref,
                        },
                    )),
                    None => Err(RuntimeError::LazyMapNotFound(lazy_map_id)),
                }
            }
            _ => Err(RuntimeError::LazyMapNotFound(lazy_map_id)),
        }
    }

    fn get_local_vault(&mut self, vault_id: VaultId) -> Result<&mut Vault, RuntimeError> {
        match self.unclaimed_vaults.get_mut(&vault_id) {
            Some(vault) => Ok(vault),
            None => {
                // TODO: Optimize to prevent iteration
                for (_, unclaimed) in self.unclaimed_lazy_maps.iter_mut() {
                    let vault = unclaimed.descendent_vaults.get_mut(&vault_id);
                    if vault.is_some() {
                        return Ok(vault.unwrap());
                    }
                }

                match self.vm.as_ref().unwrap().invocation.actor {
                    Actor::Component(component_ref) => {
                        match self.track.get_vault_mut(component_ref, vault_id) {
                            Some(vault) => Ok(vault),
                            None => Err(RuntimeError::VaultNotFound(vault_id)),
                        }
                    }
                    _ => Err(RuntimeError::VaultNotFound(vault_id)),
                }
            }
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

        self.get_local_vault(input.vault_id)?
            .put(bucket)
            .map_err(RuntimeError::VaultError)?;

        Ok(PutIntoVaultOutput {})
    }

    fn check_take_from_vault_auth(
        &mut self,
        vault_id: VaultId,
        badge: Option<ResourceDefRef>,
    ) -> Result<(), RuntimeError> {
        let resource_def_ref = self.get_local_vault(vault_id)?.resource_def_ref();

        let resource_def = self
            .track
            .get_resource_def(resource_def_ref)
            .ok_or(RuntimeError::ResourceDefNotFound(resource_def_ref))?;
        resource_def
            .check_take_from_vault_auth(badge)
            .map_err(RuntimeError::ResourceDefError)
    }

    fn handle_take_from_vault(
        &mut self,
        input: TakeFromVaultInput,
    ) -> Result<TakeFromVaultOutput, RuntimeError> {
        // TODO: restrict access

        let badge = self.check_badge(input.auth)?;
        self.check_take_from_vault_auth(input.vault_id.clone(), badge)?;

        let new_bucket = self
            .get_local_vault(input.vault_id)?
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
        // TODO: restrict access

        let badge = self.check_badge(input.auth)?;
        self.check_take_from_vault_auth(input.vault_id.clone(), badge)?;

        let new_bucket = self
            .get_local_vault(input.vault_id)?
            .take_non_fungible(&input.key)
            .map_err(RuntimeError::VaultError)?;

        let bucket_id = self.track.new_bucket_id();
        self.buckets.insert(bucket_id, new_bucket);

        Ok(TakeNonFungibleFromVaultOutput { bucket_id })
    }

    fn handle_get_non_fungible_keys_in_vault(
        &mut self,
        input: GetNonFungibleKeysInVaultInput,
    ) -> Result<GetNonFungibleKeysInVaultOutput, RuntimeError> {
        let vault = self.get_local_vault(input.vault_id)?;
        let keys = vault
            .get_non_fungible_ids()
            .map_err(RuntimeError::VaultError)?;

        Ok(GetNonFungibleKeysInVaultOutput { keys })
    }

    fn handle_get_vault_amount(
        &mut self,
        input: GetVaultDecimalInput,
    ) -> Result<GetVaultDecimalOutput, RuntimeError> {
        let vault = self.get_local_vault(input.vault_id)?;

        Ok(GetVaultDecimalOutput {
            amount: vault.amount(),
        })
    }

    fn handle_get_vault_resource_def(
        &mut self,
        input: GetVaultResourceDefInput,
    ) -> Result<GetVaultResourceDefOutput, RuntimeError> {
        let vault = self.get_local_vault(input.vault_id)?;

        Ok(GetVaultResourceDefOutput {
            resource_def_ref: vault.resource_def_ref(),
        })
    }

    fn handle_create_bucket(
        &mut self,
        input: CreateEmptyBucketInput,
    ) -> Result<CreateEmptyBucketOutput, RuntimeError> {
        let definition = self
            .track
            .get_resource_def(input.resource_def_ref)
            .ok_or(RuntimeError::ResourceDefNotFound(input.resource_def_ref))?;

        let new_bucket = Bucket::new(
            input.resource_def_ref,
            definition.resource_type(),
            match definition.resource_type() {
                ResourceType::Fungible { .. } => Resource::Fungible {
                    amount: Decimal::zero(),
                },
                ResourceType::NonFungible { .. } => Resource::NonFungible {
                    keys: BTreeSet::new(),
                },
            },
        );
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
        input: GetBucketDecimalInput,
    ) -> Result<GetBucketDecimalOutput, RuntimeError> {
        let amount = self
            .buckets
            .get(&input.bucket_id)
            .map(|b| b.amount())
            .or_else(|| {
                self.buckets_locked
                    .get(&input.bucket_id)
                    .map(|x| x.bucket().amount())
            })
            .ok_or(RuntimeError::BucketNotFound(input.bucket_id))?;

        Ok(GetBucketDecimalOutput { amount })
    }

    fn handle_get_bucket_resource_def(
        &mut self,
        input: GetBucketResourceDefInput,
    ) -> Result<GetBucketResourceDefOutput, RuntimeError> {
        let resource_def_ref = self
            .buckets
            .get(&input.bucket_id)
            .map(|b| b.resource_def_ref())
            .or_else(|| {
                self.buckets_locked
                    .get(&input.bucket_id)
                    .map(|x| x.bucket().resource_def_ref())
            })
            .ok_or(RuntimeError::BucketNotFound(input.bucket_id))?;

        Ok(GetBucketResourceDefOutput { resource_def_ref })
    }

    fn handle_take_non_fungible_from_bucket(
        &mut self,
        input: TakeNonFungibleFromBucketInput,
    ) -> Result<TakeNonFungibleFromBucketOutput, RuntimeError> {
        let new_bucket = self
            .buckets
            .get_mut(&input.bucket_id)
            .ok_or(RuntimeError::BucketNotFound(input.bucket_id))?
            .take_non_fungible(&input.key)
            .map_err(RuntimeError::BucketError)?;
        let bucket_id = self.track.new_bucket_id();
        self.buckets.insert(bucket_id, new_bucket);

        Ok(TakeNonFungibleFromBucketOutput { bucket_id })
    }

    fn handle_get_non_fungible_keys_in_bucket(
        &mut self,
        input: GetNonFungibleKeysInBucketInput,
    ) -> Result<GetNonFungibleKeysInBucketOutput, RuntimeError> {
        let bucket = self
            .buckets
            .get(&input.bucket_id)
            .ok_or(RuntimeError::BucketNotFound(input.bucket_id))?;

        Ok(GetNonFungibleKeysInBucketOutput {
            keys: bucket
                .get_non_fungible_keys()
                .map_err(RuntimeError::BucketError)?,
        })
    }

    fn handle_create_bucket_ref(
        &mut self,
        input: CreateBucketRefInput,
    ) -> Result<CreateBucketRefOutput, RuntimeError> {
        let bucket_id = input.bucket_id;
        let bucket_ref_id = self.track.new_bucket_ref_id();
        re_debug!(
            self,
            "Borrowing: bucket_id = {:?}, bucket_ref_id = {:?}",
            bucket_id,
            bucket_ref_id
        );

        match self.buckets_locked.get_mut(&bucket_id) {
            Some(bucket_rc) => {
                // re-borrow
                self.bucket_refs.insert(bucket_ref_id, bucket_rc.clone());
            }
            None => {
                // first time borrow
                let bucket = BucketRef::new(LockedBucket::new(
                    bucket_id,
                    self.buckets
                        .remove(&bucket_id)
                        .ok_or(RuntimeError::BucketNotFound(bucket_id))?,
                ));
                self.buckets_locked.insert(bucket_id, bucket.clone());
                self.bucket_refs.insert(bucket_ref_id, bucket);
            }
        }

        Ok(CreateBucketRefOutput { bucket_ref_id })
    }

    fn handle_drop_bucket_ref(
        &mut self,
        input: DropBucketRefInput,
    ) -> Result<DropBucketRefOutput, RuntimeError> {
        let bucket_ref_id = input.bucket_ref_id;

        let (count, bucket_id) = {
            let bucket_ref = self
                .bucket_refs
                .remove(&bucket_ref_id)
                .ok_or(RuntimeError::BucketRefNotFound(bucket_ref_id))?;
            re_debug!(
                self,
                "Dropping bucket ref: bucket_ref_id = {:?}, bucket = {:?}",
                bucket_ref_id,
                bucket_ref
            );
            (Rc::strong_count(&bucket_ref) - 1, bucket_ref.bucket_id())
        };

        if count == 1 {
            if let Some(b) = self.buckets_locked.remove(&bucket_id) {
                self.buckets
                    .insert(bucket_id, Rc::try_unwrap(b).unwrap().into());
            }
        }

        Ok(DropBucketRefOutput {})
    }

    fn handle_get_bucket_ref_amount(
        &mut self,
        input: GetBucketRefDecimalInput,
    ) -> Result<GetBucketRefDecimalOutput, RuntimeError> {
        let bucket_ref = self
            .bucket_refs
            .get(&input.bucket_ref_id)
            .ok_or(RuntimeError::BucketRefNotFound(input.bucket_ref_id))?;

        Ok(GetBucketRefDecimalOutput {
            amount: bucket_ref.bucket().amount(),
        })
    }

    fn handle_get_bucket_ref_resource_def(
        &mut self,
        input: GetBucketRefResourceDefInput,
    ) -> Result<GetBucketRefResourceDefOutput, RuntimeError> {
        let bucket_ref = self
            .bucket_refs
            .get(&input.bucket_ref_id)
            .ok_or(RuntimeError::BucketRefNotFound(input.bucket_ref_id))?;

        Ok(GetBucketRefResourceDefOutput {
            resource_def_ref: bucket_ref.bucket().resource_def_ref(),
        })
    }

    fn handle_get_non_fungible_keys_in_bucket_ref(
        &mut self,
        input: GetNonFungibleKeysInBucketRefInput,
    ) -> Result<GetNonFungibleKeysInBucketRefOutput, RuntimeError> {
        let bucket_ref = self
            .bucket_refs
            .get(&input.bucket_ref_id)
            .ok_or(RuntimeError::BucketRefNotFound(input.bucket_ref_id))?;

        Ok(GetNonFungibleKeysInBucketRefOutput {
            keys: bucket_ref
                .bucket()
                .get_non_fungible_keys()
                .map_err(RuntimeError::BucketError)?,
        })
    }

    fn handle_clone_bucket_ref(
        &mut self,
        input: CloneBucketRefInput,
    ) -> Result<CloneBucketRefOutput, RuntimeError> {
        let bucket_ref = self
            .bucket_refs
            .get(&input.bucket_ref_id)
            .ok_or(RuntimeError::BucketRefNotFound(input.bucket_ref_id))?
            .clone();

        let new_bucket_ref_id = self.track.new_bucket_ref_id();
        re_debug!(
            self,
            "Cloning: bucket_ref_id = {:?}, new bucket_ref_id = {:?}",
            input.bucket_ref_id,
            new_bucket_ref_id
        );

        self.bucket_refs.insert(new_bucket_ref_id, bucket_ref);
        Ok(CloneBucketRefOutput {
            bucket_ref_id: new_bucket_ref_id,
        })
    }

    fn handle_emit_log(&mut self, input: EmitLogInput) -> Result<EmitLogOutput, RuntimeError> {
        self.track.add_log(input.level, input.message);

        Ok(EmitLogOutput {})
    }

    fn handle_get_call_data(
        &mut self,
        _input: GetCallDataInput,
    ) -> Result<GetCallDataOutput, RuntimeError> {
        Ok(GetCallDataOutput {
            function: self.function()?,
            args: self.args()?,
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
        Ok(GetActorOutput {
            actor: self.actor()?,
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
                    UPDATE_RESOURCE_METADATA => {
                        self.handle(args, Self::handle_update_resource_metadata)
                    }

                    CREATE_EMPTY_VAULT => self.handle(args, Self::handle_create_vault),
                    PUT_INTO_VAULT => self.handle(args, Self::handle_put_into_vault),
                    TAKE_FROM_VAULT => self.handle(args, Self::handle_take_from_vault),
                    GET_VAULT_AMOUNT => self.handle(args, Self::handle_get_vault_amount),
                    GET_VAULT_RESOURCE_DEF => {
                        self.handle(args, Self::handle_get_vault_resource_def)
                    }
                    TAKE_NON_FUNGIBLE_FROM_VAULT => {
                        self.handle(args, Self::handle_take_non_fungible_from_vault)
                    }
                    GET_NON_FUNGIBLE_KEYS_IN_VAULT => {
                        self.handle(args, Self::handle_get_non_fungible_keys_in_vault)
                    }

                    CREATE_EMPTY_BUCKET => self.handle(args, Self::handle_create_bucket),
                    PUT_INTO_BUCKET => self.handle(args, Self::handle_put_into_bucket),
                    TAKE_FROM_BUCKET => self.handle(args, Self::handle_take_from_bucket),
                    GET_BUCKET_AMOUNT => self.handle(args, Self::handle_get_bucket_amount),
                    GET_BUCKET_RESOURCE_DEF => {
                        self.handle(args, Self::handle_get_bucket_resource_def)
                    }
                    TAKE_NON_FUNGIBLE_FROM_BUCKET => {
                        self.handle(args, Self::handle_take_non_fungible_from_bucket)
                    }
                    GET_NON_FUNGIBLE_KEYS_IN_BUCKET => {
                        self.handle(args, Self::handle_get_non_fungible_keys_in_bucket)
                    }

                    CREATE_BUCKET_REF => self.handle(args, Self::handle_create_bucket_ref),
                    DROP_BUCKET_REF => self.handle(args, Self::handle_drop_bucket_ref),
                    GET_BUCKET_REF_AMOUNT => self.handle(args, Self::handle_get_bucket_ref_amount),
                    GET_BUCKET_REF_RESOURCE_DEF => {
                        self.handle(args, Self::handle_get_bucket_ref_resource_def)
                    }
                    GET_NON_FUNGIBLE_KEYS_IN_BUCKET_REF => {
                        self.handle(args, Self::handle_get_non_fungible_keys_in_bucket_ref)
                    }
                    CLONE_BUCKET_REF => self.handle(args, Self::handle_clone_bucket_ref),

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
