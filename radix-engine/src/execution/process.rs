use std::cell::RefCell;
use std::fmt;
use std::rc::Rc;
use std::time::Instant;

use colored::*;
use sbor::*;
use scrypto::buffer::*;
use scrypto::kernel::*;
use scrypto::types::rust::collections::*;
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
    references: HashMap<RID, Rc<RefCell<BucketBorrowed>>>,
    locked_buckets: HashMap<BID, Rc<RefCell<BucketBorrowed>>>,
    moving_buckets: HashMap<BID, Bucket>,
    moving_references: HashMap<RID, Rc<RefCell<BucketBorrowed>>>,
    vm: Option<VM>,
}

pub struct VM {
    package: Address,
    function: String,
    args: Vec<Vec<u8>>,
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

    /// Put resources into this process's treasury.
    pub fn put_resources(
        &mut self,
        buckets: HashMap<BID, Bucket>,
        references: HashMap<RID, Rc<RefCell<BucketBorrowed>>>,
    ) {
        self.buckets.extend(buckets);
        self.references.extend(references);
    }

    /// Take resources from this process.
    pub fn take_resources(
        &mut self,
    ) -> (
        HashMap<BID, Bucket>,
        HashMap<RID, Rc<RefCell<BucketBorrowed>>>,
    ) {
        let buckets = self.moving_buckets.clone();
        let references = self.moving_references.clone();
        self.moving_buckets.clear();
        self.moving_references.clear();
        (buckets, references)
    }

    /// Run the specified function within this process.
    pub fn run(
        &mut self,
        package: Address,
        export: String,
        function: String,
        args: Vec<Vec<u8>>,
    ) -> Result<Vec<u8>, RuntimeError> {
        let now = Instant::now();
        info!(
            self,
            "Run started: package = {}, export = {}", package, export
        );

        // Load the code
        let (module, memory) = self
            .runtime
            .load_module(package)
            .ok_or(RuntimeError::PackageNotFound(package))?;
        let vm = VM {
            package,
            function,
            args,
            module: module.clone(),
            memory: memory.clone(),
        };
        assert!(self.vm.is_none(), "Each process can run at most once.");
        self.vm = Some(vm);

        // run the main function
        let invoke_result = module.invoke_export(export.as_str(), &[], self);
        trace!(self, "Invoke result: {:?}", invoke_result);
        let return_value = invoke_result
            .map_err(|e| RuntimeError::InvokeError(e))?
            .ok_or(RuntimeError::NoReturnValue)?;

        // move resources based on return data
        let output = match return_value {
            RuntimeValue::I32(ptr) => {
                let bytes = self.read_bytes(ptr)?;
                self.transform_sbor_data(
                    &bytes,
                    Self::move_transient_reject_persisted,
                    Self::move_references,
                )?;
                bytes
            }
            _ => {
                return Err(RuntimeError::InvalidReturnType);
            }
        };

        info!(
            self,
            "Run finished: time elapsed = {} ms",
            now.elapsed().as_millis()
        );

        Ok(output)
    }

    /// Call a blueprint function.
    pub fn call_function(
        &mut self,
        package: Address,
        blueprint: String,
        function: String,
        args: Vec<Vec<u8>>,
    ) -> Result<Vec<u8>, RuntimeError> {
        trace!(
            self,
            "Calling function: package = {}, blueprint = {}, function = {}, args = {:02x?}",
            package,
            blueprint,
            function,
            args
        );
        // move resources
        for arg in &args {
            self.transform_sbor_data(
                arg,
                Self::move_transient_reject_persisted,
                Self::move_references,
            )?;
        }
        let (buckets_out, references_out) = self.take_resources();
        let mut process = Process::new(self.depth + 1, self.trace, self.runtime);
        process.put_resources(buckets_out, references_out);

        // run the function and finalize
        let result = process.run(package, format!("{}_main", blueprint), function, args);
        process.finalize()?;

        // move resources
        let (buckets_in, references_in) = process.take_resources();
        self.put_resources(buckets_in, references_in);

        // scan locked buckets for some might have been unlocked by child processes
        let bids: Vec<BID> = self
            .locked_buckets
            .values()
            .filter(|v| v.borrow().ref_count() == 0)
            .map(|v| v.borrow().bid())
            .collect();
        for bid in bids {
            trace!(self, "Moving {:02x?} to unlocked_buckets state", bid);
            let bucket_rc = self.locked_buckets.remove(&bid).unwrap();
            let bucket = Rc::try_unwrap(bucket_rc).unwrap().into_inner();
            self.buckets.insert(bid, bucket.into());
        }

        result
    }

