use colored::*;
use sbor::any::*;
use sbor::rust::boxed::Box;
use sbor::*;
use scrypto::buffer::*;
use scrypto::kernel::*;
use scrypto::rust::borrow::ToOwned;
use scrypto::rust::collections::*;
use scrypto::rust::convert::TryFrom;
use scrypto::rust::fmt;
use scrypto::rust::format;
use scrypto::rust::rc::Rc;
use scrypto::rust::string::String;
use scrypto::rust::vec;
use scrypto::rust::vec::Vec;
use scrypto::types::*;
use wasmi::*;

use crate::engine::*;
use crate::ledger::*;
use crate::model::*;

macro_rules! trace {
    ($proc:expr, $($args: expr),+) => {
        if $proc.trace {
            $proc.log(Level::Trace, format!($($args),+));
        }
    };
}

macro_rules! debug {
    ($proc:expr, $($args: expr),+) => {
        if $proc.trace {
            $proc.log(Level::Debug, format!($($args),+));
        }
    };
}

macro_rules! info {
    ($proc:expr, $($args: expr),+) => {
        if $proc.trace {
            $proc.log(Level::Info, format!($($args),+));
        }
    };
}

macro_rules! warn {
    ($proc:expr, $($args: expr),+) => {
        if $proc.trace {
            $proc.log(Level::Warn, format!($($args),+));
        }
    };
}

/// A process keeps track of resource movements and code execution.
pub struct Process<'r, 'l, L: Ledger> {
    depth: usize,
    trace: bool,
    track: &'r mut Track<'l, L>,
    buckets: HashMap<Bid, Bucket>,
    bucket_refs: HashMap<Rid, BucketRef>,
    locked_buckets: HashMap<Bid, BucketRef>,
    moving_buckets: HashMap<Bid, Bucket>,
    moving_bucket_refs: HashMap<Rid, BucketRef>,
    temp_buckets: HashMap<Bid, Bucket>,
    temp_bucket_refs: HashMap<Rid, BucketRef>,
    reserved_bids: HashSet<Bid>,
    reserved_rids: HashSet<Rid>,
    vm: Option<Interpreter>,
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
    package: Address,
    export: String,
    function: String,
    args: Vec<Vec<u8>>,
}

impl<'r, 'l, L: Ledger> Process<'r, 'l, L> {
    /// Create a new process, which is not started.
    pub fn new(depth: usize, trace: bool, track: &'r mut Track<'l, L>) -> Self {
        Self {
            depth,
            trace,
            track,
            buckets: HashMap::new(),
            bucket_refs: HashMap::new(),
            locked_buckets: HashMap::new(),
            moving_buckets: HashMap::new(),
            moving_bucket_refs: HashMap::new(),
            temp_buckets: HashMap::new(),
            temp_bucket_refs: HashMap::new(),
            reserved_bids: HashSet::new(),
            reserved_rids: HashSet::new(),
            vm: None,
        }
    }

    /// Publishes a package.
    pub fn publish(&mut self, code: &[u8]) -> Result<Address, RuntimeError> {
        let address = self.track.new_package_address();

        if self.track.get_package(address).is_some() {
            return Err(RuntimeError::PackageAlreadyExists(address));
        }
        validate_module(code)?;

        debug!(self, "New package: {:?}", address);
        self.track
            .put_package(address, Package::new(code.to_owned()));
        Ok(address)
    }

    /// Create a resource with mutable supply.
    pub fn create_resource_mutable(
        &mut self,
        metadata: HashMap<String, String>,
        minter: Address,
    ) -> Result<Address, RuntimeError> {
        let auth = match minter {
            Address::Package(_) => minter,
            Address::Component(_) => self
                .track
                .get_component(minter)
                .ok_or(RuntimeError::ComponentNotFound(minter))?
                .package(),
            _ => {
                return Err(RuntimeError::InvalidAddressType);
            }
        };
        let resource_def = ResourceDef {
            metadata: metadata,
            minter: Some(minter),
            supply: Amount::zero(),
            auth: Some(auth),
        };

        let address = self.track.new_resource_def_address();
        if self.track.get_resource_def(address).is_some() {
            return Err(RuntimeError::ResourceDefAlreadyExists(address));
        } else {
            debug!(self, "New resource definition: {:?}", address);

            self.track.put_resource_def(address, resource_def);
        }

        Ok(address)
    }

    /// Create a resource with fixed supply.
    pub fn create_resource_fixed(
        &mut self,
        metadata: HashMap<String, String>,
        supply: Amount,
    ) -> Result<(Address, Bid), RuntimeError> {
        let resource_def = ResourceDef {
            metadata: metadata,
            minter: None,
            supply: supply,
            auth: None,
        };

        let address = self.track.new_resource_def_address();

        if self.track.get_resource_def(address).is_some() {
            return Err(RuntimeError::ResourceDefAlreadyExists(address));
        } else {
            debug!(self, "New resource definition: {:?}", address);

            self.track.put_resource_def(address, resource_def);
        }

        let bucket = Bucket::new(supply, address);
        let bid = self.track.new_bid();
        self.buckets.insert(bid, bucket);

        Ok((address, bid))
    }

    /// Mints resource.
    pub fn mint_resource(
        &mut self,
        amount: Amount,
        resource_def: Address,
    ) -> Result<Bid, RuntimeError> {
        let package = self.package()?;
        let definition = self
            .track
            .get_resource_def_mut(resource_def)
            .ok_or(RuntimeError::ResourceDefNotFound(resource_def))?;
        match definition.auth {
            Some(pkg) => {
                if package != pkg {
                    return Err(RuntimeError::UnauthorizedMint);
                }
            }
            None => {
                return Err(RuntimeError::MintFixedSupplyResource);
            }
        }
        definition.supply += amount;

        let bucket = Bucket::new(amount, resource_def);
        let bid = self.track.new_bid();
        self.buckets.insert(bid, bucket);

        Ok(bid)
    }

