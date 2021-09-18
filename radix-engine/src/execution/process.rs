use colored::*;
use sbor::parse::*;
use sbor::rust::boxed::Box;
use sbor::*;
use scrypto::buffer::*;
use scrypto::constants::*;
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

use crate::execution::*;
use crate::ledger::*;
use crate::model::*;

macro_rules! trace {
    ($proc:expr, $($args: expr),+) => {
        $proc.log(Level::Trace, format!($($args),+));
    };
}

macro_rules! info {
    ($proc:expr, $($args: expr),+) => {
        $proc.log(Level::Info, format!($($args),+));
    };
}

macro_rules! warn {
    ($proc:expr, $($args: expr),+) => {
        $proc.log(Level::Warn, format!($($args),+));
    };
}

/// A runnable process
pub struct Process<'rt, 'le, L: Ledger> {
    depth: usize,
    trace: bool,
    runtime: &'rt mut Runtime<'le, L>,
    buckets: HashMap<BID, Bucket>,
    references: HashMap<RID, BucketRef>,
    locked_buckets: HashMap<BID, BucketRef>,
    moving_buckets: HashMap<BID, Bucket>,
    moving_references: HashMap<RID, BucketRef>,
    vm: Option<VM>,
}

/// The target function to be invoked.
#[derive(Debug, Clone)]
pub struct Target {
    package: Address,
    export: String,
    function: String,
    args: Vec<Vec<u8>>,
}

/// Represents a VM instance.
pub struct VM {
    target: Target,
    module: ModuleRef,
    memory: MemoryRef,
}

impl<'rt, 'le, L: Ledger> Process<'rt, 'le, L> {
    /// Create a new process which is yet started.
    pub fn new(depth: usize, trace: bool, runtime: &'rt mut Runtime<'le, L>) -> Self {
        Self {
            depth,
            trace,
            runtime,
            buckets: HashMap::new(),
            references: HashMap::new(),
            locked_buckets: HashMap::new(),
            moving_buckets: HashMap::new(),
            moving_references: HashMap::new(),
            vm: None,
        }
    }

    pub fn prepare_call_function(
        &mut self,
        package: Address,
        blueprint: &str,
        function: String,
        args: Vec<Vec<u8>>,
    ) -> Result<Target, RuntimeError> {
        Ok(Target {
            package,
            export: format!("{}_main", blueprint),
            function,
            args,
        })
    }

    pub fn prepare_call_method(
        &mut self,
        component: Address,
        method: String,
        args: Vec<Vec<u8>>,
    ) -> Result<Target, RuntimeError> {
        let com = self
            .runtime
            .get_component(component)
            .ok_or(RuntimeError::ComponentNotFound(component))?
            .clone();

        let mut self_args = vec![scrypto_encode(&component)];
        self_args.extend(args);

        self.prepare_call_function(com.package(), com.blueprint(), method, self_args)
    }

    pub fn prepare_call_abi(
        &mut self,
        package: Address,
        blueprint: &str,
    ) -> Result<Target, RuntimeError> {
        Ok(Target {
            package,
            export: format!("{}_abi", blueprint),
            function: String::new(),
            args: Vec::new(),
        })
    }

    /// Put resources into this process's treasury.
    pub fn put_resources(
        &mut self,
        buckets: HashMap<BID, Bucket>,
        references: HashMap<RID, BucketRef>,
    ) {
        self.buckets.extend(buckets);
        self.references.extend(references);
    }

    /// Take resources from this process.
    pub fn take_resources(&mut self) -> (HashMap<BID, Bucket>, HashMap<RID, BucketRef>) {
        let buckets = self.moving_buckets.drain().collect();
        let references = self.moving_references.drain().collect();
        (buckets, references)
    }

