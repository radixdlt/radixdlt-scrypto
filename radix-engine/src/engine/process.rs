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
            $proc.log(LogLevel::Trace, format!($($args),+));
        }
    };
}

macro_rules! debug {
    ($proc:expr, $($args: expr),+) => {
        if $proc.trace {
            $proc.log(LogLevel::Debug, format!($($args),+));
        }
    };
}

macro_rules! info {
    ($proc:expr, $($args: expr),+) => {
        if $proc.trace {
            $proc.log(LogLevel::Info, format!($($args),+));
        }
    };
}

macro_rules! warn {
    ($proc:expr, $($args: expr),+) => {
        if $proc.trace {
            $proc.log(LogLevel::Warn, format!($($args),+));
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
    package_address: Address,
    export_name: String,
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

    /// Reserves a BID.
    pub fn declare_bucket(&mut self) -> Bid {
        let bid = self.track.new_bid();
        self.reserved_bids.insert(bid);
        bid
    }

    /// Reserves a RID.
    pub fn declare_bucket_ref(&mut self) -> Rid {
        let rid = self.track.new_rid();
        self.reserved_rids.insert(rid);
        rid
    }

    fn withdraw_resource(
        &mut self,
        amount: Decimal,
        resource_address: Address,
    ) -> Result<Bucket, RuntimeError> {
        let definition = self
            .track
            .get_resource_def(resource_address)
            .ok_or(RuntimeError::ResourceDefNotFound(resource_address))?;

        let candidates: BTreeSet<Bid> = self
            .buckets
            .iter()
            .filter(|(_, v)| v.resource_address() == resource_address)
            .map(|(k, _)| *k)
            .collect();

        let mut collector = Bucket::new(
            resource_address,
            definition.resource_type(),
            match definition.resource_type() {
                ResourceType::Fungible { .. } => Supply::Fungible { amount: 0.into() },
                ResourceType::NonFungible { .. } => Supply::NonFungible {
                    ids: BTreeSet::new(),
                },
            },
        );
        let mut needed = amount;
        for candidate in candidates {
            if needed.is_zero() {
                break;
            }
            let available = self.buckets.get(&candidate).unwrap().amount();
            if available > needed {
                debug!(self, "Withdrawing {:?} from {:?}", needed, candidate);
                collector
                    .put(
                        self.buckets
                            .get_mut(&candidate)
                            .unwrap()
                            .take(needed)
                            .unwrap(),
                    )
                    .map_err(RuntimeError::BucketError)?;
                needed = Decimal::zero();
            } else {
                debug!(self, "Withdrawing all from {:?}", candidate);
                collector
                    .put(self.buckets.remove(&candidate).unwrap())
                    .map_err(RuntimeError::BucketError)?;
                needed -= available;
            }
        }

        if needed.is_zero() {
            Ok(collector)
        } else {
            Err(RuntimeError::BucketError(BucketError::InsufficientBalance))
        }
    }

    /// Takes resource from this context to a temporary bucket.
    pub fn take_from_context(
        &mut self,
        amount: Decimal,
        resource_address: Address,
        bid: Bid,
    ) -> Result<(), RuntimeError> {
        debug!(
            self,
            "Creating bucket: amount = {:?}, resource_address = {:?}, bid = {:?}",
            amount,
            resource_address,
            bid
        );
        if !self.reserved_bids.remove(&bid) {
            return Err(RuntimeError::BucketNotReserved);
        }
        let bucket = self.withdraw_resource(amount, resource_address)?;
        self.temp_buckets.insert(bid, bucket);

        Ok(())
    }

    /// Borrows resource from this context to a temporary bucket ref.
    ///
    /// A bucket will be created to support the reference.
    pub fn borrow_from_context(
        &mut self,
        amount: Decimal,
        resource_address: Address,
        rid: Rid,
    ) -> Result<(), RuntimeError> {
        debug!(
            self,
            "Creating bucket ref: amount = {:?}, resource_def = {:?}, rid = {:?}",
            amount,
            resource_address,
            rid
        );
        if !self.reserved_rids.remove(&rid) {
            return Err(RuntimeError::BucketRefNotReserved);
        }
        let bid = self.track.new_bid();
        let bucket = BucketRef::new(LockedBucket::new(
            bid,
            self.withdraw_resource(amount, resource_address)?,
        ));
        self.locked_buckets.insert(bid, bucket.clone());
        self.temp_bucket_refs.insert(rid, bucket);

        Ok(())
    }

    /// Puts buckets and bucket refs into this process.
    pub fn put_resources(
        &mut self,
        buckets: HashMap<Bid, Bucket>,
        bucket_refs: HashMap<Rid, BucketRef>,
    ) {
        self.buckets.extend(buckets);
        self.bucket_refs.extend(bucket_refs);
    }

    /// Takes all **moving** buckets and bucket refs from this process.
    pub fn take_moving_resources(&mut self) -> (HashMap<Bid, Bucket>, HashMap<Rid, BucketRef>) {
        let buckets = self.moving_buckets.drain().collect();
        let bucket_refs = self.moving_bucket_refs.drain().collect();
        (buckets, bucket_refs)
    }

    /// Returns all bucket ids.
    pub fn list_buckets(&mut self) -> Vec<Bid> {
        self.buckets.keys().copied().collect()
    }

    /// Returns all bucket ids.
    pub fn drop_bucket_refs(&mut self) {
        let rids: Vec<Rid> = self.bucket_refs.keys().copied().collect();

        for rid in rids {
            self.handle_drop_bucket_ref(DropBucketRefInput { rid })
                .unwrap();
        }
    }

    /// Runs the given export within this process.
    pub fn run(&mut self, invocation: Invocation) -> Result<Vec<u8>, RuntimeError> {
        #[cfg(not(feature = "alloc"))]
        let now = std::time::Instant::now();
        info!(
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
        package_address: Address,
        blueprint_name: &str,
        function: &str,
        args: Vec<Vec<u8>>,
    ) -> Result<Invocation, RuntimeError> {
        Ok(Invocation {
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
        args: Vec<Vec<u8>>,
    ) -> Result<Invocation, RuntimeError> {
        let component = self
            .track
            .get_component(component_address)
            .ok_or(RuntimeError::ComponentNotFound(component_address))?
            .clone();

        let mut self_args = vec![scrypto_encode(&component_address)];
        self_args.extend(args);

        self.prepare_call_function(
            component.package_address(),
            component.blueprint_name(),
            method,
            self_args,
        )
    }

    /// Prepares an ABI call.
    pub fn prepare_call_abi(
        &mut self,
        package_address: Address,
        blueprint_name: &str,
    ) -> Result<Invocation, RuntimeError> {
        Ok(Invocation {
            package_address: package_address,
            export_name: format!("{}_abi", blueprint_name),
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
        let (buckets_out, bucket_refs_out) = self.take_moving_resources();
        let mut process = Process::new(self.depth + 1, self.trace, self.track);
        process.put_resources(buckets_out, bucket_refs_out);

        // run the function
        let result = process.run(invocation)?;
        process.check_resource()?;

        // move resource
        let (buckets_in, bucket_refs_in) = process.take_moving_resources();
        self.put_resources(buckets_in, bucket_refs_in);

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
        package_address: Address,
        blueprint_name: &str,
        function: &str,
        args: Vec<Vec<u8>>,
    ) -> Result<Vec<u8>, RuntimeError> {
        debug!(self, "Call function started");
        let invocation =
            self.prepare_call_function(package_address, blueprint_name, function, args)?;
        let result = self.call(invocation);
        debug!(self, "Call function ended");
        result
    }

    /// Calls a method.
    pub fn call_method(
        &mut self,
        component_address: Address,
        method: &str,
        args: Vec<Vec<u8>>,
    ) -> Result<Vec<u8>, RuntimeError> {
        debug!(self, "Call method started");
        let invocation = self.prepare_call_method(component_address, method, args)?;
        let result = self.call(invocation);
        debug!(self, "Call method ended");
        result
    }

    /// Calls the ABI generator of a blueprint.
    pub fn call_abi(
        &mut self,
        package_address: Address,
        blueprint_name: &str,
    ) -> Result<Vec<u8>, RuntimeError> {
        debug!(self, "Call abi started");
        let invocation = self.prepare_call_abi(package_address, blueprint_name)?;
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
            SCRYPTO_TYPE_DECIMAL
            | SCRYPTO_TYPE_BIG_DECIMAL
            | SCRYPTO_TYPE_ADDRESS
            | SCRYPTO_TYPE_H256
            | SCRYPTO_TYPE_MID
            | SCRYPTO_TYPE_VID => Ok(Value::Custom(ty, data)),
            _ => Err(RuntimeError::InvalidData(DecodeError::InvalidCustomData(
                ty,
            ))),
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

    fn authenticate(&self) -> Result<Actor, RuntimeError> {
        Ok(Actor::Package(self.package()?))
    }

    fn authenticate_with_badge(
        &mut self,
        optional_rid: Option<Rid>,
    ) -> Result<Actor, RuntimeError> {
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

            let mut set = HashSet::new();
            set.insert(resource_address);
            Ok(Actor::PackageWithBadges(self.package()?, set))
        } else {
            Ok(Actor::PackageWithBadges(self.package()?, HashSet::new()))
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
        validate_module(&input.code)?;

        debug!(self, "New package: {:?}", package_address);
        self.track
            .put_package(package_address, Package::new(input.code));

        Ok(PublishPackageOutput { package_address })
    }

    fn handle_call_function(
        &mut self,
        input: CallFunctionInput,
    ) -> Result<CallFunctionOutput, RuntimeError> {
        Self::expect_package_address(input.package_address)?;

        debug!(
            self,
            "CALL started: package = {:?}, blueprint = {:?}, function = {:?}, args = {:?}",
            input.package_address,
            input.blueprint_name,
            input.function,
            input.args
        );

        let invocation = self.prepare_call_function(
            input.package_address,
            &input.blueprint_name,
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
        Self::expect_component_address(input.component_address)?;

        debug!(
            self,
            "CALL started: component = {:?}, method = {:?}, args = {:?}",
            input.component_address,
            input.method,
            input.args
        );

        let invocation =
            self.prepare_call_method(input.component_address, input.method.as_str(), input.args)?;
        let result = self.call(invocation);

        debug!(self, "CALL finished");
        Ok(CallMethodOutput { rtn: result? })
    }

    fn handle_create_component(
        &mut self,
        input: CreateComponentInput,
    ) -> Result<CreateComponentOutput, RuntimeError> {
        let component_address = self.track.new_component_address();

        if self.track.get_component(component_address).is_some() {
            return Err(RuntimeError::ComponentAlreadyExists(component_address));
        }

        let new_state =
            self.process_data(&input.state, Self::reject_buckets, Self::reject_bucket_refs)?;
        debug!(
            self,
            "New component: address = {:?}, state = {:?}", component_address, new_state
        );

        let component = Component::new(self.package()?, input.blueprint_name, new_state);
        self.track.put_component(component_address, component);

        Ok(CreateComponentOutput { component_address })
    }

    fn handle_get_component_info(
        &mut self,
        input: GetComponentInfoInput,
    ) -> Result<GetComponentInfoOutput, RuntimeError> {
        Self::expect_component_address(input.component_address)?;

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
        let actor = self.authenticate()?;

        let component = self
            .track
            .get_component(input.component_address)
            .ok_or(RuntimeError::ComponentNotFound(input.component_address))?;

        let state = component
            .state(actor)
            .map_err(RuntimeError::ComponentError)?;

        Ok(GetComponentStateOutput {
            state: state.to_owned(),
        })
    }

    fn handle_put_component_state(
        &mut self,
        input: PutComponentStateInput,
    ) -> Result<PutComponentStateOutput, RuntimeError> {
        Self::expect_component_address(input.component_address)?;
        let actor = self.authenticate()?;

        let new_state =
            self.process_data(&input.state, Self::reject_buckets, Self::reject_bucket_refs)?;
        debug!(self, "Transformed state: {:?}", new_state);

        let component = self
            .track
            .get_component_mut(input.component_address)
            .ok_or(RuntimeError::ComponentNotFound(input.component_address))?;

        component
            .set_state(new_state, actor)
            .map_err(RuntimeError::ComponentError)?;

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

        Ok(CreateLazyMapOutput { mid })
    }

    fn handle_get_lazy_map_entry(
        &mut self,
        input: GetLazyMapEntryInput,
    ) -> Result<GetLazyMapEntryOutput, RuntimeError> {
        let actor = self.authenticate()?;

        let lazy_map = self
            .track
            .get_lazy_map(input.mid)
            .ok_or(RuntimeError::LazyMapNotFound(input.mid))?;

        let value = lazy_map
            .get_entry(&input.key, actor)
            .map_err(RuntimeError::LazyMapError)?;

        Ok(GetLazyMapEntryOutput {
            value: value.map(|e| e.to_vec()),
        })
    }

    fn handle_put_lazy_map_entry(
        &mut self,
        input: PutLazyMapEntryInput,
    ) -> Result<PutLazyMapEntryOutput, RuntimeError> {
        let actor = self.authenticate()?;

        let new_key =
            self.process_data(&input.key, Self::reject_buckets, Self::reject_bucket_refs)?;
        debug!(self, "Transformed key: {:?}", new_key);
        let new_value =
            self.process_data(&input.value, Self::reject_buckets, Self::reject_bucket_refs)?;
        debug!(self, "Transformed value: {:?}", new_value);

        let lazy_map = self
            .track
            .get_lazy_map_mut(input.mid)
            .ok_or(RuntimeError::LazyMapNotFound(input.mid))?;

        lazy_map
            .set_entry(new_key, new_value, actor)
            .map_err(RuntimeError::LazyMapError)?;

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
                let mut ids = BTreeSet::new();

                for (id, data) in entries {
                    if self.track.get_nft(resource_address, id).is_some() {
                        return Err(RuntimeError::NftAlreadyExists(resource_address, id));
                    }
                    let immutable_data =
                        self.process_data(&data.0, Self::reject_buckets, Self::reject_bucket_refs)?;
                    let mutable_data =
                        self.process_data(&data.1, Self::reject_buckets, Self::reject_bucket_refs)?;

                    self.track.put_nft(
                        resource_address,
                        id,
                        Nft::new(immutable_data, mutable_data),
                    );
                    ids.insert(id);
                }

                Ok(Supply::NonFungible { ids })
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
        debug!(self, "New resource definition: {:?}", resource_address);
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
        let actor = self.authenticate_with_badge(Some(input.auth))?;

        let resource_def = self
            .track
            .get_resource_def_mut(input.resource_address)
            .ok_or(RuntimeError::ResourceDefNotFound(input.resource_address))?;
        resource_def
            .update_flags(input.new_flags, actor)
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
        let actor = self.authenticate_with_badge(Some(input.auth))?;

        let resource_def = self
            .track
            .get_resource_def_mut(input.resource_address)
            .ok_or(RuntimeError::ResourceDefNotFound(input.resource_address))?;
        resource_def
            .update_mutable_flags(input.new_mutable_flags, actor)
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
        let actor = self.authenticate_with_badge(Some(input.auth))?;

        // allocate resource
        let supply = self.allocate_resource(input.resource_address, input.new_supply)?;

        // mint resource
        let resource_def = self
            .track
            .get_resource_def_mut(input.resource_address)
            .ok_or(RuntimeError::ResourceDefNotFound(input.resource_address))?;
        resource_def
            .mint(&supply, actor)
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
        let actor = self.authenticate_with_badge(input.auth)?;

        let bucket = self
            .buckets
            .remove(&input.bid)
            .ok_or(RuntimeError::BucketNotFound(input.bid))?;

        let resource_def = self
            .track
            .get_resource_def_mut(bucket.resource_address())
            .ok_or(RuntimeError::ResourceDefNotFound(bucket.resource_address()))?;

        resource_def
            .burn(bucket.supply(), actor)
            .map_err(RuntimeError::ResourceDefError)?;
        Ok(BurnResourceOutput {})
    }

    fn handle_update_nft_mutable_data(
        &mut self,
        input: UpdateNftMutableDataInput,
    ) -> Result<UpdateNftMutableDataOutput, RuntimeError> {
        let actor = self.authenticate_with_badge(Some(input.auth))?;

        // obtain authorization from resource definition
        let resource_def = self
            .track
            .get_resource_def(input.resource_address)
            .ok_or(RuntimeError::ResourceDefNotFound(input.resource_address))?;
        resource_def
            .check_update_nft_mutable_data_auth(actor)
            .map_err(RuntimeError::ResourceDefError)?;
        // update state
        let mutable_data = self.process_data(
            &input.new_mutable_data,
            Self::reject_buckets,
            Self::reject_bucket_refs,
        )?;
        self.track
            .get_nft_mut(input.resource_address, input.id)
            .ok_or(RuntimeError::NftNotFound(input.resource_address, input.id))?
            .set_mutable_data(mutable_data)
            .map_err(RuntimeError::NftError)?;

        Ok(UpdateNftMutableDataOutput {})
    }

    fn handle_get_nft_data(
        &mut self,
        input: GetNftDataInput,
    ) -> Result<GetNftDataOutput, RuntimeError> {
        let nft = self
            .track
            .get_nft(input.resource_address, input.id)
            .ok_or(RuntimeError::NftNotFound(input.resource_address, input.id))?;

        Ok(GetNftDataOutput {
            immutable_data: nft.immutable_data(),
            mutable_data: nft.mutable_data(),
        })
    }

    fn handle_update_resource_metadata(
        &mut self,
        input: UpdateResourceMetadataInput,
    ) -> Result<UpdateResourceMetadataOutput, RuntimeError> {
        let actor = self.authenticate_with_badge(Some(input.auth))?;

        let resource_def = self
            .track
            .get_resource_def_mut(input.resource_address)
            .ok_or(RuntimeError::ResourceDefNotFound(input.resource_address))?;
        resource_def
            .update_metadata(input.new_metadata, actor)
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
                        ids: BTreeSet::new(),
                    },
                },
            ),
            self.package()?,
        );
        let vid = self.track.new_vid();
        self.track.put_vault(vid, new_vault);

        Ok(CreateEmptyVaultOutput { vid })
    }

    fn handle_put_into_vault(
        &mut self,
        input: PutIntoVaultInput,
    ) -> Result<PutIntoVaultOutput, RuntimeError> {
        let actor = self.authenticate()?;

        let other = self
            .buckets
            .remove(&input.bid)
            .ok_or(RuntimeError::BucketNotFound(input.bid))?;

        self.track
            .get_vault_mut(input.vid)
            .ok_or(RuntimeError::VaultNotFound(input.vid))?
            .put(other, actor)
            .map_err(RuntimeError::VaultError)?;

        Ok(PutIntoVaultOutput {})
    }

    fn check_take_from_vault_auth(&mut self, vid: Vid, actor: Actor) -> Result<(), RuntimeError> {
        let resource_address = self
            .track
            .get_vault(vid)
            .ok_or(RuntimeError::VaultNotFound(vid))?
            .resource_address(actor.clone())
            .map_err(RuntimeError::VaultError)?;
        let resource_def = self
            .track
            .get_resource_def(resource_address)
            .ok_or(RuntimeError::ResourceDefNotFound(resource_address))?;
        resource_def
            .check_take_from_vault_auth(actor)
            .map_err(RuntimeError::ResourceDefError)
    }

    fn handle_take_from_vault(
        &mut self,
        input: TakeFromVaultInput,
    ) -> Result<TakeFromVaultOutput, RuntimeError> {
        let actor = self.authenticate_with_badge(input.auth)?;
        self.check_take_from_vault_auth(input.vid, actor.clone())?;

        let new_bucket = self
            .track
            .get_vault_mut(input.vid)
            .ok_or(RuntimeError::VaultNotFound(input.vid))?
            .take(input.amount, actor)
            .map_err(RuntimeError::VaultError)?;

        let bid = self.track.new_bid();
        self.buckets.insert(bid, new_bucket);

        Ok(TakeFromVaultOutput { bid })
    }

    fn handle_take_nft_from_vault(
        &mut self,
        input: TakeNftFromVaultInput,
    ) -> Result<TakeNftFromVaultOutput, RuntimeError> {
        let actor = self.authenticate_with_badge(input.auth)?;
        self.check_take_from_vault_auth(input.vid, actor.clone())?;

        let new_bucket = self
            .track
            .get_vault_mut(input.vid)
            .ok_or(RuntimeError::VaultNotFound(input.vid))?
            .take_nft(input.id, actor)
            .map_err(RuntimeError::VaultError)?;

        let bid = self.track.new_bid();
        self.buckets.insert(bid, new_bucket);

        Ok(TakeNftFromVaultOutput { bid })
    }

    fn handle_get_nft_ids_in_vault(
        &mut self,
        input: GetNftIdsInVaultInput,
    ) -> Result<GetNftIdsInVaultOutput, RuntimeError> {
        let actor = self.authenticate()?;

        let vault = self
            .track
            .get_vault(input.vid)
            .ok_or(RuntimeError::VaultNotFound(input.vid))?;

        Ok(GetNftIdsInVaultOutput {
            ids: vault.get_nft_ids(actor).map_err(RuntimeError::VaultError)?,
        })
    }

    fn handle_get_vault_amount(
        &mut self,
        input: GetVaultDecimalInput,
    ) -> Result<GetVaultDecimalOutput, RuntimeError> {
        let actor = self.authenticate()?;

        let vault = self
            .track
            .get_vault(input.vid)
            .ok_or(RuntimeError::VaultNotFound(input.vid))?;

        Ok(GetVaultDecimalOutput {
            amount: vault.amount(actor).map_err(RuntimeError::VaultError)?,
        })
    }

    fn handle_get_vault_resource_address(
        &mut self,
        input: GetVaultResourceAddressInput,
    ) -> Result<GetVaultResourceAddressOutput, RuntimeError> {
        let actor = self.authenticate()?;

        let vault = self
            .track
            .get_vault(input.vid)
            .ok_or(RuntimeError::VaultNotFound(input.vid))?;

        Ok(GetVaultResourceAddressOutput {
            resource_address: vault
                .resource_address(actor)
                .map_err(RuntimeError::VaultError)?,
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
                    ids: BTreeSet::new(),
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
                self.locked_buckets
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
                self.locked_buckets
                    .get(&input.bid)
                    .map(|x| x.bucket().resource_address())
            })
            .ok_or(RuntimeError::BucketNotFound(input.bid))?;

        Ok(GetBucketResourceAddressOutput { resource_address })
    }

    fn handle_take_nft_from_bucket(
        &mut self,
        input: TakeNftFromBucketInput,
    ) -> Result<TakeNftFromBucketOutput, RuntimeError> {
        let new_bucket = self
            .buckets
            .get_mut(&input.bid)
            .ok_or(RuntimeError::BucketNotFound(input.bid))?
            .take_nft(input.id)
            .map_err(RuntimeError::BucketError)?;
        let bid = self.track.new_bid();
        self.buckets.insert(bid, new_bucket);

        Ok(TakeNftFromBucketOutput { bid })
    }

    fn handle_get_nft_ids_in_bucket(
        &mut self,
        input: GetNftIdsInBucketInput,
    ) -> Result<GetNftIdsInBucketOutput, RuntimeError> {
        let bucket = self
            .buckets
            .get(&input.bid)
            .ok_or(RuntimeError::BucketNotFound(input.bid))?;

        Ok(GetNftIdsInBucketOutput {
            ids: bucket.get_nft_ids().map_err(RuntimeError::BucketError)?,
        })
    }

    fn handle_create_bucket_ref(
        &mut self,
        input: CreateBucketRefInput,
    ) -> Result<CreateBucketRefOutput, RuntimeError> {
        let bid = input.bid;
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
            debug!(self, "Returning {:?}: {:?}", rid, bucket_ref);
            (Rc::strong_count(&bucket_ref) - 1, bucket_ref.bucket_id())
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

    fn handle_get_nft_ids_in_bucket_ref(
        &mut self,
        input: GetNftIdsInBucketRefInput,
    ) -> Result<GetNftIdsInBucketRefOutput, RuntimeError> {
        let bucket_ref = self
            .bucket_refs
            .get(&input.rid)
            .ok_or(RuntimeError::BucketRefNotFound(input.rid))?;

        Ok(GetNftIdsInBucketRefOutput {
            ids: bucket_ref
                .bucket()
                .get_nft_ids()
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
        debug!(
            self,
            "Cloning: rid = {:?}, new rid = {:?}", input.rid, new_rid
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

    fn handle_get_transaction_signers(
        &mut self,
        _input: GetTransactionSignersInput,
    ) -> Result<GetTransactionSignersOutput, RuntimeError> {
        Ok(GetTransactionSignersOutput {
            transaction_signers: self.track.transaction_signers(),
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
                    UPDATE_NFT_MUTABLE_DATA => {
                        self.handle(args, Self::handle_update_nft_mutable_data)
                    }
                    GET_NFT_DATA => self.handle(args, Self::handle_get_nft_data),
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
                    TAKE_NFT_FROM_VAULT => self.handle(args, Self::handle_take_nft_from_vault),
                    GET_NFT_IDS_IN_VAULT => self.handle(args, Self::handle_get_nft_ids_in_vault),

                    CREATE_EMPTY_BUCKET => self.handle(args, Self::handle_create_bucket),
                    PUT_INTO_BUCKET => self.handle(args, Self::handle_put_into_bucket),
                    TAKE_FROM_BUCKET => self.handle(args, Self::handle_take_from_bucket),
                    GET_BUCKET_AMOUNT => self.handle(args, Self::handle_get_bucket_amount),
                    GET_BUCKET_RESOURCE_ADDRESS => {
                        self.handle(args, Self::handle_get_bucket_resource_address)
                    }
                    TAKE_NFT_FROM_BUCKET => self.handle(args, Self::handle_take_nft_from_bucket),
                    GET_NFT_IDS_IN_BUCKET => self.handle(args, Self::handle_get_nft_ids_in_bucket),

                    CREATE_BUCKET_REF => self.handle(args, Self::handle_create_bucket_ref),
                    DROP_BUCKET_REF => self.handle(args, Self::handle_drop_bucket_ref),
                    GET_BUCKET_REF_AMOUNT => self.handle(args, Self::handle_get_bucket_ref_amount),
                    GET_BUCKET_REF_RESOURCE_DEF => {
                        self.handle(args, Self::handle_get_bucket_ref_resource_def)
                    }
                    GET_NFT_IDS_IN_BUCKET_REF => {
                        self.handle(args, Self::handle_get_nft_ids_in_bucket_ref)
                    }
                    CLONE_BUCKET_REF => self.handle(args, Self::handle_clone_bucket_ref),

                    EMIT_LOG => self.handle(args, Self::handle_emit_log),
                    GET_PACKAGE_ADDRESS => self.handle(args, Self::handle_get_package_address),
                    GET_CALL_DATA => self.handle(args, Self::handle_get_call_data),
                    GET_TRANSACTION_HASH => self.handle(args, Self::handle_get_transaction_hash),
                    GET_CURRENT_EPOCH => self.handle(args, Self::handle_get_current_epoch),
                    GET_TRANSACTION_SIGNERS => {
                        self.handle(args, Self::handle_get_transaction_signers)
                    }
                    GENERATE_UUID => self.handle(args, Self::handle_generate_uuid),

                    _ => Err(RuntimeError::InvalidRequestCode(operation).into()),
                }
            }
            _ => Err(RuntimeError::HostFunctionNotFound(index).into()),
        }
    }
}