    /// Reserves a bucket id.
    pub fn reserve_bucket_id(&mut self) -> Bid {
        let bid = self.track.new_bid();
        self.reserved_bids.insert(bid);
        bid
    }

    /// Reserves a bucket ref id.
    pub fn reserve_bucket_ref_id(&mut self) -> Rid {
        let rid = self.track.new_rid();
        self.reserved_rids.insert(rid);
        rid
    }

    fn withdraw_resource(
        &mut self,
        amount: Amount,
        resource_def: Address,
    ) -> Result<(), RuntimeError> {
        let candidates: BTreeSet<Bid> = self
            .buckets
            .iter()
            .filter(|(_, v)| v.resource_def() == resource_def)
            .map(|(k, _)| *k)
            .collect();

        let mut needed = amount;
        for candidate in candidates {
            if needed.is_zero() {
                break;
            }
            let available = self.buckets.get(&candidate).unwrap().amount();
            if available > needed {
                debug!(self, "Withdrawing {:?} from {:?}", amount, candidate);
                self.buckets
                    .get_mut(&candidate)
                    .unwrap()
                    .take(needed)
                    .unwrap();
                needed = Amount::zero();
            } else {
                debug!(self, "Withdrawing all from {:?}", candidate);
                self.buckets.remove(&candidate);
                needed -= available;
            }
        }

        if needed.is_zero() {
            Ok(())
        } else {
            Err(RuntimeError::AccountingError(
                BucketError::InsufficientBalance,
            ))
        }
    }

    /// Creates a bucket by withdrawing resource from context.
    pub fn create_temp_bucket(
        &mut self,
        amount: Amount,
        resource_def: Address,
        bid: Bid,
    ) -> Result<(), RuntimeError> {
        debug!(
            self,
            "Creating bucket: amount = {:?}, resource_def = {:?}, bid = {:?}",
            amount,
            resource_def,
            bid
        );
        if !self.reserved_bids.remove(&bid) {
            return Err(RuntimeError::BucketNotReserved);
        }

        self.withdraw_resource(amount, resource_def)?;

        self.temp_buckets
            .insert(bid, Bucket::new(amount, resource_def));

        Ok(())
    }

    /// Creates a bucket ref by borrowing resource from context.
    pub fn create_temp_bucket_ref(
        &mut self,
        amount: Amount,
        resource_def: Address,
        rid: Rid,
    ) -> Result<(), RuntimeError> {
        debug!(
            self,
            "Creating bucket ref: amount = {:?}, resource_def = {:?}, rid = {:?}",
            amount,
            resource_def,
            rid
        );
        if !self.reserved_rids.remove(&rid) {
            return Err(RuntimeError::BucketRefNotReserved);
        }

        self.withdraw_resource(amount, resource_def)?;

        let bid = self.track.new_bid();
        let bucket = BucketRef::new(LockedBucket::new(bid, Bucket::new(amount, resource_def)));
        self.locked_buckets.insert(bid, bucket.clone());
        self.temp_bucket_refs.insert(rid, bucket);

        Ok(())
    }

    /// Puts buckets and bucket refs into this process, used for passing resources to child process.
    pub fn put_buckets_and_refs(
        &mut self,
        buckets: HashMap<Bid, Bucket>,
        bucket_refs: HashMap<Rid, BucketRef>,
    ) {
        self.buckets.extend(buckets);
        self.bucket_refs.extend(bucket_refs);
    }

    /// Takes all **moving** buckets and bucket refs from this process, used for returning resources to parent.
    pub fn take_moving_buckets_and_refs(
        &mut self,
    ) -> (HashMap<Bid, Bucket>, HashMap<Rid, BucketRef>) {
        let buckets = self.moving_buckets.drain().collect();
        let bucket_refs = self.moving_bucket_refs.drain().collect();
        (buckets, bucket_refs)
    }

    /// Returns the IDs of all owned buckets.
    pub fn owned_buckets(&mut self) -> Vec<Bid> {
        self.buckets.keys().copied().collect()
    }

    /// Runs the given export within this process.
    pub fn run(&mut self, invocation: Invocation) -> Result<Vec<u8>, RuntimeError> {
        #[cfg(not(feature = "alloc"))]
        let now = std::time::Instant::now();
        info!(
            self,
            "Run started: package = {:?}, export = {:?}", invocation.package, invocation.export
        );

        // Load the code
        let (module, memory) = self
            .track
            .load_module(invocation.package)
            .ok_or(RuntimeError::PackageNotFound(invocation.package))?;
        let vm = Interpreter {
            invocation: invocation.clone(),
            module: module.clone(),
            memory,
        };
        self.vm = Some(vm);

        // run the main function
        let result = module.invoke_export(invocation.export.as_str(), &[], self);
        debug!(self, "Invoke result: {:?}", result);
        let rtn = result
            .map_err(RuntimeError::InvokeError)?
            .ok_or(RuntimeError::NoReturnData)?;

        // move resource based on return data
        let output = match rtn {
            RuntimeValue::I32(ptr) => {
                let bytes = self.read_bytes(ptr)?;
                self.process_data(&bytes, Self::move_buckets, Self::move_bucket_refs)?;
                bytes
            }
            _ => {
                return Err(RuntimeError::InvalidReturnType);
            }
        };

        #[cfg(not(feature = "alloc"))]
        info!(
            self,
            "Run ended: time elapsed = {} ms",
            now.elapsed().as_millis()
        );
        #[cfg(feature = "alloc")]
        info!(self, "Run ended");

        Ok(output)
    }

