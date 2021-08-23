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
    ($self:ident, $($args: expr),+) => {
        $self.log(Level::Trace, format!($($args),+));
    };
}

macro_rules! info {
    ($self:ident, $($args: expr),+) => {
        $self.log(Level::Info, format!($($args),+));
    };
}

macro_rules! warn {
    ($self:ident, $($args: expr),+) => {
        $self.log(Level::Warn, format!($($args),+));
    };
}

/// A running process
pub struct Process<'m, 'rt, 'le, L: Ledger> {
    runtime: &'rt mut Runtime<'le, L>,
    package: Address,
    blueprint: String,
    method: String,
    args: Vec<Vec<u8>>,
    depth: usize,
    module: &'m ModuleRef,
    memory: &'m MemoryRef,
    buckets: HashMap<BID, Bucket>,
    buckets_borrowed: HashMap<BID, Rc<RefCell<BucketBorrowed>>>,
    buckets_moving: HashMap<BID, Bucket>,
    references: HashMap<RID, Rc<RefCell<BucketBorrowed>>>, // TODO: support persisted bucket borrow
    references_moving: HashMap<RID, Rc<RefCell<BucketBorrowed>>>,
}

impl<'m, 'rt, 'le, L: Ledger> Process<'m, 'rt, 'le, L> {
    pub fn new(
        runtime: &'rt mut Runtime<'le, L>,
        package: Address,
        blueprint: String,
        method: String,
        args: Vec<Vec<u8>>,
        depth: usize,
        module: &'m ModuleRef,
        memory: &'m MemoryRef,
        buckets: HashMap<BID, Bucket>,
        references: HashMap<RID, Rc<RefCell<BucketBorrowed>>>,
    ) -> Self {
        Self {
            runtime,
            package,
            blueprint,
            method,
            args,
            depth,
            module,
            memory,
            buckets,
            buckets_borrowed: HashMap::new(),
            buckets_moving: HashMap::new(),
            references,
            references_moving: HashMap::new(),
        }
    }

    /// Start this process by invoking the main method.
    pub fn run(&mut self) -> Result<Vec<u8>, RuntimeError> {
        let now = Instant::now();
        info!(
            self,
            "CALL started: package = {}, blueprint = {}, method = {}, args = {:02x?}",
            self.package,
            self.blueprint,
            self.method,
            self.args
        );

        let result = self.invoke(self.blueprint.clone());

        info!(
            self,
            "CALL finished: time elapsed = {} ms, result = {:02x?}",
            now.elapsed().as_millis(),
            result
        );
        result
    }

