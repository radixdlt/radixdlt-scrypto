use colored::*;
use sbor::*;
use scrypto::buffer::*;
use scrypto::engine::*;
use scrypto::rust::borrow::ToOwned;
use scrypto::rust::collections::*;
use scrypto::rust::fmt;
use scrypto::rust::format;
use scrypto::rust::rc::Rc;
use scrypto::rust::string::String;
use scrypto::rust::vec;
use scrypto::rust::vec::Vec;
use scrypto::types::*;
use wasmi::*;

use crate::engine::*;
use crate::engine::process::LazyMapState::{ClaimedByLazyMap, PartOfComponent};
use crate::ledger::*;
use crate::model::*;

macro_rules! re_trace {
    ($proc:expr, $($args: expr),+) => {
        if $proc.trace {
            $proc.log(LogLevel::Trace, format!($($args),+));
        }
    };
}

macro_rules! re_debug {
    ($proc:expr, $($args: expr),+) => {
        if $proc.trace {
            $proc.log(LogLevel::Debug, format!($($args),+));
        }
    };
}

macro_rules! re_info {
    ($proc:expr, $($args: expr),+) => {
        if $proc.trace {
            $proc.log(LogLevel::Info, format!($($args),+));
        }
    };
}

macro_rules! re_warn {
    ($proc:expr, $($args: expr),+) => {
        if $proc.trace {
            $proc.log(LogLevel::Warn, format!($($args),+));
        }
    };
}

enum LazyMapState {
    ClaimedByLazyMap(Mid),
    PartOfComponent(Address)
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
    buckets: HashMap<Bid, Bucket>,
    /// Buckets owned by this process (but LOCKED because there is a reference to it)
    buckets_locked: HashMap<Bid, BucketRef>,
    /// Bucket references
    bucket_refs: HashMap<Rid, BucketRef>,
    /// The buckets that will be moved to another process SHORTLY.
    moving_buckets: HashMap<Bid, Bucket>,
    /// The bucket refs that will be moved to another process SHORTLY.
    moving_bucket_refs: HashMap<Rid, BucketRef>,