    /// Run the specified export with this process.
    pub fn run(&mut self, target: Target) -> Result<Vec<u8>, RuntimeError> {
        #[cfg(not(feature = "alloc"))]
        let now = std::time::Instant::now();
        info!(
            self,
            "Run started: package = {}, export = {}", target.package, target.export
        );

        // Load the code
        let (module, memory) = self
            .runtime
            .load_module(target.package)
            .ok_or(RuntimeError::PackageNotFound(target.package))?;
        let vm = VM {
            target: target.clone(),
            module: module.clone(),
            memory,
        };
        self.vm = Some(vm);

        // run the main function
        let invoke_res = module.invoke_export(target.export.as_str(), &[], self);
        trace!(self, "Invoke result: {:?}", invoke_res);
        let return_value = invoke_res
            .map_err(RuntimeError::InvokeError)?
            .ok_or(RuntimeError::NoReturnValue)?;

        // move resources based on return data
        let output = match return_value {
            RuntimeValue::I32(ptr) => {
                let bytes = self.read_bytes(ptr)?;
                self.process_data(&bytes, Self::move_buckets, Self::move_references)?;
                bytes
            }
            _ => {
                return Err(RuntimeError::InvalidReturnType);
            }
        };

        #[cfg(not(feature = "alloc"))]
        info!(
            self,
            "Run finished: time elapsed = {} ms",
            now.elapsed().as_millis()
        );
        #[cfg(feature = "alloc")]
        info!(self, "Run finished");

        Ok(output)
    }

    /// Call a function/method.
    pub fn call(&mut self, target: Target) -> Result<Vec<u8>, RuntimeError> {
        // move resources
        for arg in &target.args {
            self.process_data(arg, Self::move_buckets, Self::move_references)?;
        }
        let (buckets_out, references_out) = self.take_resources();
        let mut process = Process::new(self.depth + 1, self.trace, self.runtime);
        process.put_resources(buckets_out, references_out);

        // run the function and finalize
        let result = process.run(target)?;
        process.finalize()?;

        // move resources
        let (buckets_in, references_in) = process.take_resources();
        self.put_resources(buckets_in, references_in);

        // scan locked buckets for some might have been unlocked by child processes
        let bids: Vec<BID> = self
            .locked_buckets
            .values()
            .filter(|v| Rc::strong_count(v) == 1)
            .map(|v| v.bucket_id())
            .collect();
        for bid in bids {
            trace!(self, "Moving {:?} to unlocked_buckets state", bid);
            let bucket_rc = self.locked_buckets.remove(&bid).unwrap();
            let bucket = Rc::try_unwrap(bucket_rc).unwrap();
            self.buckets.insert(bid, bucket.into());
        }

        Ok(result)
    }

    /// Return the package address
    pub fn package(&self) -> Result<Address, RuntimeError> {
        self.vm
            .as_ref()
            .ok_or(RuntimeError::VmNotStarted)
            .map(|vm| vm.target.package)
    }

    /// Return the function name
    pub fn function(&self) -> Result<String, RuntimeError> {
        self.vm
            .as_ref()
            .ok_or(RuntimeError::VmNotStarted)
            .map(|vm| vm.target.function.clone())
    }

    /// Return the function name
    pub fn args(&self) -> Result<Vec<Vec<u8>>, RuntimeError> {
        self.vm
            .as_ref()
            .ok_or(RuntimeError::VmNotStarted)
            .map(|vm| vm.target.args.clone())
    }

    /// Return the module reference
    pub fn module(&self) -> Result<ModuleRef, RuntimeError> {
        self.vm
            .as_ref()
            .ok_or(RuntimeError::VmNotStarted)
            .map(|vm| vm.module.clone())
    }

    /// Return the memory reference
    pub fn memory(&self) -> Result<MemoryRef, RuntimeError> {
        self.vm
            .as_ref()
            .ok_or(RuntimeError::VmNotStarted)
            .map(|vm| vm.memory.clone())
    }

    /// Finalize this process.
    pub fn finalize(&self) -> Result<(), RuntimeError> {
        let mut success = true;

        for (bid, bucket) in &self.buckets {
            if bucket.amount() != Amount::zero() {
                warn!(self, "Pending bucket: {:?} {:?}", bid, bucket);
                success = false;
            }
        }
        for (bid, bucket) in &self.locked_buckets {
            warn!(self, "Pending locked bucket: {:?} {:?}", bid, bucket);
            success = false;
        }
        for (rid, bucket_ref) in &self.references {
            warn!(self, "Pending reference: {:?} {:?}", rid, bucket_ref);
            success = false;
        }

        if success {
            Ok(())
        } else {
            Err(RuntimeError::ResourceLeak)
        }
    }