    pub fn invoke(&mut self, func: String) -> Result<Vec<u8>, RuntimeError> {
        let invoke_res = self.module.invoke_export(func.as_str(), &[], self);

        match invoke_res.map_err(|e| RuntimeError::InvokeError(e))? {
            Some(RuntimeValue::I32(ptr)) => {
                let bytes = self.read_bytes(ptr)?;
                self.process_sbor_data(
                    &bytes,
                    Self::move_transient_reject_persisted,
                    Self::move_references,
                )?;
                self.finalize()?;
                Ok(bytes)
            }
            _ => Err(RuntimeError::NoValidReturn),
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

    pub fn call(
        &mut self,
        package: Address,
        blueprint: String,
        method: String,
        args: Vec<Vec<u8>>,
    ) -> Result<Vec<u8>, RuntimeError> {
        // load the code
        let (module, memory) = self
            .runtime
            .load_module(package)
            .ok_or(RuntimeError::PackageNotFound(package))?;

        // move buckets and references
        for arg in &args {
            self.process_sbor_data(
                arg,
                Self::move_transient_reject_persisted,
                Self::move_references,
            )?;
        }
        let buckets_out = self.buckets_moving.clone();
        self.buckets_moving.clear();
        let references_out = self.references_moving.clone();
        self.references_moving.clear();

        // create a process
        let mut process = Process::new(
            self.runtime,
            package,
            format!("{}_main", blueprint),
            method,
            args,
            self.depth + 1,
            &module,
            &memory,
            buckets_out,
            references_out,
        );

        // run!
        let result = process.run();

        // collect buckets and references
        self.buckets.extend(process.buckets_moving.clone());
        process.buckets_moving.clear();
        self.references.extend(process.references_moving.clone());
        process.references_moving.clear();

        // check borrowed buckets
        let bids: Vec<BID> = self
            .buckets_borrowed
            .values()
            .filter(|v| v.borrow().ref_count() == 0)
            .map(|v| v.borrow().bid())
            .collect();
        for bid in bids {
            trace!(self, "Moving {:02x?} to un-borrowed state", bid);
            let bucket_rc = self.buckets_borrowed.remove(&bid).unwrap();
            let bucket = Rc::try_unwrap(bucket_rc).unwrap().into_inner();
            self.buckets.insert(bid, bucket.into());
        }

        result
    }

    pub fn call_blueprint(
        &mut self,
        input: CallBlueprintInput,
    ) -> Result<CallBlueprintOutput, RuntimeError> {
        self.call(input.package, input.name, input.function, input.args)
            .map(|rtn| CallBlueprintOutput { rtn })
    }

    pub fn call_component(
        &mut self,
        input: CallComponentInput,
    ) -> Result<CallComponentOutput, RuntimeError> {
        let component = self
            .runtime
            .get_component(input.component)
            .ok_or(RuntimeError::ComponentNotFound(input.component))?
            .clone();

        let mut args = input.args;
        args.insert(0, scrypto_encode(&input.component));

        self.call(
            component.package(),
            component.name().to_owned(),
            input.method,
            args,
        )
        .map(|rtn| CallComponentOutput { rtn })
    }

    pub fn create_component(
        &mut self,
        input: CreateComponentInput,
    ) -> Result<CreateComponentOutput, RuntimeError> {
        let address = self.runtime.new_component_address();

        if self.runtime.get_component(address).is_some() {
            return Err(RuntimeError::ComponentAlreadyExists(address));
        }

        let new_state = self.process_sbor_data(
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

        let component = Component::new(self.package, input.name, new_state);
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
        let new_state = self.process_sbor_data(
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
            .new_resource_address(self.package, info.symbol.as_str());

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
            .new_resource_address(self.package, info.symbol.as_str());

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
                    Address::Package(_) => address == self.package,
                    Address::Component(_) => {
                        self.runtime
                            .get_component(address)
                            .ok_or(RuntimeError::ComponentNotFound(address))?
                            .package()
                            == self.package
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
                .buckets_borrowed
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
                .buckets_borrowed
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

        match self.buckets_borrowed.get_mut(&bid) {
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
                self.buckets_borrowed.insert(bid, bucket);
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
            if let Some(b) = self.buckets_borrowed.remove(&bucket.borrow().bid()) {
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
            Address::Package(_) => address == self.package,
            Address::Component(_) => {
                self.runtime
                    .get_component(address)
                    .ok_or(RuntimeError::ComponentNotFound(address))?
                    .package()
                    == self.package
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
            address: self.package,
        })
    }

    pub fn get_call_data(
        &mut self,
        _input: GetCallDataInput,
    ) -> Result<GetCallDataOutput, RuntimeError> {
        Ok(GetCallDataOutput {
            method: self.method.clone(),
            args: self.args.clone(),
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

    fn process_sbor_data(
        &mut self,
        state: &Vec<u8>,
        bid_handler: fn(&mut Self, BID) -> Result<BID, RuntimeError>,
        rid_handler: fn(&mut Self, RID) -> Result<RID, RuntimeError>,
    ) -> Result<Vec<u8>, RuntimeError> {
        let mut decoder = Decoder::with_metadata(state);
        let mut encoder = Encoder::with_metadata();

        self.traverse_sbor(None, &mut decoder, &mut encoder, bid_handler, rid_handler)?;

        if decoder.remaining() > 0 {
            // We expect a single SBOR value
            Err(RuntimeError::InvalidSborValue(
                DecodeError::NotAllBytesUsed(decoder.remaining()),
            ))
        } else {
            Ok(encoder.into())
        }
    }

    // TODO: stack overflow
    fn traverse_sbor(
        &mut self,
        ty_from_ctx: Option<u8>,
        dec: &mut Decoder,
        enc: &mut Encoder,
        bid_handler: fn(&mut Self, BID) -> Result<BID, RuntimeError>,
        rid_handler: fn(&mut Self, RID) -> Result<RID, RuntimeError>,
    ) -> Result<(), RuntimeError> {
        let ty = match ty_from_ctx {
            Some(t) => t,
            None => {
                let t = dec.read_type().map_err(RuntimeError::invalid_sbor_data)?;
                enc.write_type(t);
                t
            }
        };

        match ty {
            constants::TYPE_UNIT => self.dte::<()>(dec, enc, |_, v| Ok(v)),
            constants::TYPE_BOOL => self.dte::<bool>(dec, enc, |_, v| Ok(v)),
            constants::TYPE_I8 => self.dte::<i8>(dec, enc, |_, v| Ok(v)),
            constants::TYPE_I16 => self.dte::<i16>(dec, enc, |_, v| Ok(v)),
            constants::TYPE_I32 => self.dte::<i32>(dec, enc, |_, v| Ok(v)),
            constants::TYPE_I64 => self.dte::<i64>(dec, enc, |_, v| Ok(v)),
            constants::TYPE_I128 => self.dte::<i128>(dec, enc, |_, v| Ok(v)),
            constants::TYPE_U8 => self.dte::<u8>(dec, enc, |_, v| Ok(v)),
            constants::TYPE_U16 => self.dte::<u16>(dec, enc, |_, v| Ok(v)),
            constants::TYPE_U32 => self.dte::<u32>(dec, enc, |_, v| Ok(v)),
            constants::TYPE_U64 => self.dte::<u64>(dec, enc, |_, v| Ok(v)),
            constants::TYPE_U128 => self.dte::<u128>(dec, enc, |_, v| Ok(v)),
            constants::TYPE_STRING => self.dte::<String>(dec, enc, |_, v| Ok(v)),
            constants::TYPE_OPTION => {
                // index
                let index = dec.read_index().map_err(RuntimeError::invalid_sbor_data)?;
                enc.write_index(index as usize);
                // optional value
                match index {
                    0 => Ok(()),
                    1 => self.traverse_sbor(None, dec, enc, bid_handler, rid_handler),
                    _ => Err(RuntimeError::invalid_sbor_data(DecodeError::InvalidIndex(
                        index,
                    ))),
                }
            }
            constants::TYPE_BOX => {
                // value
                self.traverse_sbor(None, dec, enc, bid_handler, rid_handler)
            }
            constants::TYPE_ARRAY | constants::TYPE_VEC => {
                // element type
                let ele_ty = dec.read_type().map_err(RuntimeError::invalid_sbor_data)?;
                enc.write_type(ele_ty);
                // length
                let len = dec.read_len().map_err(RuntimeError::invalid_sbor_data)?;
                enc.write_len(len);
                // values
                for _ in 0..len {
                    self.traverse_sbor(Some(ele_ty), dec, enc, bid_handler, rid_handler)?;
                }
                Ok(())
            }
            constants::TYPE_TUPLE => {
                //length
                let len = dec.read_len().map_err(RuntimeError::invalid_sbor_data)?;
                enc.write_len(len);
                // values
                for _ in 0..len {
                    self.traverse_sbor(None, dec, enc, bid_handler, rid_handler)?;
                }
                Ok(())
            }
            constants::TYPE_STRUCT => {
                // fields
                self.traverse_sbor(None, dec, enc, bid_handler, rid_handler)
            }
            constants::TYPE_ENUM => {
                // index
                let index = dec.read_index().map_err(RuntimeError::invalid_sbor_data)?;
                enc.write_index(index as usize);
                // name
                let name = dec.read_name().map_err(RuntimeError::invalid_sbor_data)?;
                enc.write_name(name.as_str());
                // fields
                self.traverse_sbor(None, dec, enc, bid_handler, rid_handler)
            }
            constants::TYPE_FIELDS_NAMED => {
                //length
                let len = dec.read_len().map_err(RuntimeError::invalid_sbor_data)?;
                enc.write_len(len);
                // named fields
                for _ in 0..len {
                    // name
                    let name = dec.read_name().map_err(RuntimeError::invalid_sbor_data)?;
                    enc.write_name(name.as_str());
                    // value
                    self.traverse_sbor(None, dec, enc, bid_handler, rid_handler)?;
                }
                Ok(())
            }
            constants::TYPE_FIELDS_UNNAMED => {
                //length
                let len = dec.read_len().map_err(RuntimeError::invalid_sbor_data)?;
                enc.write_len(len);
                // named fields
                for _ in 0..len {
                    // value
                    self.traverse_sbor(None, dec, enc, bid_handler, rid_handler)?;
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
            constants::TYPE_H256 => self.dte::<H256>(dec, enc, |_, v| Ok(v)),
            constants::TYPE_U256 => self.dte::<U256>(dec, enc, |_, v| Ok(v)),
            constants::TYPE_ADDRESS => self.dte::<Address>(dec, enc, |_, v| Ok(v)),
            constants::TYPE_BID => self.dte::<BID>(dec, enc, bid_handler),
            constants::TYPE_RID => self.dte::<RID>(dec, enc, rid_handler),
            _ => Err(RuntimeError::InvalidSborValue(DecodeError::InvalidType {
                expected: 0xff,
                actual: ty,
            })),
        }
    }

    /// Decode, transform and encode
    #[inline]
    fn dte<T: Decode + Encode + std::fmt::Debug>(
        &mut self,
        dec: &mut Decoder,
        enc: &mut Encoder,
        transform: fn(&mut Self, T) -> Result<T, RuntimeError>,
    ) -> Result<(), RuntimeError> {
        transform(
            self,
            T::decode_value(dec).map_err(RuntimeError::invalid_sbor_data)?,
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
            self.buckets_moving.insert(bid, bucket);
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
        self.references_moving.insert(rid, bucket_ref);
        Ok(rid)
    }

    /// Remove transient buckets from this process, and reject persisted buckets.
    fn reject_references(&mut self, _: RID) -> Result<RID, RuntimeError> {
        Err(RuntimeError::ReferenceNotAllowed)
    }

    /// Finalize this process.
    fn finalize(&self) -> Result<(), RuntimeError> {
        let mut buckets = vec![];
        let mut references = vec![];

        for (bid, bucket) in &self.buckets {
            if bucket.amount() != U256::zero() {
                warn!(self, "Burning bucket: {:02x?} {:02x?}", bid, bucket);
                buckets.push(*bid);
            }
        }
        for (bid, bucket) in &self.buckets_borrowed {
            warn!(self, "Bucket borrowed out: {:02x?} {:02x?}", bid, bucket);
            buckets.push(*bid);
        }
        for (reference, bucket_ref) in &self.references {
            warn!(
                self,
                "Hanging reference: {:02x?} {:02x?}", reference, bucket_ref
            );
            references.push(*reference);
        }

        if buckets.is_empty() && references.is_empty() {
            Ok(())
        } else {
            Err(RuntimeError::ResourceLeak(buckets, references))
        }
    }

    /// Send a byte array to this process.
    fn send_bytes(&mut self, bytes: &[u8]) -> Result<i32, RuntimeError> {
        let result = self.module.invoke_export(
            "scrypto_alloc",
            &[RuntimeValue::I32((bytes.len()) as i32)],
            &mut NopExternals,
        );

        match result {
            Ok(Some(RuntimeValue::I32(ptr))) => {
                if self.memory.set(ptr as u32, bytes).is_ok() {
                    return Ok(ptr);
                }
            }
            _ => {}
        }

        Err(RuntimeError::UnableToAllocateMemory)
    }

    /// Read a length-prefixed byte array from this process.
    fn read_bytes(&mut self, ptr: i32) -> Result<Vec<u8>, RuntimeError> {
        let a = self
            .memory
            .get((ptr - 4) as u32, 4)
            .map_err(|e| RuntimeError::MemoryAccessError(e))?;
        let len = u32::from_le_bytes([a[0], a[1], a[2], a[3]]);

        self.memory
            .get(ptr as u32, len as usize)
            .map_err(|e| RuntimeError::MemoryAccessError(e))
    }

    /// Log a message to console.
    fn log(&self, level: Level, msg: String) {
        let (l, m) = match level {
            Level::Error => ("ERROR".red(), msg.to_string().red()),
            Level::Warn => ("WARN".yellow(), msg.to_string().yellow()),
            Level::Info => ("INFO".green(), msg.to_string().green()),
            Level::Debug => ("DEBUG".cyan(), msg.to_string().cyan()),
            Level::Trace => ("TRACE".normal(), msg.to_string().normal()),
        };

        println!("{}[{:5}] {}", "  ".repeat(self.depth), l, m);
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
            .memory
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

impl<'m, 'rt, 'le, T: Ledger> Externals for Process<'m, 'rt, 'le, T> {
    fn invoke_index(
        &mut self,
        index: usize,
        args: RuntimeArgs,
    ) -> Result<Option<RuntimeValue>, Trap> {
        match index {
            KERNEL_INDEX => {
                let operation: u32 = args.nth_checked(0)?;
                match operation {
                    PUBLISH => self.handle(args, Process::publish, false),
                    CALL_BLUEPRINT => self.handle(args, Process::call_blueprint, true),
                    CALL_COMPONENT => self.handle(args, Process::call_component, true),

                    CREATE_COMPONENT => self.handle(args, Process::create_component, true),
                    GET_COMPONENT_INFO => self.handle(args, Process::get_component_info, true),
                    GET_COMPONENT_STATE => self.handle(args, Process::get_component_state, true),
                    PUT_COMPONENT_STATE => self.handle(args, Process::put_component_state, true),

                    CREATE_RESOURCE_MUTABLE => {
                        self.handle(args, Process::create_resource_mutable, true)
                    }
                    CREATE_RESOURCE_FIXED => {
                        self.handle(args, Process::create_resource_fixed, true)
                    }
                    GET_RESOURCE_INFO => self.handle(args, Process::get_resource_info, true),
                    MINT_RESOURCE => self.handle(args, Process::mint_resource, true),

                    NEW_EMPTY_BUCKET => self.handle(args, Process::new_empty_bucket, true),
                    COMBINE_BUCKETS => self.handle(args, Process::combine_buckets, true),
                    SPLIT_BUCKET => self.handle(args, Process::split_bucket, true),
                    GET_AMOUNT => self.handle(args, Process::get_amount, true),
                    GET_RESOURCE => self.handle(args, Process::get_resource, true),
                    BORROW_IMMUTABLE => self.handle(args, Process::borrow_immutable, true),
                    DROP_REFERENCE => self.handle(args, Process::drop_reference, true),
                    GET_AMOUNT_REF => self.handle(args, Process::get_amount_ref, true),
                    GET_RESOURCE_REF => self.handle(args, Process::get_resource_ref, true),

                    WITHDRAW => self.handle(args, Process::withdraw, true),
                    DEPOSIT => self.handle(args, Process::deposit, true),

                    EMIT_LOG => self.handle(args, Process::emit_log, true),
                    GET_PACKAGE_ADDRESS => self.handle(args, Process::get_package_address, true),
                    GET_CALL_DATA => self.handle(args, Process::get_call_data, true),
                    GET_TRANSACTION_HASH => self.handle(args, Process::get_transaction_hash, true),
                    _ => Err(RuntimeError::InvalidOpCode(operation).into()),
                }
            }
            _ => Err(RuntimeError::UnknownHostFunction(index).into()),
        }
    }
}