    /// Lazy maps which haven't been assigned to a component or lazy map yet.
    unclaimed_lazy_maps: HashMap<Mid, LazyMap>,
    claimed_lazy_maps: HashMap<Mid, (LazyMap, Mid)>,
    lazy_map_descendents: HashMap<Mid, (HashSet<Mid>, HashSet<Vid>)>,
    /// Vaults which haven't been assigned to a component or lazy map yet.
    unclaimed_vaults: HashMap<Vid, Vault>,
    claimed_vaults: HashMap<Vid, (Vault, Mid)>,
    /// Components which have been loaded and possibly updated in the lifetime of this process.
    loaded_components: HashMap<Address, ValidatedData>,

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
    worktop: HashMap<Address, Bucket>,
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
    package_address: Address,
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
            unclaimed_lazy_maps: HashMap::new(),
            claimed_lazy_maps: HashMap::new(),
            lazy_map_descendents: HashMap::new(),
            unclaimed_vaults: HashMap::new(),
            claimed_vaults: HashMap::new(),
            loaded_components: HashMap::new(),
            vm: None,
            id_allocator: IdAllocator::new(IdSpace::Transaction),
            worktop: HashMap::new(),
        }
    }

    // (Transaction ONLY) Takes resource from worktop and returns a bucket.
    pub fn take_from_worktop(
        &mut self,
        amount: Option<Decimal>,
        resource_address: Address,
    ) -> Result<ValidatedData, RuntimeError> {
        re_debug!(
            self,
            "(Transaction) Taking from worktop: amount = {:?}, resource_address = {:?}",
            amount,
            resource_address
        );

        let new_bid = self
            .id_allocator
            .new_bid()
            .map_err(RuntimeError::IdAllocatorError)?;
        let bucket = match self.worktop.remove(&resource_address) {
            Some(mut bucket) => {
                if let Some(amount) = amount {
                    let to_return = bucket.take(amount).map_err(RuntimeError::BucketError)?;
                    if !bucket.amount().is_zero() {
                        self.worktop.insert(resource_address, bucket);
                    }
                    Ok(to_return)
                } else {
                    Ok(bucket)
                }
            }
            None => Err(RuntimeError::BucketError(BucketError::InsufficientBalance)),
        }?;
        self.buckets.insert(new_bid, bucket);
        Ok(validate_data(&scrypto_encode(&new_bid)).unwrap())
    }

    // (Transaction ONLY) Returns resource back to worktop.
    pub fn return_to_worktop(&mut self, bid: Bid) -> Result<ValidatedData, RuntimeError> {
        re_debug!(self, "(Transaction) Returning to worktop: bid = {:?}", bid);

        let bucket = self
            .buckets
            .remove(&bid)
            .ok_or(RuntimeError::BucketNotFound(bid))?;

        if !bucket.amount().is_zero() {
            if let Some(existing_bucket) = self.worktop.get_mut(&bucket.resource_address()) {
                existing_bucket
                    .put(bucket)
                    .map_err(RuntimeError::BucketError)?;
            } else {
                self.worktop.insert(bucket.resource_address(), bucket);
            }
        }
        Ok(validate_data(&scrypto_encode(&())).unwrap())
    }

    // (Transaction ONLY) Assert worktop contains at least this amount.
    pub fn assert_worktop_contains(
        &mut self,
        amount: Decimal,
        resource_address: Address,
    ) -> Result<ValidatedData, RuntimeError> {
        re_debug!(
            self,
            "(Transaction) Asserting worktop contains: amount = {:?}, resource_address = {:?}",
            amount,
            resource_address
        );

        let balance = match self.worktop.get(&resource_address) {
            Some(bucket) => bucket.amount(),
            None => Decimal::zero(),
        };

        if balance < amount {
            re_warn!(
                self,
                "(Transaction) Assertion failed: required = {}, actual = {}, resource_address = {}",
                amount,
                balance,
                resource_address
            );
            Err(RuntimeError::AssertionFailed)
        } else {
            Ok(validate_data(&scrypto_encode(&())).unwrap())
        }
    }

    // (Transaction ONLY) Creates a bucket ref.
    pub fn create_bucket_ref(&mut self, bid: Bid) -> Result<ValidatedData, RuntimeError> {
        re_debug!(self, "(Transaction) Creating bucket ref: bid = {:?}", bid);

        let new_rid = self
            .id_allocator
            .new_rid()
            .map_err(RuntimeError::IdAllocatorError)?;
        match self.buckets_locked.get_mut(&bid) {
            Some(bucket_rc) => {
                // re-borrow
                self.bucket_refs.insert(new_rid, bucket_rc.clone());
            }
            None => {
                // first time borrow
                let bucket = BucketRef::new(LockedBucket::new(
                    bid,
                    self.buckets
                        .remove(&bid)
                        .ok_or(RuntimeError::BucketNotFound(bid))?,
                ));
                self.buckets_locked.insert(bid, bucket.clone());
                self.bucket_refs.insert(new_rid, bucket);
            }
        };

        Ok(validate_data(&scrypto_encode(&new_rid)).unwrap())
    }

    // (Transaction ONLY) Clone a bucket ref.
    pub fn clone_bucket_ref(&mut self, rid: Rid) -> Result<ValidatedData, RuntimeError> {
        re_debug!(self, "(Transaction) Cloning bucket ref: rid = {:?}", rid);

        let new_rid = self
            .id_allocator
            .new_rid()
            .map_err(RuntimeError::IdAllocatorError)?;
        let bucket_ref = self
            .bucket_refs
            .get(&rid)
            .ok_or(RuntimeError::BucketRefNotFound(rid))?
            .clone();
        self.bucket_refs.insert(new_rid, bucket_ref);

        Ok(validate_data(&scrypto_encode(&new_rid)).unwrap())
    }

    // (Transaction ONLY) Drop a bucket ref.
    pub fn drop_bucket_ref(&mut self, rid: Rid) -> Result<ValidatedData, RuntimeError> {
        re_debug!(self, "(Transaction) Dropping bucket ref: rid = {:?}", rid);

        self.handle_drop_bucket_ref(DropBucketRefInput { rid })?;

        Ok(validate_data(&scrypto_encode(&())).unwrap())
    }

    /// (Transaction ONLY) Calls a method.
    pub fn call_method_with_all_resources(
        &mut self,
        component_address: Address,
        method: &str,
    ) -> Result<ValidatedData, RuntimeError> {
        re_debug!(
            self,
            "(Transaction) Calling method with all resources started"
        );
        // 1. Move collected resource to temp buckets
        for (_, bucket) in self.worktop.clone() {
            let bid = self.track.new_bid(); // this is unbounded
            self.buckets.insert(bid, bucket);
        }
        self.worktop.clear();

        // 2. Drop all bucket refs to unlock the buckets
        self.drop_all_bucket_refs()?;

        // 3. Call the method with all buckets
        let to_deposit: Vec<Bid> = self.buckets.keys().cloned().collect();
        let invocation = self.prepare_call_method(
            component_address,
            method,
            vec![validate_data(&scrypto_encode(&to_deposit)).unwrap()],
        )?;
        let result = self.call(invocation);

        re_debug!(
            self,
            "(Transaction) Calling method with all resources ended"
        );
        result
    }

    /// (SYSTEM ONLY)  Creates a bucket ref which references a virtual bucket
    pub fn create_virtual_bucket_ref(&mut self, bid: Bid, rid: Rid, bucket: Bucket) {
        let locked_bucket = LockedBucket::new(bid, bucket);
        let bucket_ref = BucketRef::new(locked_bucket);
        self.bucket_refs.insert(rid, bucket_ref);
    }

    /// Moves buckets and bucket refs into this process.
    pub fn move_in_resources(
        &mut self,
        buckets: HashMap<Bid, Bucket>,
        bucket_refs: HashMap<Rid, BucketRef>,
    ) -> Result<(), RuntimeError> {
        if self.depth == 0 {
            assert!(bucket_refs.is_empty());

            for (_, bucket) in buckets {
                if !bucket.amount().is_zero() {
                    let address = bucket.resource_address();
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
    pub fn move_out_resources(&mut self) -> (HashMap<Bid, Bucket>, HashMap<Rid, BucketRef>) {
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
            invocation.package_address,
            invocation.export_name
        );

        // Load the code
        let (module, memory) = self
            .track
            .load_module(invocation.package_address)
            .ok_or(RuntimeError::PackageNotFound(invocation.package_address))?;
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
                let data = validate_data(&self.read_bytes(ptr)?)
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
        package_address: Address,
        blueprint_name: &str,
        function: &str,
        args: Vec<ValidatedData>,
    ) -> Result<Invocation, RuntimeError> {
        Ok(Invocation {
            actor: Actor::Blueprint(package_address, blueprint_name.to_owned()),
            package_address,
            export_name: format!("{}_main", blueprint_name),
            function: function.to_owned(),
            args,
        })
    }

    /// Prepares a method call.
    pub fn prepare_call_method(
        &mut self,
        component_address: Address,
        method: &str,
        args: Vec<ValidatedData>,
    ) -> Result<Invocation, RuntimeError> {
        let component = self
            .track
            .get_component(component_address)
            .ok_or(RuntimeError::ComponentNotFound(component_address))?
            .clone();
        let mut args_with_self = vec![validate_data(&scrypto_encode(&component_address)).unwrap()];
        args_with_self.extend(args);

        Ok(Invocation {
            actor: Actor::Component(component_address),
            package_address: component.package_address(),
            export_name: format!("{}_main", component.blueprint_name()),
            function: method.to_owned(),
            args: args_with_self,
        })
    }

    /// Prepares an ABI call.
    pub fn prepare_call_abi(
        &mut self,
        package_address: Address,
        blueprint_name: &str,
    ) -> Result<Invocation, RuntimeError> {
        Ok(Invocation {
            actor: Actor::Blueprint(package_address, blueprint_name.to_owned()),
            package_address: package_address,
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
        let bids: Vec<Bid> = self
            .buckets_locked
            .values()
            .filter(|v| Rc::strong_count(v) == 1)
            .map(|v| v.bucket_id())
            .collect();
        for bid in bids {
            re_debug!(self, "Changing bucket {:?} to unlocked state", bid);
            let bucket_rc = self.buckets_locked.remove(&bid).unwrap();
            let bucket = Rc::try_unwrap(bucket_rc).unwrap();
            self.buckets.insert(bid, bucket.into());
        }

        Ok(result)
    }

    /// Calls a function.
    pub fn call_function(
        &mut self,
        package_address: Address,
        blueprint_name: &str,
        function: &str,
        args: Vec<ValidatedData>,
    ) -> Result<ValidatedData, RuntimeError> {
        re_debug!(self, "Call function started");
        let invocation =
            self.prepare_call_function(package_address, blueprint_name, function, args)?;
        let result = self.call(invocation);
        re_debug!(self, "Call function ended");
        result
    }

    /// Calls a method.
    pub fn call_method(
        &mut self,
        component_address: Address,
        method: &str,
        args: Vec<ValidatedData>,
    ) -> Result<ValidatedData, RuntimeError> {
        re_debug!(self, "Call method started");
        let invocation = self.prepare_call_method(component_address, method, args)?;
        let result = self.call(invocation);
        re_debug!(self, "Call method ended");
        result
    }

    /// Calls the ABI generator of a blueprint.
    pub fn call_abi(
        &mut self,
        package_address: Address,
        blueprint_name: &str,
    ) -> Result<ValidatedData, RuntimeError> {
        re_debug!(self, "Call abi started");
        let invocation = self.prepare_call_abi(package_address, blueprint_name)?;
        let result = self.call(invocation);
        re_debug!(self, "Call abi ended");
        result
    }

    /// Drops all bucket refs owned by this process.
    pub fn drop_all_bucket_refs(&mut self) -> Result<(), RuntimeError> {
        let rids: Vec<Rid> = self.bucket_refs.keys().cloned().collect();
        for rid in rids {
            self.handle_drop_bucket_ref(DropBucketRefInput { rid })?;
        }
        Ok(())
    }

    /// Checks resource leak.
    pub fn check_resource(&self) -> Result<(), RuntimeError> {
        re_debug!(self, "Resource check started");
        let mut success = true;

        for (bid, bucket) in &self.buckets {
            re_warn!(self, "Dangling bucket: {:?}, {:?}", bid, bucket);
            success = false;
        }
        for (bid, bucket) in &self.buckets_locked {
            re_warn!(self, "Dangling bucket: {:?}, {:?}", bid, bucket);
            success = false;
        }
        for (_, bucket) in &self.worktop {
            re_warn!(self, "Dangling resource: {:?}", bucket);
            success = false;
        }
        for (vid, vault) in &self.unclaimed_vaults {
            re_warn!(self, "Dangling vault: {:?}, {:?}", vid, vault);
            success = false;
        }
        for (mid, lazy_map) in &self.unclaimed_lazy_maps {
            re_warn!(self, "Dangling lazy map: {:?}, {:?}", mid, lazy_map);
            success = false;
        }
        assert!(self.claimed_vaults.is_empty());
        assert!(self.claimed_lazy_maps.is_empty());
        assert!(self.lazy_map_descendents.is_empty());

        re_debug!(self, "Resource check ended");
        if success {
            Ok(())
        } else {
            Err(RuntimeError::ResourceCheckFailure)
        }
    }

    /// Logs a message to the console.
    #[allow(unused_variables)]
    pub fn log(&self, level: LogLevel, msg: String) {
        let (l, m) = match level {
            LogLevel::Error => ("ERROR".red(), msg.red()),
            LogLevel::Warn => ("WARN".yellow(), msg.yellow()),
            LogLevel::Info => ("INFO".green(), msg.green()),
            LogLevel::Debug => ("DEBUG".cyan(), msg.cyan()),
            LogLevel::Trace => ("TRACE".normal(), msg.normal()),
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

    /// Return the package address
    fn package(&self) -> Result<Address, RuntimeError> {
        self.vm
            .as_ref()
            .ok_or(RuntimeError::InterpreterNotStarted)
            .map(|vm| vm.invocation.package_address)
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
        self.move_buckets(&validated.buckets)?;
        if is_argument {
            self.move_bucket_refs(&validated.bucket_refs)?;
        } else {
            if !validated.bucket_refs.is_empty() {
                return Err(RuntimeError::BucketRefNotAllowed);
            }
        }
        if !validated.lazy_maps.is_empty() {
            return Err(RuntimeError::LazyMapNotAllowed);
        }
        if !validated.vaults.is_empty() {
            return Err(RuntimeError::VaultNotAllowed);
        }
        Ok(())
    }

    fn process_component_data(data: &[u8]) -> Result<ValidatedData, RuntimeError> {
        let validated = validate_data(data).map_err(RuntimeError::DataValidationError)?;
        if !validated.buckets.is_empty() {
            return Err(RuntimeError::BucketNotAllowed);
        }
        if !validated.bucket_refs.is_empty() {
            return Err(RuntimeError::BucketRefNotAllowed);
        }
        // lazy map allowed
        // vaults allowed
        Ok(validated)
    }

    fn process_map_data(data: &[u8]) -> Result<ValidatedData, RuntimeError> {
        let validated = validate_data(data).map_err(RuntimeError::DataValidationError)?;
        if !validated.buckets.is_empty() {
            return Err(RuntimeError::BucketNotAllowed);
        }
        if !validated.bucket_refs.is_empty() {
            return Err(RuntimeError::BucketRefNotAllowed);
        }
        // lazy map allowed
        // vaults allowed
        Ok(validated)
    }

    fn process_non_fungible_data(&mut self, data: &[u8]) -> Result<ValidatedData, RuntimeError> {
        let validated = validate_data(data).map_err(RuntimeError::DataValidationError)?;
        if !validated.buckets.is_empty() {
            return Err(RuntimeError::BucketNotAllowed);
        }
        if !validated.bucket_refs.is_empty() {
            return Err(RuntimeError::BucketRefNotAllowed);
        }
        if !validated.lazy_maps.is_empty() {
            return Err(RuntimeError::LazyMapNotAllowed);
        }
        if !validated.vaults.is_empty() {
            return Err(RuntimeError::VaultNotAllowed);
        }
        Ok(validated)
    }

    /// Remove transient buckets from this process
    fn move_buckets(&mut self, buckets: &[Bid]) -> Result<(), RuntimeError> {
        for bid in buckets {
            let bucket = self
                .buckets
                .remove(bid)
                .ok_or(RuntimeError::BucketNotFound(*bid))?;
            re_debug!(self, "Moving bucket: {:?}, {:?}", bid, bucket);
            self.moving_buckets.insert(*bid, bucket);
        }
        Ok(())
    }

    /// Remove transient buckets from this process
    fn move_bucket_refs(&mut self, bucket_refs: &[Rid]) -> Result<(), RuntimeError> {
        for rid in bucket_refs {
            let bucket_ref = self
                .bucket_refs
                .remove(rid)
                .ok_or(RuntimeError::BucketRefNotFound(*rid))?;
            re_debug!(self, "Moving bucket ref: {:?}, {:?}", rid, bucket_ref);
            self.moving_bucket_refs.insert(*rid, bucket_ref);
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

    fn expect_package_address(address: Address) -> Result<(), RuntimeError> {
        if address.is_package() {
            Ok(())
        } else {
            Err(RuntimeError::InvalidPackageAddress(address))
        }
    }

    fn expect_component_address(address: Address) -> Result<(), RuntimeError> {
        if address.is_component() {
            Ok(())
        } else {
            Err(RuntimeError::InvalidComponentAddress(address))
        }
    }

    fn expect_resource_address(address: Address) -> Result<(), RuntimeError> {
        if address.is_resource_def() {
            Ok(())
        } else {
            Err(RuntimeError::InvalidResourceDefAddress(address))
        }
    }

    fn check_badge(&mut self, optional_rid: Option<Rid>) -> Result<Option<Address>, RuntimeError> {
        if let Some(rid) = optional_rid {
            // retrieve bucket reference
            let bucket_ref = self
                .bucket_refs
                .get(&rid)
                .ok_or(RuntimeError::BucketRefNotFound(rid))?;

            // read amount & address
            if bucket_ref.bucket().amount().is_zero() {
                return Err(RuntimeError::EmptyBucketRef);
            }
            let resource_address = bucket_ref.bucket().resource_address();

            // drop bucket reference after use
            self.handle_drop_bucket_ref(DropBucketRefInput { rid })?;

            Ok(Some(resource_address))
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
        let package_address = self.track.new_package_address();

        if self.track.get_package(package_address).is_some() {
            return Err(RuntimeError::PackageAlreadyExists(package_address));
        }
        validate_module(&input.code).map_err(RuntimeError::WasmValidationError)?;

        re_debug!(self, "New package: {:?}", package_address);
        self.track
            .put_package(package_address, Package::new(input.code));

        Ok(PublishPackageOutput { package_address })
    }

    fn handle_call_function(
        &mut self,
        input: CallFunctionInput,
    ) -> Result<CallFunctionOutput, RuntimeError> {
        Self::expect_package_address(input.package_address)?;

        let mut validated_args = Vec::new();
        for arg in input.args {
            validated_args.push(validate_data(&arg).map_err(RuntimeError::DataValidationError)?);
        }

        re_debug!(
            self,
            "CALL started: package = {:?}, blueprint = {:?}, function = {:?}, args = {:?}",
            input.package_address,
            input.blueprint_name,
            input.function,
            validated_args
        );

        let invocation = self.prepare_call_function(
            input.package_address,
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
        Self::expect_component_address(input.component_address)?;

        let mut validated_args = Vec::new();
        for arg in input.args {
            validated_args.push(validate_data(&arg).map_err(RuntimeError::DataValidationError)?);
        }

        re_debug!(
            self,
            "CALL started: component = {:?}, method = {:?}, args = {:?}",
            input.component_address,
            input.method,
            validated_args
        );

        let invocation = self.prepare_call_method(
            input.component_address,
            input.method.as_str(),
            validated_args,
        )?;
        let result = self.call(invocation);

        re_debug!(self, "CALL finished");
        Ok(CallMethodOutput { rtn: result?.raw })
    }

    fn handle_create_component(
        &mut self,
        input: CreateComponentInput,
    ) -> Result<CreateComponentOutput, RuntimeError> {
        let component_address = self.track.new_component_address();

        if self.track.get_component(component_address).is_some() {
            return Err(RuntimeError::ComponentAlreadyExists(component_address));
        }

        let data = Self::process_component_data(&input.state)?;
        re_debug!(
            self,
            "New component: address = {:?}, state = {:?}",
            component_address,
            data
        );

        for vid in data.vaults {
            let vault = self
                .unclaimed_vaults
                .remove(&vid)
                .ok_or(RuntimeError::VaultNotFound(vid))?;
            self.track.put_vault(component_address, vid, vault);
        }

        for mid in data.lazy_maps {
            let lazy_map = self
                .unclaimed_lazy_maps
                .remove(&mid)
                .ok_or(RuntimeError::LazyMapNotFound(mid))?;
            self.track.put_lazy_map(component_address, mid, lazy_map);

            match self.lazy_map_descendents.remove(&mid) {
                Some((mids, vids)) => {
                    for descendent_mid in mids {
                        let descendent_lazy_map = self.claimed_lazy_maps.remove(&descendent_mid).unwrap().0;
                        self.track.put_lazy_map(component_address, descendent_mid, descendent_lazy_map);
                    }

                    for vid in vids {
                        let descendent_vault = self.claimed_vaults.remove(&vid).unwrap().0;
                        self.track.put_vault(component_address, vid, descendent_vault);
                    }
                },
                None => {}
            }
        }

        let component = Component::new(self.package()?, input.blueprint_name, data.raw);
        self.track.put_component(component_address, component);

        Ok(CreateComponentOutput { component_address })
    }

    fn handle_get_component_info(
        &mut self,
        input: GetComponentInfoInput,
    ) -> Result<GetComponentInfoOutput, RuntimeError> {
        Self::expect_component_address(input.component_address)?;
        // TODO: restrict access?

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
        input: GetComponentStateInput,
    ) -> Result<GetComponentStateOutput, RuntimeError> {
        Self::expect_component_address(input.component_address)?;
        // TODO: restrict access

        let component = self
            .track
            .get_component(input.component_address)
            .ok_or(RuntimeError::ComponentNotFound(input.component_address))?;

        let state = component.state();
        let updating_component_data = Self::process_component_data(state).unwrap();
        let existing = self.loaded_components.insert(input.component_address, updating_component_data);
        existing.map_or(Ok(GetComponentStateOutput { state: state.to_owned() }),
            |_| Err(RuntimeError::ComponentAlreadyLoaded(input.component_address))
        )
    }

    fn handle_put_component_state(
        &mut self,
        input: PutComponentStateInput,
    ) -> Result<PutComponentStateOutput, RuntimeError> {
        Self::expect_component_address(input.component_address)?;
        // TODO: restrict access

        let old_state = self.loaded_components.remove(&input.component_address)
            .ok_or(RuntimeError::ComponentNotFound(input.component_address))?;
        let new_state = Self::process_component_data(&input.state)?;
        re_debug!(self, "New component state: {:?}", new_state);

        // Only allow vaults to be added, never removed
        let mut old_vaults: HashSet<Vid> = HashSet::from_iter(old_state.vaults.into_iter());
        for vid in new_state.vaults {
            if !old_vaults.remove(&vid) {
                let vault = self
                    .unclaimed_vaults
                    .remove(&vid)
                    .ok_or(RuntimeError::VaultNotFound(vid))?;
                self.track.put_vault(input.component_address, vid, vault);
            }
        }
        old_vaults.into_iter().try_for_each(|vid| Err(RuntimeError::VaultRemoved(vid)))?;

        // Only allow lazy maps to be added, never removed
        let mut old_lazy_maps: HashSet<Mid> = HashSet::from_iter(old_state.lazy_maps.into_iter());
        for mid in new_state.lazy_maps {
            if !old_lazy_maps.remove(&mid) {
                let lazy_map = self
                    .unclaimed_lazy_maps
                    .remove(&mid)
                    .ok_or(RuntimeError::LazyMapNotFound(mid))?;
                self.track.put_lazy_map(input.component_address, mid, lazy_map);
                match self.lazy_map_descendents.remove(&mid) {
                    Some((mids, vids)) => {
                        for descendent_mid in mids {
                            let descendent_lazy_map = self.claimed_lazy_maps.remove(&descendent_mid).unwrap().0;
                            self.track.put_lazy_map(input.component_address, descendent_mid, descendent_lazy_map);
                        }

                        for vid in vids {
                            let vault = self.claimed_vaults.remove(&vid).unwrap().0;
                            self.track.put_vault(input.component_address, vid, vault);
                        }
                    },
                    None => {}
                }
            }
        }
        old_lazy_maps.into_iter().try_for_each(|mid| Err(RuntimeError::LazyMapRemoved(mid)))?;

        let component = self.track.get_component_mut(input.component_address).unwrap();
        component.set_state(new_state.raw);

        Ok(PutComponentStateOutput {})
    }

    fn handle_create_lazy_map(
        &mut self,
        _input: CreateLazyMapInput,
    ) -> Result<CreateLazyMapOutput, RuntimeError> {
        let mid = self.track.new_mid();
        self.unclaimed_lazy_maps.insert(mid, LazyMap::new(self.package()?));

        Ok(CreateLazyMapOutput { mid })
    }

    fn get_local_lazy_map(&mut self, mid: Mid) -> Result<(&mut LazyMap, LazyMapState), RuntimeError> {
        match self.unclaimed_lazy_maps.get_mut(&mid) {
            Some(map) => Ok((map, ClaimedByLazyMap(mid))),
            None => {
                match self.claimed_lazy_maps.get_mut(&mid) {
                    Some((map, ancestor)) => Ok((map, ClaimedByLazyMap(ancestor.clone()))),
                    None => {
                        match self.vm.as_ref().unwrap().invocation.actor {
                            Actor::Component(component_address) => {
                                match self.track.get_lazy_map_mut(component_address, mid) {
                                    Some(lazy_map) => Ok((lazy_map, PartOfComponent(component_address))),
                                    None => Err(RuntimeError::LazyMapNotFound(mid))
                                }
                            },
                            _ => Err(RuntimeError::LazyMapNotFound(mid))
                        }
                    }
                }
            }
        }
    }

    fn handle_get_lazy_map_entry(
        &mut self,
        input: GetLazyMapEntryInput,
    ) -> Result<GetLazyMapEntryOutput, RuntimeError> {
        let lazy_map = self.get_local_lazy_map(input.mid)?.0;
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

        let lazy_map = self.get_local_lazy_map(input.mid)?;
        let ancestor = lazy_map.1;
        let mut old_entry_state = lazy_map.0
            .get_entry(&input.key)
            .map_or((HashSet::new(), HashSet::new()), |e| {
                let data = Self::process_map_data(e).unwrap();
                let old_vaults = HashSet::from_iter(data.vaults.into_iter());
                let old_lazy_maps = HashSet::from_iter(data.lazy_maps.into_iter());
                (old_vaults, old_lazy_maps)
            });

        // Only allow vaults to be added, never removed
        for vid in new_entry_state.vaults {
            if !old_entry_state.0.remove(&vid) {
                let vault = self
                    .unclaimed_vaults
                    .remove(&vid)
                    .ok_or(RuntimeError::VaultNotFound(vid))?;
                match ancestor {
                    ClaimedByLazyMap(ancestor_mid) => {
                        match self.lazy_map_descendents.get_mut(&ancestor_mid) {
                            Some((_, vids)) => {
                                vids.insert(vid);
                            },
                            None => {
                                self.lazy_map_descendents.insert(ancestor_mid, (HashSet::new(), {
                                    let mut mids = HashSet::new();
                                    mids.insert(vid);
                                    mids
                                }));
                            }
                        }
                        self.claimed_vaults.insert(vid, (vault, ancestor_mid));
                    },
                    PartOfComponent(component_address) => {
                        self.track.put_vault(component_address, vid, vault);
                    }
                }
            }
        }
        old_entry_state.0.into_iter().try_for_each(|vid| Err(RuntimeError::VaultRemoved(vid)))?;

        // Only allow lazy maps to be added, never removed
        for mid in new_entry_state.lazy_maps {
            if !old_entry_state.1.remove(&mid) {
                let lazy_map = self
                    .unclaimed_lazy_maps
                    .remove(&mid)
                    .ok_or(RuntimeError::LazyMapNotFound(mid))?;

                match ancestor {
                    ClaimedByLazyMap(ancestor_mid) => {
                        let old_set = self.lazy_map_descendents.remove(&mid)
                            .unwrap_or((HashSet::new(), HashSet::new()));
                        for mid in old_set.0.iter() {
                            let old = self.claimed_lazy_maps.remove(mid).unwrap();
                            self.claimed_lazy_maps.insert(mid.clone(), (old.0, ancestor_mid));
                        }
                        for vid in old_set.1.iter() {
                            let old = self.claimed_vaults.remove(vid).unwrap();
                            self.claimed_vaults.insert(vid.clone(), (old.0, ancestor_mid));
                        }
                        let mut new_descendent_set = self.lazy_map_descendents.remove(&ancestor_mid)
                            .unwrap_or((HashSet::new(), HashSet::new()));
                        new_descendent_set.0.extend(old_set.0);
                        new_descendent_set.0.insert(mid);
                        new_descendent_set.1.extend(old_set.1);

                        self.claimed_lazy_maps.insert(mid, (lazy_map, ancestor_mid));
                        self.lazy_map_descendents.insert(ancestor_mid, new_descendent_set);
                    },
                    PartOfComponent(component_address) => {
                        self.track.put_lazy_map(component_address, mid, lazy_map);
                    }
                }
            }
        }
        old_entry_state.1.into_iter().try_for_each(|mid| Err(RuntimeError::LazyMapRemoved(mid)))?;

        let lazy_map = self.get_local_lazy_map(input.mid)?;
        lazy_map.0.set_entry(key.raw, new_entry_state.raw);

        Ok(PutLazyMapEntryOutput {})
    }

    fn allocate_resource(
        &mut self,
        resource_address: Address,
        new_supply: NewSupply,
    ) -> Result<Supply, RuntimeError> {
        match new_supply {
            NewSupply::Fungible { amount } => Ok(Supply::Fungible { amount }),
            NewSupply::NonFungible { entries } => {
                let mut keys = BTreeSet::new();

                for (key, data) in entries {
                    if self
                        .track
                        .get_non_fungible(resource_address, &key)
                        .is_some()
                    {
                        return Err(RuntimeError::NonFungibleAlreadyExists(
                            resource_address,
                            key.clone(),
                        ));
                    }

                    let immutable_data = self.process_non_fungible_data(&data.0)?;
                    let mutable_data = self.process_non_fungible_data(&data.1)?;

                    self.track.put_non_fungible(
                        resource_address,
                        &key,
                        NonFungible::new(immutable_data.raw, mutable_data.raw),
                    );
                    keys.insert(key.clone());
                }

                Ok(Supply::NonFungible { keys })
            }
        }
    }

    fn handle_create_resource(
        &mut self,
        input: CreateResourceInput,
    ) -> Result<CreateResourceOutput, RuntimeError> {
        for (address, _) in &input.authorities {
            Self::expect_resource_address(*address)?;
        }

        // instantiate resource definition
        let resource_address = self.track.new_resource_address();
        if self.track.get_resource_def(resource_address).is_some() {
            return Err(RuntimeError::ResourceDefAlreadyExists(resource_address));
        }
        re_debug!(self, "New resource definition: {:?}", resource_address);
        let definition = ResourceDef::new(
            input.resource_type,
            input.metadata,
            input.flags,
            input.mutable_flags,
            input.authorities,
            &input.initial_supply,
        )
        .map_err(RuntimeError::ResourceDefError)?;
        self.track.put_resource_def(resource_address, definition);

        // allocate supply
        let bucket = if let Some(initial_supply) = input.initial_supply {
            let supply = self.allocate_resource(resource_address, initial_supply)?;

            let bucket = Bucket::new(resource_address, input.resource_type, supply);
            let bid = self.track.new_bid();
            self.buckets.insert(bid, bucket);
            Some(bid)
        } else {
            None
        };

        Ok(CreateResourceOutput {
            resource_address,
            bucket,
        })
    }

    fn handle_get_resource_metadata(
        &mut self,
        input: GetResourceMetadataInput,
    ) -> Result<GetResourceMetadataOutput, RuntimeError> {
        Self::expect_resource_address(input.resource_address)?;

        let resource_def = self
            .track
            .get_resource_def(input.resource_address)
            .ok_or(RuntimeError::ResourceDefNotFound(input.resource_address))?;

        Ok(GetResourceMetadataOutput {
            metadata: resource_def.metadata().clone(),
        })
    }

    fn handle_get_resource_total_supply(
        &mut self,
        input: GetResourceTotalSupplyInput,
    ) -> Result<GetResourceTotalSupplyOutput, RuntimeError> {
        Self::expect_resource_address(input.resource_address)?;

        let resource_def = self
            .track
            .get_resource_def(input.resource_address)
            .ok_or(RuntimeError::ResourceDefNotFound(input.resource_address))?;

        Ok(GetResourceTotalSupplyOutput {
            total_supply: resource_def.total_supply(),
        })
    }

    fn handle_get_resource_flags(
        &mut self,
        input: GetResourceFlagsInput,
    ) -> Result<GetResourceFlagsOutput, RuntimeError> {
        Self::expect_resource_address(input.resource_address)?;

        let resource_def = self
            .track
            .get_resource_def(input.resource_address)
            .ok_or(RuntimeError::ResourceDefNotFound(input.resource_address))?;

        Ok(GetResourceFlagsOutput {
            flags: resource_def.flags(),
        })
    }

    fn handle_update_resource_flags(
        &mut self,
        input: UpdateResourceFlagsInput,
    ) -> Result<UpdateResourceFlagsOutput, RuntimeError> {
        Self::expect_resource_address(input.resource_address)?;
        let badge = self.check_badge(Some(input.auth))?;

        let resource_def = self
            .track
            .get_resource_def_mut(input.resource_address)
            .ok_or(RuntimeError::ResourceDefNotFound(input.resource_address))?;
        resource_def
            .update_flags(input.new_flags, badge)
            .map_err(RuntimeError::ResourceDefError)?;

        Ok(UpdateResourceFlagsOutput {})
    }

    fn handle_get_resource_mutable_flags(
        &mut self,
        input: GetResourceMutableFlagsInput,
    ) -> Result<GetResourceMutableFlagsOutput, RuntimeError> {
        Self::expect_resource_address(input.resource_address)?;

        let resource_def = self
            .track
            .get_resource_def(input.resource_address)
            .ok_or(RuntimeError::ResourceDefNotFound(input.resource_address))?;

        Ok(GetResourceMutableFlagsOutput {
            mutable_flags: resource_def.mutable_flags(),
        })
    }

    fn handle_update_resource_mutable_flags(
        &mut self,
        input: UpdateResourceMutableFlagsInput,
    ) -> Result<UpdateResourceMutableFlagsOutput, RuntimeError> {
        Self::expect_resource_address(input.resource_address)?;
        let badge = self.check_badge(Some(input.auth))?;

        let resource_def = self
            .track
            .get_resource_def_mut(input.resource_address)
            .ok_or(RuntimeError::ResourceDefNotFound(input.resource_address))?;
        resource_def
            .update_mutable_flags(input.new_mutable_flags, badge)
            .map_err(RuntimeError::ResourceDefError)?;

        Ok(UpdateResourceMutableFlagsOutput {})
    }

    fn handle_get_resource_type(
        &mut self,
        input: GetResourceTypeInput,
    ) -> Result<GetResourceTypeOutput, RuntimeError> {
        Self::expect_resource_address(input.resource_address)?;

        let resource_def = self
            .track
            .get_resource_def(input.resource_address)
            .ok_or(RuntimeError::ResourceDefNotFound(input.resource_address))?;

        Ok(GetResourceTypeOutput {
            resource_type: resource_def.resource_type(),
        })
    }

    fn handle_mint_resource(
        &mut self,
        input: MintResourceInput,
    ) -> Result<MintResourceOutput, RuntimeError> {
        Self::expect_resource_address(input.resource_address)?;
        let badge = self.check_badge(Some(input.auth))?;

        // allocate resource
        let supply = self.allocate_resource(input.resource_address, input.new_supply)?;

        // mint resource
        let resource_def = self
            .track
            .get_resource_def_mut(input.resource_address)
            .ok_or(RuntimeError::ResourceDefNotFound(input.resource_address))?;
        resource_def
            .mint(&supply, badge)
            .map_err(RuntimeError::ResourceDefError)?;

        // wrap resource into a bucket
        let bucket = Bucket::new(input.resource_address, resource_def.resource_type(), supply);
        let bid = self.track.new_bid();
        self.buckets.insert(bid, bucket);

        Ok(MintResourceOutput { bid })
    }

    fn handle_burn_resource(
        &mut self,
        input: BurnResourceInput,
    ) -> Result<BurnResourceOutput, RuntimeError> {
        let badge = self.check_badge(input.auth)?;

        let bucket = self
            .buckets
            .remove(&input.bid)
            .ok_or(RuntimeError::BucketNotFound(input.bid))?;

        let resource_def = self
            .track
            .get_resource_def_mut(bucket.resource_address())
            .ok_or(RuntimeError::ResourceDefNotFound(bucket.resource_address()))?;

        resource_def
            .burn(bucket.supply(), badge)
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
            .get_resource_def(input.resource_address)
            .ok_or(RuntimeError::ResourceDefNotFound(input.resource_address))?;
        resource_def
            .check_update_non_fungible_mutable_data_auth(badge)
            .map_err(RuntimeError::ResourceDefError)?;
        // update state
        let data = self.process_non_fungible_data(&input.new_mutable_data)?;
        self.track
            .get_non_fungible_mut(input.resource_address, &input.key)
            .ok_or(RuntimeError::NonFungibleNotFound(
                input.resource_address,
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
            .get_non_fungible(input.resource_address, &input.key)
            .ok_or(RuntimeError::NonFungibleNotFound(
                input.resource_address,
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
            .get_resource_def_mut(input.resource_address)
            .ok_or(RuntimeError::ResourceDefNotFound(input.resource_address))?;
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
            .get_resource_def(input.resource_address)
            .ok_or(RuntimeError::ResourceDefNotFound(input.resource_address))?;

        let new_vault = Vault::new(
            Bucket::new(
                input.resource_address,
                definition.resource_type(),
                match definition.resource_type() {
                    ResourceType::Fungible { .. } => Supply::Fungible {
                        amount: Decimal::zero(),
                    },
                    ResourceType::NonFungible { .. } => Supply::NonFungible {
                        keys: BTreeSet::new(),
                    },
                },
            ),
            self.package()?,
        );
        let vid = self.track.new_vid();
        self.unclaimed_vaults.insert(vid, new_vault);

        Ok(CreateEmptyVaultOutput { vid })
    }

    fn get_local_vault(&mut self, vid: Vid) -> Result<&mut Vault, RuntimeError> {
        match self.unclaimed_vaults.get_mut(&vid) {
            Some(vault) => Ok(vault),
            None => match self.claimed_vaults.get_mut(&vid) {
                Some((vault, _)) => Ok(vault),
                None => match self.vm.as_ref().unwrap().invocation.actor {
                    Actor::Component(component_address) => {
                        match self.track.get_vault_mut(component_address, vid) {
                            Some(vault) => Ok(vault),
                            None => Err(RuntimeError::VaultNotFound(vid))
                        }
                    },
                    _ => Err(RuntimeError::VaultNotFound(vid))
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
            .remove(&input.bid)
            .ok_or(RuntimeError::BucketNotFound(input.bid))?;

        self.get_local_vault(input.vid)?
            .put(bucket)
            .map_err(RuntimeError::VaultError)?;

        Ok(PutIntoVaultOutput {})
    }

    fn check_take_from_vault_auth(
        &mut self,
        vid: Vid,
        badge: Option<Address>,
    ) -> Result<(), RuntimeError> {
        let resource_address = self
            .get_local_vault(vid)?
            .resource_address();

        let resource_def = self
            .track
            .get_resource_def(resource_address)
            .ok_or(RuntimeError::ResourceDefNotFound(resource_address))?;
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
        self.check_take_from_vault_auth(input.vid.clone(), badge)?;

        let new_bucket = self
            .get_local_vault(input.vid)?
            .take(input.amount)
            .map_err(RuntimeError::VaultError)?;

        let bid = self.track.new_bid();
        self.buckets.insert(bid, new_bucket);

        Ok(TakeFromVaultOutput { bid })
    }

    fn handle_take_non_fungible_from_vault(
        &mut self,
        input: TakeNonFungibleFromVaultInput,
    ) -> Result<TakeNonFungibleFromVaultOutput, RuntimeError> {
        // TODO: restrict access

        let badge = self.check_badge(input.auth)?;
        self.check_take_from_vault_auth(input.vid.clone(), badge)?;

        let new_bucket = self
            .get_local_vault(input.vid)?
            .take_non_fungible(&input.key)
            .map_err(RuntimeError::VaultError)?;

        let bid = self.track.new_bid();
        self.buckets.insert(bid, new_bucket);

        Ok(TakeNonFungibleFromVaultOutput { bid })
    }

    fn handle_get_non_fungible_keys_in_vault(
        &mut self,
        input: GetNonFungibleKeysInVaultInput,
    ) -> Result<GetNonFungibleKeysInVaultOutput, RuntimeError> {
        let vault = self.get_local_vault(input.vid)?;
        let keys = vault.get_non_fungible_ids().map_err(RuntimeError::VaultError)?;

        Ok(GetNonFungibleKeysInVaultOutput {
            keys
        })
    }

    fn handle_get_vault_amount(
        &mut self,
        input: GetVaultDecimalInput,
    ) -> Result<GetVaultDecimalOutput, RuntimeError> {
        let vault = self.get_local_vault(input.vid)?;

        Ok(GetVaultDecimalOutput {
            amount: vault.amount(),
        })
    }

    fn handle_get_vault_resource_address(
        &mut self,
        input: GetVaultResourceAddressInput,
    ) -> Result<GetVaultResourceAddressOutput, RuntimeError> {
        let vault = self.get_local_vault(input.vid)?;

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
            .get_resource_def(input.resource_address)
            .ok_or(RuntimeError::ResourceDefNotFound(input.resource_address))?;

        let new_bucket = Bucket::new(
            input.resource_address,
            definition.resource_type(),
            match definition.resource_type() {
                ResourceType::Fungible { .. } => Supply::Fungible {
                    amount: Decimal::zero(),
                },
                ResourceType::NonFungible { .. } => Supply::NonFungible {
                    keys: BTreeSet::new(),
                },
            },
        );
        let bid = self.track.new_bid();
        self.buckets.insert(bid, new_bucket);

        Ok(CreateEmptyBucketOutput { bid })
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
            .get_mut(&input.bid)
            .ok_or(RuntimeError::BucketNotFound(input.bid))?
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
            .get_mut(&input.bid)
            .ok_or(RuntimeError::BucketNotFound(input.bid))?
            .take(input.amount)
            .map_err(RuntimeError::BucketError)?;
        let bid = self.track.new_bid();
        self.buckets.insert(bid, new_bucket);

        Ok(TakeFromBucketOutput { bid })
    }

    fn handle_get_bucket_amount(
        &mut self,
        input: GetBucketDecimalInput,
    ) -> Result<GetBucketDecimalOutput, RuntimeError> {
        let amount = self
            .buckets
            .get(&input.bid)
            .map(|b| b.amount())
            .or_else(|| {
                self.buckets_locked
                    .get(&input.bid)
                    .map(|x| x.bucket().amount())
            })
            .ok_or(RuntimeError::BucketNotFound(input.bid))?;

        Ok(GetBucketDecimalOutput { amount })
    }

    fn handle_get_bucket_resource_address(
        &mut self,
        input: GetBucketResourceAddressInput,
    ) -> Result<GetBucketResourceAddressOutput, RuntimeError> {
        let resource_address = self
            .buckets
            .get(&input.bid)
            .map(|b| b.resource_address())
            .or_else(|| {
                self.buckets_locked
                    .get(&input.bid)
                    .map(|x| x.bucket().resource_address())
            })
            .ok_or(RuntimeError::BucketNotFound(input.bid))?;

        Ok(GetBucketResourceAddressOutput { resource_address })
    }

    fn handle_take_non_fungible_from_bucket(
        &mut self,
        input: TakeNonFungibleFromBucketInput,
    ) -> Result<TakeNonFungibleFromBucketOutput, RuntimeError> {
        let new_bucket = self
            .buckets
            .get_mut(&input.bid)
            .ok_or(RuntimeError::BucketNotFound(input.bid))?
            .take_non_fungible(&input.key)
            .map_err(RuntimeError::BucketError)?;
        let bid = self.track.new_bid();
        self.buckets.insert(bid, new_bucket);

        Ok(TakeNonFungibleFromBucketOutput { bid })
    }

    fn handle_get_non_fungible_keys_in_bucket(
        &mut self,
        input: GetNonFungibleKeysInBucketInput,
    ) -> Result<GetNonFungibleKeysInBucketOutput, RuntimeError> {
        let bucket = self
            .buckets
            .get(&input.bid)
            .ok_or(RuntimeError::BucketNotFound(input.bid))?;

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
        let bid = input.bid;
        let rid = self.track.new_rid();
        re_debug!(self, "Borrowing: bid = {:?}, rid = {:?}", bid, rid);

        match self.buckets_locked.get_mut(&bid) {
            Some(bucket_rc) => {
                // re-borrow
                self.bucket_refs.insert(rid, bucket_rc.clone());
            }
            None => {
                // first time borrow
                let bucket = BucketRef::new(LockedBucket::new(
                    bid,
                    self.buckets
                        .remove(&bid)
                        .ok_or(RuntimeError::BucketNotFound(bid))?,
                ));
                self.buckets_locked.insert(bid, bucket.clone());
                self.bucket_refs.insert(rid, bucket);
            }
        }

        Ok(CreateBucketRefOutput { rid })
    }

    fn handle_drop_bucket_ref(
        &mut self,
        input: DropBucketRefInput,
    ) -> Result<DropBucketRefOutput, RuntimeError> {
        let rid = input.rid;

        let (count, bid) = {
            let bucket_ref = self
                .bucket_refs
                .remove(&rid)
                .ok_or(RuntimeError::BucketRefNotFound(rid))?;
            re_debug!(
                self,
                "Dropping bucket ref: rid = {:?}, bucket = {:?}",
                rid,
                bucket_ref
            );
            (Rc::strong_count(&bucket_ref) - 1, bucket_ref.bucket_id())
        };

        if count == 1 {
            if let Some(b) = self.buckets_locked.remove(&bid) {
                self.buckets.insert(bid, Rc::try_unwrap(b).unwrap().into());
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
            .get(&input.rid)
            .ok_or(RuntimeError::BucketRefNotFound(input.rid))?;

        Ok(GetBucketRefDecimalOutput {
            amount: bucket_ref.bucket().amount(),
        })
    }

    fn handle_get_bucket_ref_resource_def(
        &mut self,
        input: GetBucketRefResourceAddressInput,
    ) -> Result<GetBucketRefResourceAddressOutput, RuntimeError> {
        let bucket_ref = self
            .bucket_refs
            .get(&input.rid)
            .ok_or(RuntimeError::BucketRefNotFound(input.rid))?;

        Ok(GetBucketRefResourceAddressOutput {
            resource_address: bucket_ref.bucket().resource_address(),
        })
    }

    fn handle_get_non_fungible_keys_in_bucket_ref(
        &mut self,
        input: GetNonFungibleKeysInBucketRefInput,
    ) -> Result<GetNonFungibleKeysInBucketRefOutput, RuntimeError> {
        let bucket_ref = self
            .bucket_refs
            .get(&input.rid)
            .ok_or(RuntimeError::BucketRefNotFound(input.rid))?;

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
            .get(&input.rid)
            .ok_or(RuntimeError::BucketRefNotFound(input.rid))?
            .clone();

        let new_rid = self.track.new_rid();
        re_debug!(
            self,
            "Cloning: rid = {:?}, new rid = {:?}",
            input.rid,
            new_rid
        );

        self.bucket_refs.insert(new_rid, bucket_ref);
        Ok(CloneBucketRefOutput { rid: new_rid })
    }

    fn handle_emit_log(&mut self, input: EmitLogInput) -> Result<EmitLogOutput, RuntimeError> {
        self.track.add_log(input.level, input.message);

        Ok(EmitLogOutput {})
    }

    fn handle_get_package_address(
        &mut self,
        _input: GetPackageAddressInput,
    ) -> Result<GetPackageAddressOutput, RuntimeError> {
        Ok(GetPackageAddressOutput {
            package_address: self.package()?,
        })
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
                    GET_VAULT_RESOURCE_ADDRESS => {
                        self.handle(args, Self::handle_get_vault_resource_address)
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
                    GET_BUCKET_RESOURCE_ADDRESS => {
                        self.handle(args, Self::handle_get_bucket_resource_address)
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
                    GET_PACKAGE_ADDRESS => self.handle(args, Self::handle_get_package_address),
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