    /// Prepares a function call.
    pub fn prepare_call_function(
        &mut self,
        package: Address,
        name: &str,
        function: &str,
        args: Vec<Vec<u8>>,
    ) -> Result<Invocation, RuntimeError> {
        Ok(Invocation {
            package,
            export: format!("{}_main", name),
            function: function.to_owned(),
            args,
        })
    }

    /// Prepares a method call.
    pub fn prepare_call_method(
        &mut self,
        component: Address,
        method: &str,
        args: Vec<Vec<u8>>,
    ) -> Result<Invocation, RuntimeError> {
        let com = self
            .track
            .get_component(component)
            .ok_or(RuntimeError::ComponentNotFound(component))?
            .clone();

        let mut self_args = vec![scrypto_encode(&component)];
        self_args.extend(args);

        self.prepare_call_function(com.package(), com.name(), method, self_args)
    }

    /// Prepares an ABI call.
    pub fn prepare_call_abi(
        &mut self,
        package: Address,
        name: &str,
    ) -> Result<Invocation, RuntimeError> {
        Ok(Invocation {
            package: package,
            export: format!("{}_abi", name),
            function: String::new(),
            args: Vec::new(),
        })
    }

    /// Calls a function/method.
    pub fn call(&mut self, invocation: Invocation) -> Result<Vec<u8>, RuntimeError> {
        // move resource
        for arg in &invocation.args {
            self.process_data(arg, Self::move_buckets, Self::move_bucket_refs)?;
        }
        let (buckets_out, bucket_refs_out) = self.take_moving_buckets_and_refs();
        let mut process = Process::new(self.depth + 1, self.trace, self.track);
        process.put_buckets_and_refs(buckets_out, bucket_refs_out);

        // run the function
        let result = process.run(invocation)?;
        process.check_resource()?;

        // move resource
        let (buckets_in, bucket_refs_in) = process.take_moving_buckets_and_refs();
        self.put_buckets_and_refs(buckets_in, bucket_refs_in);

        // scan locked buckets for some might have been unlocked by child processes
        let bids: Vec<Bid> = self
            .locked_buckets
            .values()
            .filter(|v| Rc::strong_count(v) == 1)
            .map(|v| v.bucket_id())
            .collect();
        for bid in bids {
            debug!(self, "Changing bucket {:?} to unlocked state", bid);
            let bucket_rc = self.locked_buckets.remove(&bid).unwrap();
            let bucket = Rc::try_unwrap(bucket_rc).unwrap();
            self.buckets.insert(bid, bucket.into());
        }

        Ok(result)
    }

    /// Calls a function.
    pub fn call_function(
        &mut self,
        package: Address,
        name: &str,
        function: &str,
        args: Vec<Vec<u8>>,
    ) -> Result<Vec<u8>, RuntimeError> {
        debug!(self, "Call function started");
        let invocation = self.prepare_call_function(package, name, function, args)?;
        let result = self.call(invocation);
        debug!(self, "Call function ended");
        result
    }

    /// Calls a method.
    pub fn call_method(
        &mut self,
        component: Address,
        method: &str,
        args: Vec<Vec<u8>>,
    ) -> Result<Vec<u8>, RuntimeError> {
        debug!(self, "Call method started");
        let invocation = self.prepare_call_method(component, method, args)?;
        let result = self.call(invocation);
        debug!(self, "Call method ended");
        result
    }

    /// Calls the ABI generator of a blueprint.
    pub fn call_abi(&mut self, package: Address, name: &str) -> Result<Vec<u8>, RuntimeError> {
        debug!(self, "Call abi started");
        let invocation = self.prepare_call_abi(package, name)?;
        let result = self.call(invocation);
        debug!(self, "Call abi ended");
        result
    }