    /// Log a message to console.
    #[allow(unused_variables)]
    pub fn log(&self, level: Level, msg: String) {
        if self.trace {
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

    pub fn publish(
        &mut self,
        input: PublishPackageInput,
    ) -> Result<PublishPackageOutput, RuntimeError> {
        let address = self.runtime.new_package_address();

        if self.runtime.get_package(address).is_some() {
            return Err(RuntimeError::PackageAlreadyExists(address));
        }
        validate_module(&input.code)?;

        trace!(
            self,
            "New package: address = {:?}, code length = {}",
            address,
            input.code.len()
        );
        self.runtime.put_package(address, Package::new(input.code));

        Ok(PublishPackageOutput { package: address })
    }

    pub fn call_function(
        &mut self,
        input: CallFunctionInput,
    ) -> Result<CallFunctionOutput, RuntimeError> {
        trace!(
            self,
            "CALL started: package = {}, blueprint = {}, function = {}, args = {:?}",
            input.package,
            input.blueprint,
            input.function,
            input.args
        );

        let target = self.prepare_call_function(
            input.package,
            input.blueprint.as_str(),
            input.function,
            input.args,
        )?;
        let result = self.call(target);

        trace!(self, "CALL finished");
        Ok(CallFunctionOutput { rtn: result? })
    }

    pub fn call_method(
        &mut self,
        input: CallMethodInput,
    ) -> Result<CallMethodOutput, RuntimeError> {
        trace!(
            self,
            "CALL started: component = {}, method = {}, args = {:?}",
            input.component,
            input.method,
            input.args
        );

        let target = self.prepare_call_method(input.component, input.method, input.args)?;
        let result = self.call(target);

        trace!(self, "CALL finished");
        Ok(CallMethodOutput { rtn: result? })
    }

    pub fn create_component(
        &mut self,
        input: CreateComponentInput,
    ) -> Result<CreateComponentOutput, RuntimeError> {
        let address = self.runtime.new_component_address();

        if self.runtime.get_component(address).is_some() {
            return Err(RuntimeError::ComponentAlreadyExists(address));
        }

        let new_state =
            self.process_data(&input.state, Self::reject_buckets, Self::reject_references)?;
        trace!(
            self,
            "New component: address = {:?}, state = {:?}",
            address,
            new_state
        );

        let component = Component::new(self.package()?, input.blueprint, new_state);
        self.runtime.put_component(address, component);

        Ok(CreateComponentOutput { component: address })
    }

    pub fn get_component_info(
        &mut self,
        input: GetComponentInfoInput,
    ) -> Result<GetComponentInfoOutput, RuntimeError> {
        let package = self.package()?;

        let component = self
            .runtime
            .get_component(input.component)
            .ok_or(RuntimeError::ComponentNotFound(input.component))?;
        if package != component.package() {
            return Err(RuntimeError::UnauthorizedAccess);
        }

        Ok(GetComponentInfoOutput {
            package: component.package(),
            blueprint: component.blueprint().to_owned(),
        })
    }

    pub fn get_component_state(
        &mut self,
        input: GetComponentStateInput,
    ) -> Result<GetComponentStateOutput, RuntimeError> {
        let package = self.package()?;

        let component = self
            .runtime
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

    pub fn put_component_state(
        &mut self,
        input: PutComponentStateInput,
    ) -> Result<PutComponentStateOutput, RuntimeError> {
        let package = self.package()?;

        let new_state =
            self.process_data(&input.state, Self::reject_buckets, Self::reject_references)?;
        trace!(self, "Transformed: {:?}", new_state);

        let component = self
            .runtime
            .get_component_mut(input.component)
            .ok_or(RuntimeError::ComponentNotFound(input.component))?;
        if package != component.package() {
            return Err(RuntimeError::UnauthorizedAccess);
        }

        component.set_state(new_state);

        Ok(PutComponentStateOutput {})
    }

    pub fn create_storage(
        &mut self,
        _input: CreateStorageInput,
    ) -> Result<CreateStorageOutput, RuntimeError> {
        let sid = self.runtime.new_sid();

        self.runtime.put_storage(sid, Storage::new(self.package()?));

        Ok(CreateStorageOutput { storage: sid })
    }

    pub fn get_storage_entry(
        &mut self,
        input: GetStorageEntryInput,
    ) -> Result<GetStorageEntryOutput, RuntimeError> {
        let package = self.package()?;

        let storage = self
            .runtime
            .get_storage(input.storage)
            .ok_or(RuntimeError::StorageNotFound(input.storage))?;
        if package != storage.owner() {
            return Err(RuntimeError::UnauthorizedAccess);
        }

        Ok(GetStorageEntryOutput {
            value: storage.get_entry(&input.key).map(|e| e.to_vec()),
        })
    }

    pub fn put_storage_entry(
        &mut self,
        input: PutStorageEntryInput,
    ) -> Result<PutStorageEntryOutput, RuntimeError> {
        let package = self.package()?;

        let new_key =
            self.process_data(&input.key, Self::reject_buckets, Self::reject_references)?;
        trace!(self, "Transformed key: {:?}", new_key);
        let new_value =
            self.process_data(&input.value, Self::reject_buckets, Self::reject_references)?;
        trace!(self, "Transformed value: {:?}", new_value);

        let storage = self
            .runtime
            .get_storage_mut(input.storage)
            .ok_or(RuntimeError::StorageNotFound(input.storage))?;
        if package != storage.owner() {
            return Err(RuntimeError::UnauthorizedAccess);
        }

        storage.set_entry(new_key, new_value);

        Ok(PutStorageEntryOutput {})
    }

    pub fn create_resource_mutable(
        &mut self,
        input: CreateResourceMutableInput,
    ) -> Result<CreateResourceMutableOutput, RuntimeError> {
        let resource = Resource {
            metadata: input.metadata,
            minter: Some(input.minter),
            supply: None,
        };

        let address = self.runtime.new_resource_address();

        if self.runtime.get_resource(address).is_some() {
            return Err(RuntimeError::ResourceAlreadyExists(address));
        } else {
            trace!(self, "New resource: {:?}", address);

            self.runtime.put_resource(address, resource);
        }
        Ok(CreateResourceMutableOutput { resource: address })
    }

    pub fn create_resource_fixed(
        &mut self,
        input: CreateResourceFixedInput,
    ) -> Result<CreateResourceFixedOutput, RuntimeError> {
        let resource = Resource {
            metadata: input.metadata,
            minter: None,
            supply: Some(input.supply),
        };

        let address = self.runtime.new_resource_address();

        if self.runtime.get_resource(address).is_some() {
            return Err(RuntimeError::ResourceAlreadyExists(address));
        } else {
            trace!(self, "New resource: {:?}", address);

            self.runtime.put_resource(address, resource);
        }

        let bucket = Bucket::new(input.supply, address);
        let bid = self.runtime.new_bucket_id();
        self.buckets.insert(bid, bucket);

        Ok(CreateResourceFixedOutput { bucket: bid })
    }

    pub fn ge_resource_info(
        &mut self,
        input: GetResourceInfoInput,
    ) -> Result<GetResourceInfoOutput, RuntimeError> {
        let resource = self
            .runtime
            .get_resource(input.resource)
            .ok_or(RuntimeError::ResourceNotFound(input.resource))?
            .clone();

        Ok(GetResourceInfoOutput {
            metadata: resource.metadata,
            minter: resource.minter,
            supply: resource.supply,
        })
    }

    pub fn mint_resource(
        &mut self,
        input: MintResourceInput,
    ) -> Result<MintResourceOutput, RuntimeError> {
        let resource = self
            .runtime
            .get_resource(input.resource)
            .ok_or(RuntimeError::ResourceNotFound(input.resource))?;

        match resource.minter {
            Some(address) => {
                let authorized = match address {
                    Address::Package(_) => address == self.package()?,
                    Address::Component(_) => {
                        self.runtime
                            .get_component(address)
                            .ok_or(RuntimeError::ComponentNotFound(address))?
                            .package()
                            == self.package()?
                    }
                    _ => false,
                };
                if !authorized {
                    return Err(RuntimeError::UnauthorizedToMint);
                }
            }
            _ => {
                return Err(RuntimeError::UnableToMintFixedResource);
            }
        }

        let bucket = Bucket::new(input.amount, input.resource);
        let bid = self.runtime.new_bucket_id();
        self.buckets.insert(bid, bucket);
        Ok(MintResourceOutput { bucket: bid })
    }

    pub fn new_vault(
        &mut self,
        input: CreateEmptyVaultInput,
    ) -> Result<CreateEmptyVaultOutput, RuntimeError> {
        let package = self.package()?;

        let new_vault = Vault::new(Bucket::new(Amount::zero(), input.resource), package);
        let new_vid = self.runtime.new_vault_id();
        self.runtime.put_vault(new_vid, new_vault);

        Ok(CreateEmptyVaultOutput { vault: new_vid })
    }

    pub fn put_into_vault(
        &mut self,
        input: PutIntoVaultInput,
    ) -> Result<PutIntoVaultOutput, RuntimeError> {
        let package = self.package()?;

        let other = self
            .buckets
            .remove(&input.bucket)
            .ok_or(RuntimeError::BucketNotFound(input.bucket))?;

        self.runtime
            .get_vault_mut(input.vault)
            .ok_or(RuntimeError::VaultNotFound(input.vault))?
            .put(other, package)
            .map_err(RuntimeError::AccountingError)?;

        Ok(PutIntoVaultOutput {})
    }

    pub fn take_from_vault(
        &mut self,
        input: TakeFromVaultInput,
    ) -> Result<TakeFromVaultOutput, RuntimeError> {
        let package = self.package()?;

        let new_bucket = self
            .runtime
            .get_vault_mut(input.vault)
            .ok_or(RuntimeError::VaultNotFound(input.vault))?
            .take(input.amount, package)
            .map_err(RuntimeError::AccountingError)?;

        let new_bid = self.runtime.new_bucket_id();
        self.buckets.insert(new_bid, new_bucket);

        Ok(TakeFromVaultOutput { bucket: new_bid })
    }

    pub fn get_vault_amount(
        &mut self,
        input: GetVaultAmountInput,
    ) -> Result<GetVaultAmountOutput, RuntimeError> {
        let amount = self
            .runtime
            .get_vault(input.vault)
            .map(|b| b.amount())
            .ok_or(RuntimeError::VaultNotFound(input.vault))?;

        Ok(GetVaultAmountOutput { amount })
    }

    pub fn get_vault_resource(
        &mut self,
        input: GetVaultResourceInput,
    ) -> Result<GetVaultResourceOutput, RuntimeError> {
        let resource = self
            .runtime
            .get_vault(input.vault)
            .map(|b| b.resource())
            .ok_or(RuntimeError::VaultNotFound(input.vault))?;

        Ok(GetVaultResourceOutput { resource })
    }

    pub fn new_bucket(
        &mut self,
        input: CreateEmptyBucketInput,
    ) -> Result<CreateEmptyBucketOutput, RuntimeError> {
        let new_bucket = Bucket::new(Amount::zero(), input.resource);
        let new_bid = self.runtime.new_bucket_id();
        self.buckets.insert(new_bid, new_bucket);

        Ok(CreateEmptyBucketOutput { bucket: new_bid })
    }

    pub fn put_into_bucket(
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

    pub fn take_from_bucket(
        &mut self,
        input: TakeFromBucketInput,
    ) -> Result<TakeFromBucketOutput, RuntimeError> {
        let new_bucket = self
            .buckets
            .get_mut(&input.bucket)
            .ok_or(RuntimeError::BucketNotFound(input.bucket))?
            .take(input.amount)
            .map_err(RuntimeError::AccountingError)?;
        let new_bid = self.runtime.new_bucket_id();
        self.buckets.insert(new_bid, new_bucket);

        Ok(TakeFromBucketOutput { bucket: new_bid })
    }

    pub fn get_bucket_amount(
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

    pub fn get_bucket_resource(
        &mut self,
        input: GetBucketResourceInput,
    ) -> Result<GetBucketResourceOutput, RuntimeError> {
        let bid = input.bucket;
        let resource = self
            .buckets
            .get(&bid)
            .map(|b| b.resource())
            .or_else(|| self.locked_buckets.get(&bid).map(|x| x.bucket().resource()))
            .ok_or(RuntimeError::BucketNotFound(bid))?;

        Ok(GetBucketResourceOutput { resource })
    }

    pub fn create_reference(
        &mut self,
        input: CreateReferenceInput,
    ) -> Result<CreateReferenceOutput, RuntimeError> {
        let bid = input.bucket;
        let rid = self.runtime.new_rid();
        trace!(self, "Borrowing: bid =  {:?}, rid = {:?}", bid, rid);

        match self.locked_buckets.get_mut(&bid) {
            Some(bucket_rc) => {
                // re-borrow
                self.references.insert(rid, bucket_rc.clone());
            }
            None => {
                // first time borrow
                let bucket = BucketRef::new(LockedBucket::new(
                    bid,
                    self.buckets
                        .remove(&bid)
                        .ok_or(RuntimeError::BucketNotFound(bid))?,
                ));
                self.references.insert(rid, bucket.clone());
                self.locked_buckets.insert(bid, bucket);
            }
        }

        Ok(CreateReferenceOutput { reference: rid })
    }

    pub fn drop_reference(
        &mut self,
        input: DropReferenceInput,
    ) -> Result<DropReferenceOutput, RuntimeError> {
        let rid = input.reference;

        let (count, bid) = {
            let bucket = self
                .references
                .remove(&rid)
                .ok_or(RuntimeError::ReferenceNotFound(rid))?;
            trace!(self, "Returning {:?}: {:?}", rid, bucket);
            (Rc::strong_count(&bucket) - 1, bucket.bucket_id())
        };

        if count == 1 {
            if let Some(b) = self.locked_buckets.remove(&bid) {
                self.buckets.insert(bid, Rc::try_unwrap(b).unwrap().into());
            }
        }

        Ok(DropReferenceOutput {})
    }

    pub fn get_ref_amount(
        &mut self,
        input: GetRefAmountInput,
    ) -> Result<GetRefAmountOutput, RuntimeError> {
        let reference = self
            .references
            .get(&input.reference)
            .ok_or(RuntimeError::ReferenceNotFound(input.reference))?;

        Ok(GetRefAmountOutput {
            amount: reference.bucket().amount(),
        })
    }

    pub fn get_ref_resource(
        &mut self,
        input: GetRefResourceInput,
    ) -> Result<GetRefResourceOutput, RuntimeError> {
        let reference = self
            .references
            .get(&input.reference)
            .ok_or(RuntimeError::ReferenceNotFound(input.reference))?;

        Ok(GetRefResourceOutput {
            resource: reference.bucket().resource(),
        })
    }

    pub fn emit_log(&mut self, input: EmitLogInput) -> Result<EmitLogOutput, RuntimeError> {
        let level = match input.level {
            0 => Ok(Level::Error),
            1 => Ok(Level::Warn),
            2 => Ok(Level::Info),
            3 => Ok(Level::Debug),
            4 => Ok(Level::Trace),
            _ => Err(RuntimeError::InvalidLogLevel),
        };

        self.runtime.add_log(level?, input.message);

        Ok(EmitLogOutput {})
    }

    pub fn get_package_address(
        &mut self,
        _input: GetPackageAddressInput,
    ) -> Result<GetPackageAddressOutput, RuntimeError> {
        Ok(GetPackageAddressOutput {
            address: self.package()?,
        })
    }

    pub fn get_call_data(
        &mut self,
        _input: GetCallDataInput,
    ) -> Result<GetCallDataOutput, RuntimeError> {
        Ok(GetCallDataOutput {
            function: self.function()?,
            args: self.args()?,
        })
    }

    pub fn get_transaction_hash(
        &mut self,
        _input: GetTransactionHashInput,
    ) -> Result<GetTransactionHashOutput, RuntimeError> {
        Ok(GetTransactionHashOutput {
            tx_hash: self.runtime.tx_hash(),
        })
    }

    /// Process SBOR data by applying function on BID and RID.
    fn process_data(
        &mut self,
        data: &[u8],
        bf: fn(&mut Self, BID) -> Result<BID, RuntimeError>,
        rf: fn(&mut Self, RID) -> Result<RID, RuntimeError>,
    ) -> Result<Vec<u8>, RuntimeError> {
        let value = parse_any(data).map_err(RuntimeError::InvalidData)?;
        let transformed = self.visit(value, bf, rf)?;

        let mut encoder = Encoder::with_type(Vec::with_capacity(data.len() + 512));
        write_any(None, &transformed, &mut encoder);
        Ok(encoder.into())
    }

    /// Traverse SBOR value recursively. TODO: stack overflow
    fn visit(
        &mut self,
        v: Value,
        bf: fn(&mut Self, BID) -> Result<BID, RuntimeError>,
        rf: fn(&mut Self, RID) -> Result<RID, RuntimeError>,
    ) -> Result<Value, RuntimeError> {
        match v {
            // basic types
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
            // rust types
            Value::Option(x) => match *x {
                Some(value) => Ok(Value::Option(Box::new(Some(self.visit(value, bf, rf)?)))),
                None => Ok(Value::Option(Box::new(None))),
            },
            Value::Box(value) => Ok(Value::Box(Box::new(self.visit(*value, bf, rf)?))),
            Value::Array(ty, values) => Ok(Value::Array(ty, self.visit_vec(values, bf, rf)?)),
            Value::Tuple(values) => Ok(Value::Tuple(self.visit_vec(values, bf, rf)?)),
            Value::Struct(fields) => Ok(Value::Struct(self.visit_fields(fields, bf, rf)?)),
            Value::Enum(index, fields) => {
                Ok(Value::Enum(index, self.visit_fields(fields, bf, rf)?))
            }
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
        bf: fn(&mut Self, BID) -> Result<BID, RuntimeError>,
        rf: fn(&mut Self, RID) -> Result<RID, RuntimeError>,
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
        bf: fn(&mut Self, BID) -> Result<BID, RuntimeError>,
        rf: fn(&mut Self, RID) -> Result<RID, RuntimeError>,
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
        bf: fn(&mut Self, BID) -> Result<BID, RuntimeError>,
        rf: fn(&mut Self, RID) -> Result<RID, RuntimeError>,
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
        bf: fn(&mut Self, BID) -> Result<BID, RuntimeError>,
        rf: fn(&mut Self, RID) -> Result<RID, RuntimeError>,
    ) -> Result<Value, RuntimeError> {
        match ty {
            SCRYPTO_TYPE_BID => {
                let bid = bf(
                    self,
                    BID::try_from(data.as_slice()).map_err(|_| {
                        RuntimeError::InvalidData(DecodeError::InvalidCustomData(ty))
                    })?,
                )?;
                Ok(Value::Custom(ty, bid.to_vec()))
            }
            SCRYPTO_TYPE_RID => {
                let rid = rf(
                    self,
                    RID::try_from(data.as_slice()).map_err(|_| {
                        RuntimeError::InvalidData(DecodeError::InvalidCustomData(ty))
                    })?,
                )?;
                Ok(Value::Custom(ty, rid.to_vec()))
            }
            _ => Ok(Value::Custom(ty, data)),
        }
    }

    /// Remove transient buckets from this process
    fn move_buckets(&mut self, bid: BID) -> Result<BID, RuntimeError> {
        let bucket = self
            .buckets
            .remove(&bid)
            .ok_or(RuntimeError::BucketNotFound(bid))?;
        trace!(self, "Moving {:?}: {:?}", bid, bucket);
        self.moving_buckets.insert(bid, bucket);
        Ok(bid)
    }

    /// Remove transient buckets from this process
    fn move_references(&mut self, rid: RID) -> Result<RID, RuntimeError> {
        let bucket_ref = self
            .references
            .remove(&rid)
            .ok_or(RuntimeError::ReferenceNotFound(rid))?;
        trace!(self, "Moving {:?}: {:?}", rid, bucket_ref);
        self.moving_references.insert(rid, bucket_ref);
        Ok(rid)
    }

    /// Reject buckets movements
    fn reject_buckets(&mut self, _: BID) -> Result<BID, RuntimeError> {
        Err(RuntimeError::BucketMoveNotAllowed)
    }

    /// Reject references movements
    fn reject_references(&mut self, _: RID) -> Result<RID, RuntimeError> {
        Err(RuntimeError::ReferenceMoveNotAllowed)
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

        Err(RuntimeError::UnableToAllocateMemory)
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
        trace: bool,
    ) -> Result<Option<RuntimeValue>, Trap> {
        let input_ptr: u32 = args.nth_checked(1)?;
        let input_len: u32 = args.nth_checked(2)?;
        let input_bytes = self
            .memory()?
            .get(input_ptr, input_len as usize)
            .map_err(|e| Trap::from(RuntimeError::MemoryAccessError(e)))?;
        let input: I = scrypto_decode(&input_bytes)
            .map_err(|e| Trap::from(RuntimeError::InvalidRequest(e)))?;
        if trace {
            trace!(self, "{:?}", input);
        }

        let output: O = handler(self, input).map_err(Trap::from)?;
        let output_bytes = scrypto_encode(&output);
        let output_ptr = self.send_bytes(&output_bytes).map_err(Trap::from)?;
        if trace {
            trace!(self, "{:?}", output);
        }

        Ok(Some(RuntimeValue::I32(output_ptr)))
    }
}

impl<'rt, 'le, L: Ledger> Externals for Process<'rt, 'le, L> {
    fn invoke_index(
        &mut self,
        index: usize,
        args: RuntimeArgs,
    ) -> Result<Option<RuntimeValue>, Trap> {
        match index {
            KERNEL_INDEX => {
                let operation: u32 = args.nth_checked(0)?;
                match operation {
                    PUBLISH => self.handle(args, Self::publish, false),
                    CALL_FUNCTION => self.handle(args, Self::call_function, true),
                    CALL_METHOD => self.handle(args, Self::call_method, true),

                    CREATE_COMPONENT => self.handle(args, Self::create_component, true),
                    GET_COMPONENT_INFO => self.handle(args, Self::get_component_info, true),
                    GET_COMPONENT_STATE => self.handle(args, Self::get_component_state, true),
                    PUT_COMPONENT_STATE => self.handle(args, Self::put_component_state, true),

                    CREATE_STORAGE => self.handle(args, Self::create_storage, true),
                    GET_STORAGE_ENTRY => self.handle(args, Self::get_storage_entry, true),
                    PUT_STORAGE_ENTRY => self.handle(args, Self::put_storage_entry, true),

                    CREATE_RESOURCE_MUTABLE => {
                        self.handle(args, Self::create_resource_mutable, true)
                    }
                    CREATE_RESOURCE_FIXED => self.handle(args, Self::create_resource_fixed, true),
                    GET_RESOURCE_INFO => self.handle(args, Self::ge_resource_info, true),
                    MINT_RESOURCE => self.handle(args, Self::mint_resource, true),

                    CREATE_EMPTY_VAULT => self.handle(args, Self::new_vault, true),
                    PUT_INTO_VAULT => self.handle(args, Self::put_into_vault, true),
                    TAKE_FROM_VAULT => self.handle(args, Self::take_from_vault, true),
                    GET_VAULT_AMOUNT => self.handle(args, Self::get_vault_amount, true),
                    GET_VAULT_RESOURCE => self.handle(args, Self::get_vault_resource, true),

                    CREATE_EMPTY_BUCKET => self.handle(args, Self::new_bucket, true),
                    PUT_INTO_BUCKET => self.handle(args, Self::put_into_bucket, true),
                    TAKE_FROM_BUCKET => self.handle(args, Self::take_from_bucket, true),
                    GET_BUCKET_AMOUNT => self.handle(args, Self::get_bucket_amount, true),
                    GET_BUCKET_RESOURCE => self.handle(args, Self::get_bucket_resource, true),

                    CREATE_REFERENCE => self.handle(args, Self::create_reference, true),
                    DROP_REFERENCE => self.handle(args, Self::drop_reference, true),
                    GET_REF_AMOUNT => self.handle(args, Self::get_ref_amount, true),
                    GET_REF_RESOURCE => self.handle(args, Self::get_ref_resource, true),

                    EMIT_LOG => self.handle(args, Self::emit_log, true),
                    GET_PACKAGE_ADDRESS => self.handle(args, Self::get_package_address, true),
                    GET_CALL_DATA => self.handle(args, Self::get_call_data, true),
                    GET_TRANSACTION_HASH => self.handle(args, Self::get_transaction_hash, true),
                    _ => Err(RuntimeError::InvalidOpCode(operation).into()),
                }
            }
            _ => Err(RuntimeError::UnknownHostFunction(index).into()),
        }
    }
}