    /// Call a component method.
    pub fn call_method(
        &mut self,
        component: Address,
        method: String,
        mut args: Vec<Vec<u8>>,
    ) -> Result<Vec<u8>, RuntimeError> {
        let info = self
            .runtime
            .get_component(component)
            .ok_or(RuntimeError::ComponentNotFound(component))?
            .clone();
        args.insert(0, scrypto_encode(&component));

        self.call_function(info.package(), info.name().to_owned(), method, args)
    }

    /// Return the package address
    pub fn package(&self) -> Result<Address, RuntimeError> {
        self.vm
            .as_ref()
            .ok_or(RuntimeError::VmNotStarted)
            .map(|vm| vm.package)
    }

    /// Return the function name
    pub fn function(&self) -> Result<String, RuntimeError> {
        self.vm
            .as_ref()
            .ok_or(RuntimeError::VmNotStarted)
            .map(|vm| vm.function.clone())
    }

    /// Return the function name
    pub fn args(&self) -> Result<Vec<Vec<u8>>, RuntimeError> {
        self.vm
            .as_ref()
            .ok_or(RuntimeError::VmNotStarted)
            .map(|vm| vm.args.clone())
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
        let mut buckets = vec![];
        let mut references = vec![];

        for (bid, bucket) in &self.buckets {
            if bucket.amount() != U256::zero() {
                warn!(self, "Pending bucket: {:02x?} {:02x?}", bid, bucket);
                buckets.push(*bid);
            }
        }
        for (bid, bucket) in &self.locked_buckets {
            warn!(self, "Pending locked bucket: {:02x?} {:02x?}", bid, bucket);
            buckets.push(*bid);
        }
        for (rid, bucket_ref) in &self.references {
            warn!(self, "Pending reference: {:02x?} {:02x?}", rid, bucket_ref);
            references.push(*rid);
        }

        if buckets.is_empty() && references.is_empty() {
            Ok(())
        } else {
            Err(RuntimeError::ResourceLeak(buckets, references))
        }
    }