    /// Checks resource leak.
    pub fn check_resource(&self) -> Result<(), RuntimeError> {
        debug!(self, "Resource check started");
        let mut success = true;

        for (bid, bucket) in &self.buckets {
            warn!(self, "Dangling bucket: {:?}, {:?}", bid, bucket);
            success = false;
        }
        for (rid, bucket_ref) in &self.bucket_refs {
            warn!(self, "Dangling bucket ref: {:?}, {:?}", rid, bucket_ref);
            success = false;
        }
        for (bid, bucket) in &self.temp_buckets {
            warn!(self, "Dangling temp bucket: {:?}, {:?}", bid, bucket);
            success = false;
        }
        for (rid, bucket_ref) in &self.temp_bucket_refs {
            warn!(
                self,
                "Dangling temp bucket ref: {:?}, {:?}", rid, bucket_ref
            );
            success = false;
        }

        debug!(self, "Resource check ended");
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

    /// Return the package address
    fn package(&self) -> Result<Address, RuntimeError> {
        self.vm
            .as_ref()
            .ok_or(RuntimeError::InterpreterNotStarted)
            .map(|vm| vm.invocation.package)
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
            .map(|vm| vm.invocation.args.clone())
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

    /// Process SBOR data by applying functions on Bid and Rid.
    fn process_data(
        &mut self,
        data: &[u8],
        bf: fn(&mut Self, Bid) -> Result<Bid, RuntimeError>,
        rf: fn(&mut Self, Rid) -> Result<Rid, RuntimeError>,
    ) -> Result<Vec<u8>, RuntimeError> {
        let value = decode_any(data).map_err(RuntimeError::InvalidData)?;
        let transformed = self.visit(value, bf, rf)?;

        let mut encoder = Encoder::with_type(Vec::with_capacity(data.len() + 512));
        encode_any(None, &transformed, &mut encoder);
        Ok(encoder.into())
    }

    // TODO: stack overflow
    fn visit(
        &mut self,
        v: Value,
        bf: fn(&mut Self, Bid) -> Result<Bid, RuntimeError>,
        rf: fn(&mut Self, Rid) -> Result<Rid, RuntimeError>,
    ) -> Result<Value, RuntimeError> {
        match v {
            // primitive types
            Value::Unit
            | Value::Bool(_)
            | Value::I8(_)
            | Value::I16(_)
            | Value::I32(_)
            | Value::I64(_)
            | Value::I128(_)
            | Value::U8(_)
            | Value::U16(_)
            | Value::U32(_)
            | Value::U64(_)
            | Value::U128(_)
            | Value::String(_) => Ok(v),
            // struct & enum
            Value::Struct(fields) => Ok(Value::Struct(self.visit_fields(fields, bf, rf)?)),
            Value::Enum(index, fields) => {
                Ok(Value::Enum(index, self.visit_fields(fields, bf, rf)?))
            }
            // composite types
            Value::Option(x) => match *x {
                Some(value) => Ok(Value::Option(Box::new(Some(self.visit(value, bf, rf)?)))),
                None => Ok(Value::Option(Box::new(None))),
            },
            Value::Box(value) => Ok(Value::Box(Box::new(self.visit(*value, bf, rf)?))),
            Value::Array(ty, values) => Ok(Value::Array(ty, self.visit_vec(values, bf, rf)?)),
            Value::Tuple(values) => Ok(Value::Tuple(self.visit_vec(values, bf, rf)?)),
            Value::Result(x) => match *x {
                Ok(value) => Ok(Value::Result(Box::new(Ok(self.visit(value, bf, rf)?)))),
                Err(value) => Ok(Value::Result(Box::new(Err(self.visit(value, bf, rf)?)))),
            },
            // collections
            Value::Vec(ty, values) => Ok(Value::Vec(ty, self.visit_vec(values, bf, rf)?)),
            Value::TreeSet(ty, values) => Ok(Value::TreeSet(ty, self.visit_vec(values, bf, rf)?)),
            Value::HashSet(ty, values) => Ok(Value::HashSet(ty, self.visit_vec(values, bf, rf)?)),
            Value::TreeMap(ty_k, ty_v, values) => {
                Ok(Value::TreeMap(ty_k, ty_v, self.visit_map(values, bf, rf)?))
            }
            Value::HashMap(ty_k, ty_v, values) => {
                Ok(Value::HashMap(ty_k, ty_v, self.visit_map(values, bf, rf)?))
            }
            // custom types
            Value::Custom(ty, data) => self.visit_custom(ty, data, bf, rf),
        }
    }

    fn visit_fields(
        &mut self,
        fields: Fields,
        bf: fn(&mut Self, Bid) -> Result<Bid, RuntimeError>,
        rf: fn(&mut Self, Rid) -> Result<Rid, RuntimeError>,
    ) -> Result<Fields, RuntimeError> {
        match fields {
            Fields::Named(named) => Ok(Fields::Named(self.visit_vec(named, bf, rf)?)),
            Fields::Unnamed(unnamed) => Ok(Fields::Unnamed(self.visit_vec(unnamed, bf, rf)?)),
            Fields::Unit => Ok(Fields::Unit),
        }
    }

    fn visit_vec(
        &mut self,
        values: Vec<Value>,
        bf: fn(&mut Self, Bid) -> Result<Bid, RuntimeError>,
        rf: fn(&mut Self, Rid) -> Result<Rid, RuntimeError>,
    ) -> Result<Vec<Value>, RuntimeError> {
        let mut result = Vec::new();
        for e in values {
            result.push(self.visit(e, bf, rf)?);
        }
        Ok(result)
    }

    fn visit_map(
        &mut self,
        values: Vec<(Value, Value)>,
        bf: fn(&mut Self, Bid) -> Result<Bid, RuntimeError>,
        rf: fn(&mut Self, Rid) -> Result<Rid, RuntimeError>,
    ) -> Result<Vec<(Value, Value)>, RuntimeError> {
        let mut result = Vec::new();
        for (k, v) in values {
            result.push((self.visit(k, bf, rf)?, self.visit(v, bf, rf)?));
        }
        Ok(result)
    }

    fn visit_custom(
        &mut self,
        ty: u8,
        data: Vec<u8>,
        bf: fn(&mut Self, Bid) -> Result<Bid, RuntimeError>,
        rf: fn(&mut Self, Rid) -> Result<Rid, RuntimeError>,
    ) -> Result<Value, RuntimeError> {
        match ty {
            SCRYPTO_TYPE_BID => {
                let bid = bf(
                    self,
                    Bid::try_from(data.as_slice()).map_err(|_| {
                        RuntimeError::InvalidData(DecodeError::InvalidCustomData(ty))
                    })?,
                )?;
                Ok(Value::Custom(ty, bid.to_vec()))
            }
            SCRYPTO_TYPE_RID => {
                let rid = rf(
                    self,
                    Rid::try_from(data.as_slice()).map_err(|_| {
                        RuntimeError::InvalidData(DecodeError::InvalidCustomData(ty))
                    })?,
                )?;
                Ok(Value::Custom(ty, rid.to_vec()))
            }
            _ => Ok(Value::Custom(ty, data)),
        }
    }

    /// Remove transient buckets from this process
    fn move_buckets(&mut self, bid: Bid) -> Result<Bid, RuntimeError> {
        let bucket = self
            .buckets
            .remove(&bid)
            .or_else(|| self.temp_buckets.remove(&bid))
            .ok_or(RuntimeError::BucketNotFound(bid))?;
        debug!(self, "Moving bucket: {:?}, {:?}", bid, bucket);
        self.moving_buckets.insert(bid, bucket);
        Ok(bid)
    }

    /// Remove transient buckets from this process
    fn move_bucket_refs(&mut self, rid: Rid) -> Result<Rid, RuntimeError> {
        let bucket_ref = self
            .bucket_refs
            .remove(&rid)
            .or_else(|| self.temp_bucket_refs.remove(&rid))
            .ok_or(RuntimeError::BucketRefNotFound(rid))?;
        debug!(self, "Moving bucket ref: {:?}, {:?}", rid, bucket_ref);
        self.moving_bucket_refs.insert(rid, bucket_ref);
        Ok(rid)
    }

    /// Reject buckets
    fn reject_buckets(&mut self, _: Bid) -> Result<Bid, RuntimeError> {
        Err(RuntimeError::BucketNotAllowed)
    }

    /// Reject bucket refs
    fn reject_bucket_refs(&mut self, _: Rid) -> Result<Rid, RuntimeError> {
        Err(RuntimeError::BucketRefNotAllowed)
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

    /// Handle a kernel call.
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
            trace!(self, "{:?}", input);
        } else {
            trace!(self, "Large request: op = {:02x}, len = {}", op, input_len);
        }

        let output: O = handler(self, input).map_err(Trap::from)?;
        let output_bytes = scrypto_encode(&output);
        let output_ptr = self.send_bytes(&output_bytes).map_err(Trap::from)?;
        if output_bytes.len() <= 1024 {
            trace!(self, "{:?}", output);
        } else {
            trace!(
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
        Ok(PublishPackageOutput {
            package: self.publish(&input.code)?,
        })
    }

    fn handle_call_function(
        &mut self,
        input: CallFunctionInput,
    ) -> Result<CallFunctionOutput, RuntimeError> {
        debug!(
            self,
            "CALL started: package = {:?}, name = {:?}, function = {:?}, args = {:?}",
            input.package,
            input.name,
            input.function,
            input.args
        );

        let invocation = self.prepare_call_function(
            input.package,
            &input.name,
            input.function.as_str(),
            input.args,
        )?;
        let result = self.call(invocation);

        debug!(self, "CALL finished");
        Ok(CallFunctionOutput { rtn: result? })
    }

    fn handle_call_method(
        &mut self,
        input: CallMethodInput,
    ) -> Result<CallMethodOutput, RuntimeError> {
        debug!(
            self,
            "CALL started: component = {:?}, method = {:?}, args = {:?}",
            input.component,
            input.method,
            input.args
        );

        let invocation =
            self.prepare_call_method(input.component, input.method.as_str(), input.args)?;
        let result = self.call(invocation);

        debug!(self, "CALL finished");
        Ok(CallMethodOutput { rtn: result? })
    }

    fn handle_create_component(
        &mut self,
        input: CreateComponentInput,
    ) -> Result<CreateComponentOutput, RuntimeError> {
        let address = self.track.new_component_address();

        if self.track.get_component(address).is_some() {
            return Err(RuntimeError::ComponentAlreadyExists(address));
        }

        let new_state =
            self.process_data(&input.state, Self::reject_buckets, Self::reject_bucket_refs)?;
        debug!(
            self,
            "New component: address = {:?}, state = {:?}", address, new_state
        );

        let component = Component::new(self.package()?, input.name, new_state);
        self.track.put_component(address, component);

        Ok(CreateComponentOutput { component: address })
    }

    fn handle_get_component_blueprint(
        &mut self,
        input: GetComponentBlueprintInput,
    ) -> Result<GetComponentBlueprintOutput, RuntimeError> {
        let package = self.package()?;

        let component = self
            .track
            .get_component(input.component)
            .ok_or(RuntimeError::ComponentNotFound(input.component))?;
        if package != component.package() {
            return Err(RuntimeError::UnauthorizedAccess);
        }

        Ok(GetComponentBlueprintOutput {
            package: component.package(),
            name: component.name().to_owned(),
        })
    }

    fn handle_get_component_state(
        &mut self,
        input: GetComponentStateInput,
    ) -> Result<GetComponentStateOutput, RuntimeError> {
        let package = self.package()?;

        let component = self
            .track
            .get_component(input.component)
            .ok_or(RuntimeError::ComponentNotFound(input.component))?;
        if package != component.package() {
            return Err(RuntimeError::UnauthorizedAccess);
        }

        let state = component.state();

        Ok(GetComponentStateOutput {
            state: state.to_owned(),
        })
    }

    fn handle_put_component_state(
        &mut self,
        input: PutComponentStateInput,
    ) -> Result<PutComponentStateOutput, RuntimeError> {
        let package = self.package()?;

        let new_state =
            self.process_data(&input.state, Self::reject_buckets, Self::reject_bucket_refs)?;
        debug!(self, "Transformed state: {:?}", new_state);

        let component = self
            .track
            .get_component_mut(input.component)
            .ok_or(RuntimeError::ComponentNotFound(input.component))?;
        if package != component.package() {
            return Err(RuntimeError::UnauthorizedAccess);
        }

        component.set_state(new_state);

        Ok(PutComponentStateOutput {})
    }

    fn handle_create_lazy_map(
        &mut self,
        _input: CreateLazyMapInput,
    ) -> Result<CreateLazyMapOutput, RuntimeError> {
        let mid = self.track.new_mid();

        if self.track.get_lazy_map(mid).is_some() {
            return Err(RuntimeError::LazyMapAlreadyExists(mid));
        }

        self.track.put_lazy_map(mid, LazyMap::new(self.package()?));

        Ok(CreateLazyMapOutput { lazy_map: mid })
    }

    fn handle_get_lazy_map_entry(
        &mut self,
        input: GetLazyMapEntryInput,
    ) -> Result<GetLazyMapEntryOutput, RuntimeError> {
        let package = self.package()?;

        let lazy_map = self
            .track
            .get_lazy_map(input.lazy_map)
            .ok_or(RuntimeError::LazyMapNotFound(input.lazy_map))?;
        if package != lazy_map.auth() {
            return Err(RuntimeError::UnauthorizedAccess);
        }

        Ok(GetLazyMapEntryOutput {
            value: lazy_map.get_entry(&input.key).map(|e| e.to_vec()),
        })
    }

    fn handle_put_lazy_map_entry(
        &mut self,
        input: PutLazyMapEntryInput,
    ) -> Result<PutLazyMapEntryOutput, RuntimeError> {
        let package = self.package()?;

        let new_key =
            self.process_data(&input.key, Self::reject_buckets, Self::reject_bucket_refs)?;
        debug!(self, "Transformed key: {:?}", new_key);
        let new_value =
            self.process_data(&input.value, Self::reject_buckets, Self::reject_bucket_refs)?;
        debug!(self, "Transformed value: {:?}", new_value);

        let lazy_map = self
            .track
            .get_lazy_map_mut(input.lazy_map)
            .ok_or(RuntimeError::LazyMapNotFound(input.lazy_map))?;
        if package != lazy_map.auth() {
            return Err(RuntimeError::UnauthorizedAccess);
        }

        lazy_map.set_entry(new_key, new_value);

        Ok(PutLazyMapEntryOutput {})
    }

    fn handle_create_resource_mutable(
        &mut self,
        input: CreateResourceMutableInput,
    ) -> Result<CreateResourceMutableOutput, RuntimeError> {
        Ok(CreateResourceMutableOutput {
            resource_def: self.create_resource_mutable(input.metadata, input.minter)?,
        })
    }

    fn handle_create_resource_fixed(
        &mut self,
        input: CreateResourceFixedInput,
    ) -> Result<CreateResourceFixedOutput, RuntimeError> {
        let (resource_def, bucket) = self.create_resource_fixed(input.metadata, input.supply)?;
        Ok(CreateResourceFixedOutput {
            resource_def,
            bucket,
        })
    }

    fn handle_get_resource_metadata(
        &mut self,
        input: GetResourceMetadataInput,
    ) -> Result<GetResourceMetadataOutput, RuntimeError> {
        let resource_def = self
            .track
            .get_resource_def(input.resource_def)
            .ok_or(RuntimeError::ResourceDefNotFound(input.resource_def))?
            .clone();

        Ok(GetResourceMetadataOutput {
            metadata: resource_def.metadata,
        })
    }

    fn handle_get_resource_supply(
        &mut self,
        input: GetResourceSupplyInput,
    ) -> Result<GetResourceSupplyOutput, RuntimeError> {
        let resource_def = self
            .track
            .get_resource_def(input.resource_def)
            .ok_or(RuntimeError::ResourceDefNotFound(input.resource_def))?
            .clone();

        Ok(GetResourceSupplyOutput {
            supply: resource_def.supply,
        })
    }

    fn handle_get_resource_minter(
        &mut self,
        input: GetResourceMinterInput,
    ) -> Result<GetResourceMinterOutput, RuntimeError> {
        let resource_def = self
            .track
            .get_resource_def(input.resource_def)
            .ok_or(RuntimeError::ResourceDefNotFound(input.resource_def))?
            .clone();

        Ok(GetResourceMinterOutput {
            minter: resource_def.minter,
        })
    }

    fn handle_mint_resource(
        &mut self,
        input: MintResourceInput,
    ) -> Result<MintResourceOutput, RuntimeError> {
        Ok(MintResourceOutput {
            bucket: self.mint_resource(input.amount, input.resource_def)?,
        })
    }

    fn handle_burn_resource(
        &mut self,
        input: BurnResourceInput,
    ) -> Result<BurnResourceOutput, RuntimeError> {
        let bucket = self
            .buckets
            .remove(&input.bucket)
            .ok_or(RuntimeError::BucketNotFound(input.bucket))?;

        let resource_def = self
            .track
            .get_resource_def_mut(bucket.resource_def())
            .ok_or(RuntimeError::ResourceDefNotFound(bucket.resource_def()))?;

        resource_def.supply -= bucket.amount();

        Ok(BurnResourceOutput {})
    }

    fn handle_create_vault(
        &mut self,
        input: CreateEmptyVaultInput,
    ) -> Result<CreateEmptyVaultOutput, RuntimeError> {
        let package = self.package()?;

        let new_vault = Vault::new(Bucket::new(Amount::zero(), input.resource_def), package);
        let new_vid = self.track.new_vid();
        self.track.put_vault(new_vid, new_vault);

        Ok(CreateEmptyVaultOutput { vault: new_vid })
    }

    fn handle_put_into_vault(
        &mut self,
        input: PutIntoVaultInput,
    ) -> Result<PutIntoVaultOutput, RuntimeError> {
        let package = self.package()?;

        let other = self
            .buckets
            .remove(&input.bucket)
            .ok_or(RuntimeError::BucketNotFound(input.bucket))?;

        self.track
            .get_vault_mut(input.vault)
            .ok_or(RuntimeError::VaultNotFound(input.vault))?
            .put(other, package)
            .map_err(RuntimeError::AccountingError)?;

        Ok(PutIntoVaultOutput {})
    }

    fn handle_take_from_vault(
        &mut self,
        input: TakeFromVaultInput,
    ) -> Result<TakeFromVaultOutput, RuntimeError> {
        let package = self.package()?;

        let new_bucket = self
            .track
            .get_vault_mut(input.vault)
            .ok_or(RuntimeError::VaultNotFound(input.vault))?
            .take(input.amount, package)
            .map_err(RuntimeError::AccountingError)?;

        let new_bid = self.track.new_bid();
        self.buckets.insert(new_bid, new_bucket);

        Ok(TakeFromVaultOutput { bucket: new_bid })
    }

    fn handle_get_vault_amount(
        &mut self,
        input: GetVaultAmountInput,
    ) -> Result<GetVaultAmountOutput, RuntimeError> {
        let amount = self
            .track
            .get_vault(input.vault)
            .map(|b| b.amount())
            .ok_or(RuntimeError::VaultNotFound(input.vault))?;

        Ok(GetVaultAmountOutput { amount })
    }

    fn handle_get_vault_resource_def(
        &mut self,
        input: GetVaultResourceAddressInput,
    ) -> Result<GetVaultResourceAddressOutput, RuntimeError> {
        let resource_def = self
            .track
            .get_vault(input.vault)
            .map(|b| b.resource_def())
            .ok_or(RuntimeError::VaultNotFound(input.vault))?;

        Ok(GetVaultResourceAddressOutput { resource_def })
    }

    fn handle_create_bucket(
        &mut self,
        input: CreateEmptyBucketInput,
    ) -> Result<CreateEmptyBucketOutput, RuntimeError> {
        let new_bucket = Bucket::new(Amount::zero(), input.resource_def);
        let new_bid = self.track.new_bid();
        self.buckets.insert(new_bid, new_bucket);

        Ok(CreateEmptyBucketOutput { bucket: new_bid })
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
            .get_mut(&input.bucket)
            .ok_or(RuntimeError::BucketNotFound(input.bucket))?
            .put(other)
            .map_err(RuntimeError::AccountingError)?;

        Ok(PutIntoBucketOutput {})
    }

    fn handle_take_from_bucket(
        &mut self,
        input: TakeFromBucketInput,
    ) -> Result<TakeFromBucketOutput, RuntimeError> {
        let new_bucket = self
            .buckets
            .get_mut(&input.bucket)
            .ok_or(RuntimeError::BucketNotFound(input.bucket))?
            .take(input.amount)
            .map_err(RuntimeError::AccountingError)?;
        let new_bid = self.track.new_bid();
        self.buckets.insert(new_bid, new_bucket);

        Ok(TakeFromBucketOutput { bucket: new_bid })
    }

    fn handle_get_bucket_amount(
        &mut self,
        input: GetBucketAmountInput,
    ) -> Result<GetBucketAmountOutput, RuntimeError> {
        let bid = input.bucket;
        let amount = self
            .buckets
            .get(&bid)
            .map(|b| b.amount())
            .or_else(|| self.locked_buckets.get(&bid).map(|x| x.bucket().amount()))
            .ok_or(RuntimeError::BucketNotFound(bid))?;

        Ok(GetBucketAmountOutput { amount })
    }

    fn handle_get_bucket_resource_def(
        &mut self,
        input: GetBucketResourceAddressInput,
    ) -> Result<GetBucketResourceAddressOutput, RuntimeError> {
        let bid = input.bucket;
        let resource_def = self
            .buckets
            .get(&bid)
            .map(|b| b.resource_def())
            .or_else(|| {
                self.locked_buckets
                    .get(&bid)
                    .map(|x| x.bucket().resource_def())
            })
            .ok_or(RuntimeError::BucketNotFound(bid))?;

        Ok(GetBucketResourceAddressOutput { resource_def })
    }

    fn handle_create_bucket_ref(
        &mut self,
        input: CreateBucketRefInput,
    ) -> Result<CreateBucketRefOutput, RuntimeError> {
        let bid = input.bucket;
        let rid = self.track.new_rid();
        debug!(self, "Borrowing: bid = {:?}, rid = {:?}", bid, rid);

        match self.locked_buckets.get_mut(&bid) {
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
                self.locked_buckets.insert(bid, bucket.clone());
                self.bucket_refs.insert(rid, bucket);
            }
        }

        Ok(CreateBucketRefOutput { bucket_ref: rid })
    }

    fn handle_drop_bucket_ref(
        &mut self,
        input: DropBucketRefInput,
    ) -> Result<DropBucketRefOutput, RuntimeError> {
        let rid = input.bucket_ref;

        let (count, bid) = {
            let bucket = self
                .bucket_refs
                .remove(&rid)
                .ok_or(RuntimeError::BucketRefNotFound(rid))?;
            debug!(self, "Returning {:?}: {:?}", rid, bucket);
            (Rc::strong_count(&bucket) - 1, bucket.bucket_id())
        };

        if count == 1 {
            if let Some(b) = self.locked_buckets.remove(&bid) {
                self.buckets.insert(bid, Rc::try_unwrap(b).unwrap().into());
            }
        }

        Ok(DropBucketRefOutput {})
    }

    fn handle_get_bucket_ref_amount(
        &mut self,
        input: GetBucketRefAmountInput,
    ) -> Result<GetBucketRefAmountOutput, RuntimeError> {
        let bucket_ref = self
            .bucket_refs
            .get(&input.bucket_ref)
            .ok_or(RuntimeError::BucketRefNotFound(input.bucket_ref))?;

        Ok(GetBucketRefAmountOutput {
            amount: bucket_ref.bucket().amount(),
        })
    }

    fn handle_get_bucket_ref_resource_def(
        &mut self,
        input: GetBucketRefResourceDefInput,
    ) -> Result<GetBucketRefResourceDefOutput, RuntimeError> {
        let bucket_ref = self
            .bucket_refs
            .get(&input.bucket_ref)
            .ok_or(RuntimeError::BucketRefNotFound(input.bucket_ref))?;

        Ok(GetBucketRefResourceDefOutput {
            resource_def: bucket_ref.bucket().resource_def(),
        })
    }

    fn handle_emit_log(&mut self, input: EmitLogInput) -> Result<EmitLogOutput, RuntimeError> {
        let level = match input.level {
            0 => Ok(Level::Error),
            1 => Ok(Level::Warn),
            2 => Ok(Level::Info),
            3 => Ok(Level::Debug),
            4 => Ok(Level::Trace),
            _ => Err(RuntimeError::InvalidLogLevel),
        };

        self.track.add_log(level?, input.message);

        Ok(EmitLogOutput {})
    }

    fn handle_get_package_address(
        &mut self,
        _input: GetPackageAddressInput,
    ) -> Result<GetPackageAddressOutput, RuntimeError> {
        Ok(GetPackageAddressOutput {
            address: self.package()?,
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
            tx_hash: self.track.tx_hash(),
        })
    }

    //============================
    // SYSTEM CALL HANDLERS END
    //============================
}

impl<'r, 'l, L: Ledger> Externals for Process<'r, 'l, L> {
    fn invoke_index(
        &mut self,
        index: usize,
        args: RuntimeArgs,
    ) -> Result<Option<RuntimeValue>, Trap> {
        match index {
            KERNEL_INDEX => {
                let operation: u32 = args.nth_checked(0)?;
                match operation {
                    PUBLISH_PACKAGE => self.handle(args, Self::handle_publish),
                    CALL_FUNCTION => self.handle(args, Self::handle_call_function),
                    CALL_METHOD => self.handle(args, Self::handle_call_method),

                    CREATE_COMPONENT => self.handle(args, Self::handle_create_component),
                    GET_COMPONENT_BLUEPRINT => {
                        self.handle(args, Self::handle_get_component_blueprint)
                    }
                    GET_COMPONENT_STATE => self.handle(args, Self::handle_get_component_state),
                    PUT_COMPONENT_STATE => self.handle(args, Self::handle_put_component_state),

                    CREATE_LAZY_MAP => self.handle(args, Self::handle_create_lazy_map),
                    GET_LAZY_MAP_ENTRY => self.handle(args, Self::handle_get_lazy_map_entry),
                    PUT_LAZY_MAP_ENTRY => self.handle(args, Self::handle_put_lazy_map_entry),

                    CREATE_RESOURCE_MUTABLE => {
                        self.handle(args, Self::handle_create_resource_mutable)
                    }
                    CREATE_RESOURCE_FIXED => self.handle(args, Self::handle_create_resource_fixed),
                    GET_RESOURCE_METADATA => self.handle(args, Self::handle_get_resource_metadata),
                    GET_RESOURCE_SUPPLY => self.handle(args, Self::handle_get_resource_supply),
                    GET_RESOURCE_MINTER => self.handle(args, Self::handle_get_resource_minter),
                    MINT_RESOURCE => self.handle(args, Self::handle_mint_resource),
                    BURN_RESOURCE => self.handle(args, Self::handle_burn_resource),

                    CREATE_EMPTY_VAULT => self.handle(args, Self::handle_create_vault),
                    PUT_INTO_VAULT => self.handle(args, Self::handle_put_into_vault),
                    TAKE_FROM_VAULT => self.handle(args, Self::handle_take_from_vault),
                    GET_VAULT_AMOUNT => self.handle(args, Self::handle_get_vault_amount),
                    GET_VAULT_RESOURCE_DEF => {
                        self.handle(args, Self::handle_get_vault_resource_def)
                    }

                    CREATE_EMPTY_BUCKET => self.handle(args, Self::handle_create_bucket),
                    PUT_INTO_BUCKET => self.handle(args, Self::handle_put_into_bucket),
                    TAKE_FROM_BUCKET => self.handle(args, Self::handle_take_from_bucket),
                    GET_BUCKET_AMOUNT => self.handle(args, Self::handle_get_bucket_amount),
                    GET_BUCKET_RESOURCE_DEF => {
                        self.handle(args, Self::handle_get_bucket_resource_def)
                    }

                    CREATE_BUCKET_REF => self.handle(args, Self::handle_create_bucket_ref),
                    DROP_BUCKET_REF => self.handle(args, Self::handle_drop_bucket_ref),
                    GET_BUCKET_REF_AMOUNT => self.handle(args, Self::handle_get_bucket_ref_amount),
                    GET_BUCKET_REF_RESOURCE_DEF => {
                        self.handle(args, Self::handle_get_bucket_ref_resource_def)
                    }

                    EMIT_LOG => self.handle(args, Self::handle_emit_log),
                    GET_PACKAGE_ADDRESS => self.handle(args, Self::handle_get_package_address),
                    GET_CALL_DATA => self.handle(args, Self::handle_get_call_data),
                    GET_TRANSACTION_HASH => self.handle(args, Self::handle_get_transaction_hash),

                    _ => Err(RuntimeError::InvalidRequestCode(operation).into()),
                }
            }
            _ => Err(RuntimeError::HostFunctionNotFound(index).into()),
        }
    }
}