    /// Log a message to console.
    pub fn log(&self, level: Level, msg: String) {
        if self.trace {
            let (l, m) = match level {
                Level::Error => ("ERROR".red(), msg.to_string().red()),
                Level::Warn => ("WARN".yellow(), msg.to_string().yellow()),
                Level::Info => ("INFO".green(), msg.to_string().green()),
                Level::Debug => ("DEBUG".cyan(), msg.to_string().cyan()),
                Level::Trace => ("TRACE".normal(), msg.to_string().normal()),
            };

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
        load_module(&input.code)?;

        trace!(
            self,
            "New package: address = {:02x?}, code length = {}",
            address,
            input.code.len()
        );
        self.runtime.put_package(address, Package::new(input.code));

        Ok(PublishPackageOutput { package: address })
    }

    pub fn call_blueprint(
        &mut self,
        input: CallBlueprintInput,
    ) -> Result<CallBlueprintOutput, RuntimeError> {
        let output = self.call_function(input.package, input.name, input.function, input.args);

        Ok(CallBlueprintOutput { rtn: output? })
    }

    pub fn call_component(
        &mut self,
        input: CallComponentInput,
    ) -> Result<CallComponentOutput, RuntimeError> {
        let output = self.call_method(input.component, input.method, input.args);

        Ok(CallComponentOutput { rtn: output? })
    }

    pub fn create_component(
        &mut self,
        input: CreateComponentInput,
    ) -> Result<CreateComponentOutput, RuntimeError> {
        let address = self.runtime.new_component_address();

        if self.runtime.get_component(address).is_some() {
            return Err(RuntimeError::ComponentAlreadyExists(address));
        }

        let new_state = self.transform_sbor_data(
            &input.state,
            Self::convert_transient_to_persist,
            Self::reject_references,
        )?;
        trace!(
            self,
            "New component: address = {:02x?}, state = {:02x?}",
            address,
            new_state
        );

        let component = Component::new(self.package()?, input.name, new_state);
        self.runtime.put_component(address, component);

        Ok(CreateComponentOutput { component: address })
    }

    pub fn get_component_info(
        &mut self,
        input: GetComponentInfoInput,
    ) -> Result<GetComponentInfoOutput, RuntimeError> {
        let result = self
            .runtime
            .get_component(input.component)
            .map(|c| ComponentInfo {
                package: c.package().clone(),
                name: c.name().to_string(),
            });
        Ok(GetComponentInfoOutput { result })
    }

    pub fn get_component_state(
        &mut self,
        input: GetComponentStateInput,
    ) -> Result<GetComponentStateOutput, RuntimeError> {
        let component = self
            .runtime
            .get_component(input.component)
            .ok_or(RuntimeError::ComponentNotFound(input.component))?;

        let state = component.state();

        Ok(GetComponentStateOutput {
            state: state.to_owned(),
        })
    }

    pub fn put_component_state(
        &mut self,
        input: PutComponentStateInput,
    ) -> Result<PutComponentStateOutput, RuntimeError> {
        let new_state = self.transform_sbor_data(
            &input.state,
            Self::convert_transient_to_persist,
            Self::reject_references,
        )?;
        trace!(self, "Transformed: {:02x?}", new_state);

        let component = self
            .runtime
            .get_component_mut(input.component)
            .ok_or(RuntimeError::ComponentNotFound(input.component))?;

        component.set_state(new_state);

        Ok(PutComponentStateOutput {})
    }

    pub fn create_resource_mutable(
        &mut self,
        input: CreateResourceMutableInput,
    ) -> Result<CreateResourceMutableOutput, RuntimeError> {
        let info = input.info;
        if info.minter.is_none() || info.supply.is_some() {
            return Err(RuntimeError::InvalidResourceParameter);
        }

        let address = self
            .runtime
            .new_resource_address(self.package()?, info.symbol.as_str());

        if self.runtime.get_resource(address).is_some() {
            return Err(RuntimeError::ResourceAlreadyExists(address));
        } else {
            trace!(self, "New resource: {:02x?}", address);

            self.runtime.put_resource(address, Resource::new(info));
        }
        Ok(CreateResourceMutableOutput { resource: address })
    }

    pub fn create_resource_fixed(
        &mut self,
        input: CreateResourceFixedInput,
    ) -> Result<CreateResourceFixedOutput, RuntimeError> {
        let info = input.info;
        if info.minter.is_some() || info.supply.is_none() {
            return Err(RuntimeError::InvalidResourceParameter);
        }
        let supply = info.supply.clone().unwrap();

        let address = self
            .runtime
            .new_resource_address(self.package()?, info.symbol.as_str());

        if self.runtime.get_resource(address).is_some() {
            return Err(RuntimeError::ResourceAlreadyExists(address));
        } else {
            trace!(self, "New resource: {:02x?}", address);

            self.runtime.put_resource(address, Resource::new(info));
        }

        let bucket = Bucket::new(supply, address);
        let bid = self.runtime.new_transient_bid();
        self.buckets.insert(bid, bucket);

        Ok(CreateResourceFixedOutput {
            resource: address,
            bucket: bid,
        })
    }

    pub fn get_resource_info(
        &mut self,
        input: GetResourceInfoInput,
    ) -> Result<GetResourceInfoOutput, RuntimeError> {
        Ok(GetResourceInfoOutput {
            result: self
                .runtime
                .get_resource(input.resource)
                .map(|r| r.info().clone()),
        })
    }

    pub fn mint_resource(
        &mut self,
        input: MintResourceInput,
    ) -> Result<MintResourceOutput, RuntimeError> {
        let resource = self
            .runtime
            .get_resource(input.resource)
            .ok_or(RuntimeError::ResourceNotFound(input.resource))?
            .info();

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
                return Err(RuntimeError::FixedResourceMintNotAllowed);
            }
        }

        let bucket = Bucket::new(input.amount, input.resource);
        let bid = self.runtime.new_transient_bid();
        self.buckets.insert(bid, bucket);
        Ok(MintResourceOutput { bucket: bid })
    }

    pub fn new_empty_bucket(
        &mut self,
        input: NewEmptyBucketInput,
    ) -> Result<NewEmptyBucketOutput, RuntimeError> {
        let new_bucket = Bucket::new(U256::zero(), input.resource);
        let new_bid = self.runtime.new_transient_bid();
        self.buckets.insert(new_bid, new_bucket);

        Ok(NewEmptyBucketOutput { bucket: new_bid })
    }

    pub fn combine_buckets(
        &mut self,
        input: CombineBucketsInput,
    ) -> Result<CombineBucketsOutput, RuntimeError> {
        // The other bucket needs to be a transient bucket
        let other = self
            .buckets
            .remove(&input.other)
            .ok_or(RuntimeError::BucketNotFound)?;

        let bucket = if input.bucket.is_persisted() {
            self.runtime
                .get_bucket_mut(input.bucket)
                .ok_or(RuntimeError::BucketNotFound)?
        } else {
            self.buckets
                .get_mut(&input.bucket)
                .ok_or(RuntimeError::BucketNotFound)?
        };

        bucket
            .put(other)
            .map_err(|e| RuntimeError::AccountingError(e))?;

        Ok(CombineBucketsOutput {})
    }

    pub fn split_bucket(
        &mut self,
        input: SplitBucketInput,
    ) -> Result<SplitBucketOutput, RuntimeError> {
        let bucket = if input.bucket.is_persisted() {
            self.runtime
                .get_bucket_mut(input.bucket)
                .ok_or(RuntimeError::BucketNotFound)?
        } else {
            self.buckets
                .get_mut(&input.bucket)
                .ok_or(RuntimeError::BucketNotFound)?
        };

        let new_bucket = bucket
            .take(input.amount)
            .map_err(|e| RuntimeError::AccountingError(e))?;
        let new_bid = self.runtime.new_transient_bid();
        self.buckets.insert(new_bid, new_bucket);

        Ok(SplitBucketOutput { bucket: new_bid })
    }

    pub fn get_amount(&mut self, input: GetAmountInput) -> Result<GetAmountOutput, RuntimeError> {
        let bid = input.bucket;
        let amount = self
            .buckets
            .get(&bid)
            .map(|b| b.amount())
            .or(self
                .locked_buckets
                .get(&bid)
                .map(|x| x.borrow().bucket().amount()))
            .ok_or(RuntimeError::BucketNotFound)?;

        Ok(GetAmountOutput { amount })
    }

    pub fn get_resource(
        &mut self,
        input: GetResourceInput,
    ) -> Result<GetResourceOutput, RuntimeError> {
        let bid = input.bucket;
        let resource = self
            .buckets
            .get(&bid)
            .map(|b| b.resource())
            .or(self
                .locked_buckets
                .get(&bid)
                .map(|x| x.borrow().bucket().resource()))
            .ok_or(RuntimeError::BucketNotFound)?;

        Ok(GetResourceOutput { resource })
    }

    pub fn borrow_immutable(
        &mut self,
        input: BorrowImmutableInput,
    ) -> Result<BorrowImmutableOutput, RuntimeError> {
        let bid = input.bucket;
        let rid = self.runtime.new_fixed_rid();
        trace!(self, "Borrowing: bid =  {:02x?}, rid = {:02x?}", bid, rid);

        match self.locked_buckets.get_mut(&bid) {
            Some(bucket) => {
                // re-borrow
                bucket.borrow_mut().brw();
                self.references.insert(rid, bucket.clone());
            }
            None => {
                // first time borrow
                let bucket = Rc::new(RefCell::new(BucketBorrowed::new(
                    bid,
                    self.buckets
                        .remove(&bid)
                        .ok_or(RuntimeError::BucketNotFound)?,
                    1, // once
                )));
                self.references.insert(rid, bucket.clone());
                self.locked_buckets.insert(bid, bucket);
            }
        }

        Ok(BorrowImmutableOutput { reference: rid })
    }

    pub fn drop_reference(
        &mut self,
        input: DropReferenceInput,
    ) -> Result<DropReferenceOutput, RuntimeError> {
        let rid = input.reference;
        if rid.is_mutable() {
            todo!()
        };
        let bucket = self
            .references
            .remove(&rid)
            .ok_or(RuntimeError::ReferenceNotFound)?;
        trace!(self, "Returning {:02x?}: {:02x?}", rid, bucket);

        let new_count = bucket
            .borrow_mut()
            .rtn()
            .map_err(|e| RuntimeError::AccountingError(e))?;
        if new_count == 0 {
            if let Some(b) = self.locked_buckets.remove(&bucket.borrow().bid()) {
                self.buckets
                    .insert(b.borrow().bid(), b.borrow().bucket().clone());
            }
        }

        Ok(DropReferenceOutput {})
    }

    pub fn get_amount_ref(
        &mut self,
        input: GetAmountRefInput,
    ) -> Result<GetAmountRefOutput, RuntimeError> {
        let reference = self
            .references
            .get(&input.reference)
            .ok_or(RuntimeError::ReferenceNotFound)?;

        Ok(GetAmountRefOutput {
            amount: reference.borrow().bucket().amount(),
        })
    }

    pub fn get_resource_ref(
        &mut self,
        input: GetResourceRefInput,
    ) -> Result<GetResourceRefOutput, RuntimeError> {
        let reference = self
            .references
            .get(&input.reference)
            .ok_or(RuntimeError::ReferenceNotFound)?;

        Ok(GetResourceRefOutput {
            resource: reference.borrow().bucket().resource(),
        })
    }

    pub fn withdraw(&mut self, input: WithdrawInput) -> Result<WithdrawOutput, RuntimeError> {
        let address = input.account;

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
            return Err(RuntimeError::UnauthorizedToWithdraw);
        }

        // find the account
        if self.runtime.get_account(address).is_none() {
            self.runtime.put_account(address, Account::new());
        };
        let account = self.runtime.get_account(address).unwrap();

        // look up the bucket
        let bid = match account.get_bucket(input.resource) {
            Some(bid) => *bid,
            None => {
                let bid = self.runtime.new_persisted_bid();
                self.runtime
                    .put_bucket(bid, Bucket::new(U256::zero(), input.resource));

                let acc = self.runtime.get_account_mut(address).unwrap();
                acc.insert_bucket(input.resource, bid);

                bid
            }
        };
        let bucket = self
            .runtime
            .get_bucket_mut(bid)
            .expect("The bucket should exist");

        let new_bucket = bucket
            .take(input.amount)
            .map_err(|e| RuntimeError::AccountingError(e))?;
        let new_bid = self.runtime.new_transient_bid();
        self.buckets.insert(new_bid, new_bucket);

        Ok(WithdrawOutput { bucket: new_bid })
    }

    pub fn deposit(&mut self, input: DepositInput) -> Result<DepositOutput, RuntimeError> {
        let address = input.account;
        let to_deposit = self
            .buckets
            .remove(&input.bucket)
            .ok_or(RuntimeError::BucketNotFound)?;

        // find the account
        if self.runtime.get_account(address).is_none() {
            self.runtime.put_account(address, Account::new());
        };
        let account = self.runtime.get_account(address).unwrap();

        // look up the bucket
        let bid = match account.get_bucket(to_deposit.resource()) {
            Some(bid) => *bid,
            None => {
                let bid = self.runtime.new_persisted_bid();
                self.runtime
                    .put_bucket(bid, Bucket::new(U256::zero(), to_deposit.resource()));

                let acc = self.runtime.get_account_mut(address).unwrap();
                acc.insert_bucket(to_deposit.resource(), bid);

                bid
            }
        };
        let bucket = self
            .runtime
            .get_bucket_mut(bid)
            .expect("The bucket should exist");

        bucket
            .put(to_deposit)
            .map_err(|e| RuntimeError::AccountingError(e))?;

        Ok(DepositOutput {})
    }

    pub fn emit_log(&mut self, input: EmitLogInput) -> Result<EmitLogOutput, RuntimeError> {
        self.runtime.add_log(input.level, input.message);

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

    /// Transform SBOR data by applying function on BID and RID.
    fn transform_sbor_data(
        &mut self,
        data: &Vec<u8>,
        bid_fn: fn(&mut Self, BID) -> Result<BID, RuntimeError>,
        rid_fn: fn(&mut Self, RID) -> Result<RID, RuntimeError>,
    ) -> Result<Vec<u8>, RuntimeError> {
        let mut decoder = Decoder::with_metadata(data);
        let mut encoder = Encoder::with_metadata();

        self.traverse(None, &mut decoder, &mut encoder, bid_fn, rid_fn)?;

        if decoder.remaining() > 0 {
            Err(RuntimeError::InvalidData(DecodeError::NotAllBytesUsed(
                decoder.remaining(),
            )))
        } else {
            Ok(encoder.into())
        }
    }

    /// Traverse SBOR data. TODO: stack overflow
    fn traverse(
        &mut self,
        ty_from_ctx: Option<u8>,
        dec: &mut Decoder,
        enc: &mut Encoder,
        bid_fn: fn(&mut Self, BID) -> Result<BID, RuntimeError>,
        rid_fn: fn(&mut Self, RID) -> Result<RID, RuntimeError>,
    ) -> Result<(), RuntimeError> {
        let ty = match ty_from_ctx {
            Some(t) => t,
            None => {
                let t = dec.read_type().map_err(|e| RuntimeError::InvalidData(e))?;
                enc.write_type(t);
                t
            }
        };

        match ty {
            constants::TYPE_UNIT => self.transform::<()>(dec, enc, |_, v| Ok(v)),
            constants::TYPE_BOOL => self.transform::<bool>(dec, enc, |_, v| Ok(v)),
            constants::TYPE_I8 => self.transform::<i8>(dec, enc, |_, v| Ok(v)),
            constants::TYPE_I16 => self.transform::<i16>(dec, enc, |_, v| Ok(v)),
            constants::TYPE_I32 => self.transform::<i32>(dec, enc, |_, v| Ok(v)),
            constants::TYPE_I64 => self.transform::<i64>(dec, enc, |_, v| Ok(v)),
            constants::TYPE_I128 => self.transform::<i128>(dec, enc, |_, v| Ok(v)),
            constants::TYPE_U8 => self.transform::<u8>(dec, enc, |_, v| Ok(v)),
            constants::TYPE_U16 => self.transform::<u16>(dec, enc, |_, v| Ok(v)),
            constants::TYPE_U32 => self.transform::<u32>(dec, enc, |_, v| Ok(v)),
            constants::TYPE_U64 => self.transform::<u64>(dec, enc, |_, v| Ok(v)),
            constants::TYPE_U128 => self.transform::<u128>(dec, enc, |_, v| Ok(v)),
            constants::TYPE_STRING => self.transform::<String>(dec, enc, |_, v| Ok(v)),
            constants::TYPE_OPTION => {
                // index
                let index = dec.read_index().map_err(|e| RuntimeError::InvalidData(e))?;
                enc.write_index(index as usize);
                // optional value
                match index {
                    0 => Ok(()),
                    1 => self.traverse(None, dec, enc, bid_fn, rid_fn),
                    _ => Err(RuntimeError::InvalidData(DecodeError::InvalidIndex(index))),
                }
            }
            constants::TYPE_BOX => {
                // value
                self.traverse(None, dec, enc, bid_fn, rid_fn)
            }
            constants::TYPE_ARRAY | constants::TYPE_VEC => {
                // element type
                let ele_ty = dec.read_type().map_err(|e| RuntimeError::InvalidData(e))?;
                enc.write_type(ele_ty);
                // length
                let len = dec.read_len().map_err(|e| RuntimeError::InvalidData(e))?;
                enc.write_len(len);
                // values
                for _ in 0..len {
                    self.traverse(Some(ele_ty), dec, enc, bid_fn, rid_fn)?;
                }
                Ok(())
            }
            constants::TYPE_TUPLE => {
                //length
                let len = dec.read_len().map_err(|e| RuntimeError::InvalidData(e))?;
                enc.write_len(len);
                // values
                for _ in 0..len {
                    self.traverse(None, dec, enc, bid_fn, rid_fn)?;
                }
                Ok(())
            }
            constants::TYPE_STRUCT => {
                // fields
                self.traverse(None, dec, enc, bid_fn, rid_fn)
            }
            constants::TYPE_ENUM => {
                // index
                let index = dec.read_index().map_err(|e| RuntimeError::InvalidData(e))?;
                enc.write_index(index as usize);
                // name
                let name = dec.read_name().map_err(|e| RuntimeError::InvalidData(e))?;
                enc.write_name(name.as_str());
                // fields
                self.traverse(None, dec, enc, bid_fn, rid_fn)
            }
            constants::TYPE_FIELDS_NAMED => {
                //length
                let len = dec.read_len().map_err(|e| RuntimeError::InvalidData(e))?;
                enc.write_len(len);
                // named fields
                for _ in 0..len {
                    // name
                    let name = dec.read_name().map_err(|e| RuntimeError::InvalidData(e))?;
                    enc.write_name(name.as_str());
                    // value
                    self.traverse(None, dec, enc, bid_fn, rid_fn)?;
                }
                Ok(())
            }
            constants::TYPE_FIELDS_UNNAMED => {
                //length
                let len = dec.read_len().map_err(|e| RuntimeError::InvalidData(e))?;
                enc.write_len(len);
                // named fields
                for _ in 0..len {
                    // value
                    self.traverse(None, dec, enc, bid_fn, rid_fn)?;
                }
                Ok(())
            }
            constants::TYPE_FIELDS_UNIT => Ok(()),
            // collections
            constants::TYPE_TREE_SET | constants::TYPE_HASH_SET => {
                todo!()
            }
            constants::TYPE_TREE_MAP | constants::TYPE_HASH_MAP => {
                todo!()
            }
            // scrypto types
            constants::TYPE_H256 => self.transform::<H256>(dec, enc, |_, v| Ok(v)),
            constants::TYPE_U256 => self.transform::<U256>(dec, enc, |_, v| Ok(v)),
            constants::TYPE_ADDRESS => self.transform::<Address>(dec, enc, |_, v| Ok(v)),
            constants::TYPE_BID => self.transform::<BID>(dec, enc, bid_fn),
            constants::TYPE_RID => self.transform::<RID>(dec, enc, rid_fn),
            _ => Err(RuntimeError::InvalidData(DecodeError::InvalidType {
                expected: 0xff,
                actual: ty,
            })),
        }
    }

    /// Apply the transform function.
    #[inline]
    fn transform<T: Decode + Encode + std::fmt::Debug>(
        &mut self,
        dec: &mut Decoder,
        enc: &mut Encoder,
        transform: fn(&mut Self, T) -> Result<T, RuntimeError>,
    ) -> Result<(), RuntimeError> {
        transform(
            self,
            T::decode_value(dec).map_err(|e| RuntimeError::InvalidData(e))?,
        )?
        .encode_value(enc);

        Ok(())
    }

    /// Convert transient buckets to persisted buckets
    fn convert_transient_to_persist(&mut self, bid: BID) -> Result<BID, RuntimeError> {
        if bid.is_transient() {
            let bucket = self
                .buckets
                .remove(&bid)
                .ok_or(RuntimeError::BucketNotFound)?;
            let new_bid = self.runtime.new_persisted_bid();
            self.runtime.put_bucket(new_bid, bucket);
            trace!(self, "Converting {:02x?} to {:02x?}", bid, new_bid);
            Ok(new_bid)
        } else {
            Ok(bid)
        }
    }

    /// Remove transient buckets from this process, and reject persisted buckets.
    fn move_transient_reject_persisted(&mut self, bid: BID) -> Result<BID, RuntimeError> {
        if bid.is_transient() {
            let bucket = self
                .buckets
                .remove(&bid)
                .ok_or(RuntimeError::BucketNotFound)?;
            trace!(self, "Moving {:02x?}: {:02x?}", bid, bucket);
            self.moving_buckets.insert(bid, bucket);
            Ok(bid)
        } else {
            Err(RuntimeError::PersistedBucketMoveNotAllowed)
        }
    }

    /// Remove transient buckets from this process, and reject persisted buckets.
    fn move_references(&mut self, rid: RID) -> Result<RID, RuntimeError> {
        let bucket_ref = self
            .references
            .remove(&rid)
            .ok_or(RuntimeError::BucketNotFound)?;
        trace!(self, "Moving {:02x?}: {:02x?}", rid, bucket_ref);
        self.moving_references.insert(rid, bucket_ref);
        Ok(rid)
    }

    /// Remove transient buckets from this process, and reject persisted buckets.
    fn reject_references(&mut self, _: RID) -> Result<RID, RuntimeError> {
        Err(RuntimeError::ReferenceNotAllowed)
    }

    /// Send a byte array to wasm instance.
    fn send_bytes(&mut self, bytes: &[u8]) -> Result<i32, RuntimeError> {
        let result = self.module()?.invoke_export(
            "scrypto_alloc",
            &[RuntimeValue::I32((bytes.len()) as i32)],
            &mut NopExternals,
        );

        match result {
            Ok(Some(RuntimeValue::I32(ptr))) => {
                if self.memory()?.set(ptr as u32, bytes).is_ok() {
                    return Ok(ptr);
                }
            }
            _ => {}
        }

        Err(RuntimeError::UnableToAllocateMemory)
    }

    /// Read a byte array from wasm instance.
    fn read_bytes(&mut self, ptr: i32) -> Result<Vec<u8>, RuntimeError> {
        let a = self
            .memory()?
            .get((ptr - 4) as u32, 4)
            .map_err(|e| RuntimeError::MemoryAccessError(e))?;
        let len = u32::from_le_bytes([a[0], a[1], a[2], a[3]]);

        self.memory()?
            .get(ptr as u32, len as usize)
            .map_err(|e| RuntimeError::MemoryAccessError(e))
    }

    /// Handle a kernel call.
    fn handle<I: Decode + fmt::Debug, O: Encode + fmt::Debug>(
        &mut self,
        args: RuntimeArgs,
        handler: fn(&mut Self, input: I) -> Result<O, RuntimeError>,
        trace: bool,
    ) -> Result<Option<RuntimeValue>, Trap> {
        let now = Instant::now();
        let input_ptr: u32 = args.nth_checked(1)?;
        let input_len: u32 = args.nth_checked(2)?;
        let input_bytes = self
            .memory()?
            .get(input_ptr, input_len as usize)
            .map_err(|e| Trap::from(RuntimeError::MemoryAccessError(e)))?;
        let input: I = scrypto_decode(&input_bytes)
            .map_err(|e| Trap::from(RuntimeError::InvalidRequest(e)))?;
        if trace {
            trace!(self, "{:02x?}", input);
        }

        let output: O = handler(self, input).map_err(Trap::from)?;
        let output_bytes = scrypto_encode(&output);
        let output_ptr = self.send_bytes(&output_bytes).map_err(Trap::from)?;
        if trace {
            trace!(
                self,
                "{:02x?}, processing time = {} ms",
                output,
                now.elapsed().as_millis()
            );
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
                    CALL_BLUEPRINT => self.handle(args, Self::call_blueprint, true),
                    CALL_COMPONENT => self.handle(args, Self::call_component, true),

                    CREATE_COMPONENT => self.handle(args, Self::create_component, true),
                    GET_COMPONENT_INFO => self.handle(args, Self::get_component_info, true),
                    GET_COMPONENT_STATE => self.handle(args, Self::get_component_state, true),
                    PUT_COMPONENT_STATE => self.handle(args, Self::put_component_state, true),

                    CREATE_RESOURCE_MUTABLE => {
                        self.handle(args, Self::create_resource_mutable, true)
                    }
                    CREATE_RESOURCE_FIXED => self.handle(args, Self::create_resource_fixed, true),
                    GET_RESOURCE_INFO => self.handle(args, Self::get_resource_info, true),
                    MINT_RESOURCE => self.handle(args, Self::mint_resource, true),

                    NEW_EMPTY_BUCKET => self.handle(args, Self::new_empty_bucket, true),
                    COMBINE_BUCKETS => self.handle(args, Self::combine_buckets, true),
                    SPLIT_BUCKET => self.handle(args, Self::split_bucket, true),
                    GET_AMOUNT => self.handle(args, Self::get_amount, true),
                    GET_RESOURCE => self.handle(args, Self::get_resource, true),
                    BORROW_IMMUTABLE => self.handle(args, Self::borrow_immutable, true),
                    DROP_REFERENCE => self.handle(args, Self::drop_reference, true),
                    GET_AMOUNT_REF => self.handle(args, Self::get_amount_ref, true),
                    GET_RESOURCE_REF => self.handle(args, Self::get_resource_ref, true),

                    WITHDRAW => self.handle(args, Self::withdraw, true),
                    DEPOSIT => self.handle(args, Self::deposit, true),

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
