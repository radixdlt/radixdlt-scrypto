use std::fmt;
use std::time::Instant;

use hashbrown::HashMap;
use sbor::*;
use scrypto::buffer::*;
use scrypto::kernel::*;
use scrypto::types::*;
use wasmi::*;

use crate::execution::*;
use crate::ledger::*;
use crate::model::*;

#[derive(Debug)]
pub enum RuntimeError {
    ExecutionError(Error),

    MemoryAccessError(Error),

    NoValidBlueprintReturn,

    InvalidOpCode(u32),

    InvalidRequest(DecodeError),

    UnknownHostFunction(usize),

    UnableToAllocateMemory,

    ResourceLeak(Vec<BID>),

    BlueprintAlreadyExists(Address),

    ComponentAlreadyExists(Address),

    ResourceAlreadyExists(Address),

    ComponentNotFound(Address),

    ResourceNotFound(Address),

    ImmutableResource,

    NotAuthorizedToMint,

    BucketNotFound,

    BucketRefNotFound,
}

impl fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl HostError for RuntimeError {}

pub struct Process<'a, L: Ledger> {
    runtime: &'a mut Runtime<L>,
    module: &'a ModuleRef,
    memory: &'a MemoryRef,
    blueprint: Address,
    component: String,
    method: String,
    args: Vec<Vec<u8>>,
    depth: usize,
    buckets: HashMap<BID, Bucket>,
    buckets_lent: HashMap<BID, Bucket>,
    buckets_borrowed: HashMap<BID, BucketRef>,
}

impl<'a, L: Ledger> Process<'a, L> {
    pub fn new(
        runtime: &'a mut Runtime<L>,
        module: &'a ModuleRef,
        memory: &'a MemoryRef,
        blueprint: Address,
        component: String,
        method: String,
        args: Vec<Vec<u8>>,
        depth: usize,
    ) -> Self {
        // TODO: Move all resources passed by args into this process

        Self {
            runtime,
            module,
            memory,
            blueprint,
            component,
            method,
            args,
            depth,
            buckets: HashMap::new(),
            buckets_lent: HashMap::new(),
            buckets_borrowed: HashMap::new(),
        }
    }

    /// Start this process by invoking the component main method.
    pub fn run(&mut self) -> Result<Vec<u8>, RuntimeError> {
        let now = Instant::now();

        let func = format!("{}_{}", self.component, "main");
        self.log(Level::Info, format!("Invoking {}", func));
        let result = self.module.invoke_export(func.as_str(), &[], self);
        let output = match result.map_err(|e| RuntimeError::ExecutionError(e))? {
            Some(RuntimeValue::I32(ptr)) => {
                self.finalize()?;
                self.read_bytes(ptr)
            }
            _ => Err(RuntimeError::NoValidBlueprintReturn),
        };

        self.log(
            Level::Info,
            format!("Time elapsed: {} ms", now.elapsed().as_millis()),
        );
        output
    }

    /// Log a message.
    pub fn log<S: ToString>(&self, level: Level, msg: S) {
        self.runtime
            .logger()
            .log(self.depth, level, msg.to_string());
    }

    pub fn publish_blueprint(
        &mut self,
        input: PublishBlueprintInput,
    ) -> Result<PublishBlueprintOutput, RuntimeError> {
        let address = self.runtime.new_blueprint_address(&input.code);

        match self.runtime.get_blueprint(address) {
            Some(_) => Err(RuntimeError::BlueprintAlreadyExists(address)),
            _ => {
                self.log(
                    Level::Debug,
                    format!(
                        "New blueprint: address = {:?}, code length = {:?}",
                        address,
                        input.code.len()
                    ),
                );
                self.runtime.put_blueprint(address, input.code);

                Ok(PublishBlueprintOutput { blueprint: address })
            }
        }
    }

    pub fn call_blueprint(
        &mut self,
        _input: CallBlueprintInput,
    ) -> Result<CallBlueprintOutput, RuntimeError> {
        todo!()
    }

    pub fn create_component(
        &mut self,
        input: CreateComponentInput,
    ) -> Result<CreateComponentOutput, RuntimeError> {
        let address = self.runtime.new_component_address();

        match self.runtime.get_component(address) {
            Some(_) => Err(RuntimeError::ComponentAlreadyExists(address)),
            _ => {
                // TODO: move resources to the component

                self.log(
                    Level::Debug,
                    format!(
                        "New component: address = {:?}, name = {:?}, state = {:?}",
                        address, input.name, input.state
                    ),
                );
                let component = Component::new(self.blueprint, input.name, input.state);
                self.runtime.put_component(address, component);

                Ok(CreateComponentOutput { component: address })
            }
        }
    }

    pub fn get_component_info(
        &mut self,
        input: GetComponentInfoInput,
    ) -> Result<GetComponentInfoOutput, RuntimeError> {
        let result = self
            .runtime
            .get_component(input.component)
            .map(|c| ComponentInfo {
                blueprint: c.blueprint().clone(),
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

        // TODO: withdraw resource recursively.

        Ok(GetComponentStateOutput {
            state: state.to_owned(),
        })
    }

    pub fn put_component_state(
        &mut self,
        input: PutComponentStateInput,
    ) -> Result<PutComponentStateOutput, RuntimeError> {
        let mut component = self
            .runtime
            .get_component(input.component)
            .ok_or(RuntimeError::ComponentNotFound(input.component))?;

        component.set_state(input.state);
        // TODO: deposit resource recursively.

        Ok(PutComponentStateOutput {})
    }

    pub fn create_resource(
        &mut self,
        input: CreateResourceInput,
    ) -> Result<CreateResourceOutput, RuntimeError> {
        let address = self
            .runtime
            .new_resource_address(self.blueprint, input.info.symbol.as_str());

        if self.runtime.get_resource(address).is_some() {
            return Err(RuntimeError::ResourceAlreadyExists(address));
        } else {
            self.log(Level::Debug, format!("New resource: {:?}", address));

            self.runtime.put_resource(address, input.info);
        }
        Ok(CreateResourceOutput { resource: address })
    }

    pub fn get_resource_info(
        &mut self,
        input: GetResourceInfoInput,
    ) -> Result<GetResourceInfoOutput, RuntimeError> {
        Ok(GetResourceInfoOutput {
            result: self.runtime.get_resource(input.resource).map(Clone::clone),
        })
    }

    pub fn mint_tokens(
        &mut self,
        input: MintTokensInput,
    ) -> Result<MintTokensOutput, RuntimeError> {
        let resource = self
            .runtime
            .get_resource(input.resource)
            .ok_or(RuntimeError::ResourceNotFound(input.resource))?;

        if resource.minter.is_none() {
            Err(RuntimeError::ImmutableResource)
        } else if resource.minter != Some(self.blueprint) {
            Err(RuntimeError::NotAuthorizedToMint)
        } else {
            let bucket = Bucket::new(input.amount, input.resource);
            let bid = self.runtime.new_bid();
            self.buckets.insert(bid, bucket);
            Ok(MintTokensOutput { tokens: bid })
        }
    }

    pub fn combine_tokens(
        &mut self,
        input: CombineTokensInput,
    ) -> Result<CombineTokensOutput, RuntimeError> {
        let other = self
            .buckets
            .remove(&input.other)
            .ok_or(RuntimeError::BucketNotFound)?;
        let one = self
            .buckets
            .get_mut(&input.tokens)
            .ok_or(RuntimeError::BucketNotFound)?;
        one.put(other);

        Ok(CombineTokensOutput {})
    }

    pub fn split_tokens(
        &mut self,
        input: SplitTokensInput,
    ) -> Result<SplitTokensOutput, RuntimeError> {
        let bucket = self
            .buckets
            .get_mut(&input.tokens)
            .ok_or(RuntimeError::BucketNotFound)?;
        let taken = bucket.take(input.amount);
        let bid = self.runtime.new_bid();
        self.buckets.insert(bid, taken);
        Ok(SplitTokensOutput { tokens: bid })
    }

    pub fn borrow_tokens(
        &mut self,
        input: BorrowTokensInput,
    ) -> Result<BorrowTokensOutput, RuntimeError> {
        let bid = input.tokens;
        self.log(Level::Debug, format!("Borrowing {:?}", bid));

        match self.buckets_lent.get_mut(&bid) {
            Some(reference) => {
                // re-borrow
                self.buckets_borrowed
                    .entry(bid)
                    .or_insert(BucketRef::new(reference.amount(), reference.resource(), 0))
                    .increase_count();
            }
            None => {
                // first time borrow
                let bucket = self
                    .buckets
                    .remove(&bid)
                    .ok_or(RuntimeError::BucketNotFound)?;
                self.buckets_borrowed.insert(
                    bid,
                    BucketRef::new(bucket.amount(), bucket.resource().clone(), 1),
                );
                self.buckets_lent.insert(bid, bucket);
            }
        }

        Ok(BorrowTokensOutput { reference: bid })
    }

    pub fn return_tokens(
        &mut self,
        input: ReturnTokensInput,
    ) -> Result<ReturnTokensOutput, RuntimeError> {
        let bid = input.reference;
        self.log(Level::Debug, format!("Returning: {:?}", bid));

        let bucket = self
            .buckets_borrowed
            .get_mut(&bid)
            .ok_or(RuntimeError::BucketRefNotFound)?;

        if bucket.decrease_count() == 0 {
            self.buckets_borrowed.remove(&bid);

            if let Some(b) = self.buckets_lent.remove(&bid) {
                self.buckets.insert(bid, b);
            }
        }

        Ok(ReturnTokensOutput {})
    }

    pub fn mint_badges(
        &mut self,
        input: MintBadgesInput,
    ) -> Result<MintBadgesOutput, RuntimeError> {
        self.mint_tokens(MintTokensInput {
            amount: input.amount,
            resource: input.resource,
        })
        .map(|o| MintBadgesOutput { badges: o.tokens })
    }

    pub fn combine_badges(
        &mut self,
        input: CombineBadgesInput,
    ) -> Result<CombineBadgesOutput, RuntimeError> {
        self.combine_tokens(CombineTokensInput {
            tokens: input.badges,
            other: input.other,
        })
        .map(|_| CombineBadgesOutput {})
    }

    pub fn split_badges(
        &mut self,
        input: SplitBadgesInput,
    ) -> Result<SplitBadgesOutput, RuntimeError> {
        self.split_tokens(SplitTokensInput {
            tokens: input.badges,
            amount: input.amount,
        })
        .map(|o| SplitBadgesOutput { badges: o.tokens })
    }

    pub fn borrow_badges(
        &mut self,
        input: BorrowBadgesInput,
    ) -> Result<BorrowBadgesOutput, RuntimeError> {
        self.borrow_tokens(BorrowTokensInput {
            tokens: input.badges,
        })
        .map(|o| BorrowBadgesOutput {
            reference: o.reference,
        })
    }

    pub fn return_badges(
        &mut self,
        input: ReturnBadgesInput,
    ) -> Result<ReturnBadgesOutput, RuntimeError> {
        self.return_tokens(ReturnTokensInput {
            reference: input.reference,
        })
        .map(|_| ReturnBadgesOutput {})
    }

    pub fn get_tokens_amount(
        &mut self,
        input: GetTokensAmountInput,
    ) -> Result<GetTokensAmountOutput, RuntimeError> {
        todo!()
    }

    pub fn get_tokens_resource(
        &mut self,
        input: GetTokensResourceInput,
    ) -> Result<GetTokensResourceOutput, RuntimeError> {
        todo!()
    }

    pub fn get_badges_amount(
        &mut self,
        input: GetBadgesAmountInput,
    ) -> Result<GetBadgesAmountOutput, RuntimeError> {
        todo!()
    }

    pub fn get_badges_resource(
        &mut self,
        input: GetBadgesResourceInput,
    ) -> Result<GetBadgesResourceOutput, RuntimeError> {
        todo!()
    }

    pub fn withdraw_tokens(
        &mut self,
        input: WithdrawTokensInput,
    ) -> Result<WithdrawTokensOutput, RuntimeError> {
        todo!()
    }

    pub fn deposit_tokens(
        &mut self,
        input: DepositTokensInput,
    ) -> Result<DepositTokensOutput, RuntimeError> {
        todo!()
    }

    pub fn withdraw_badges(
        &mut self,
        input: WithdrawBadgesInput,
    ) -> Result<WithdrawBadgesOutput, RuntimeError> {
        todo!()
    }

    pub fn deposit_badges(
        &mut self,
        input: DepositBadgesInput,
    ) -> Result<DepositBadgesOutput, RuntimeError> {
        todo!()
    }

    pub fn emit_log(&mut self, input: EmitLogInput) -> Result<EmitLogOutput, RuntimeError> {
        self.log(input.level.into(), input.message);

        Ok(EmitLogOutput {})
    }

    pub fn get_context_address(
        &mut self,
        _input: GetContextAddressInput,
    ) -> Result<GetContextAddressOutput, RuntimeError> {
        Ok(GetContextAddressOutput {
            address: self.blueprint,
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

    /// Finalize this process.
    fn finalize(&self) -> Result<(), RuntimeError> {
        let mut buckets = vec![];

        for (bid, bucket) in &self.buckets {
            if bucket.amount() != U256::zero() {
                self.log(
                    Level::Error,
                    format!("Burning bucket: {:?} {:?}", bid, bucket),
                );
                buckets.push(*bid);
            }
        }
        for (bid, bucket) in &self.buckets_lent {
            self.log(Level::Error, format!("Bucket lent: {:?} {:?}", bid, bucket));
            buckets.push(*bid);
        }

        for (bid, bucket_ref) in &self.buckets_borrowed {
            self.log(
                Level::Error,
                format!("Bucket lent: {:?} {:?}", bid, bucket_ref),
            );
            buckets.push(*bid);
        }

        if buckets.is_empty() {
            Ok(())
        } else {
            Err(RuntimeError::ResourceLeak(buckets))
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
            self.log(Level::Trace, format!("input = {:?}", input));
        }

        let output: O = handler(self, input).map_err(Trap::from)?;
        let output_bytes = scrypto_encode(&output);
        let output_ptr = self.send_bytes(&output_bytes).map_err(Trap::from)?;
        if trace {
            self.log(
                Level::Trace,
                format!(
                    "output = {:?}, time = {} ms",
                    output,
                    now.elapsed().as_millis()
                ),
            );
        }

        Ok(Some(RuntimeValue::I32(output_ptr)))
    }
}

impl<'a, T: Ledger> Externals for Process<'a, T> {
    fn invoke_index(
        &mut self,
        index: usize,
        args: RuntimeArgs,
    ) -> Result<Option<RuntimeValue>, Trap> {
        match index {
            KERNEL => {
                let operation: u32 = args.nth_checked(0)?;
                match operation {
                    PUBLISH_BLUEPRINT => self.handle(args, Process::publish_blueprint, false),
                    CALL_BLUEPRINT => self.handle(args, Process::call_blueprint, true),
                    CREATE_COMPONENT => self.handle(args, Process::create_component, true),
                    GET_COMPONENT_INFO => self.handle(args, Process::get_component_info, true),
                    GET_COMPONENT_STATE => self.handle(args, Process::get_component_state, true),
                    PUT_COMPONENT_STATE => self.handle(args, Process::put_component_state, true),
                    CREATE_RESOURCE => self.handle(args, Process::create_resource, true),
                    GET_RESOURCE_INFO => self.handle(args, Process::get_resource_info, true),
                    MINT_TOKENS => self.handle(args, Process::mint_tokens, true),
                    COMBINE_TOKENS => self.handle(args, Process::combine_tokens, true),
                    SPLIT_TOKENS => self.handle(args, Process::split_tokens, true),
                    BORROW_TOKENS => self.handle(args, Process::borrow_tokens, true),
                    RETURN_TOKENS => self.handle(args, Process::return_tokens, true),
                    MINT_BADGES => self.handle(args, Process::mint_badges, true),
                    COMBINE_BADGES => self.handle(args, Process::combine_badges, true),
                    SPLIT_BADGES => self.handle(args, Process::split_badges, true),
                    BORROW_BADGES => self.handle(args, Process::borrow_badges, true),
                    RETURN_BADGES => self.handle(args, Process::return_badges, true),
                    GET_TOKENS_AMOUNT => self.handle(args, Process::get_tokens_amount, true),
                    GET_TOKENS_RESOURCE => self.handle(args, Process::get_tokens_resource, true),
                    GET_BADGES_AMOUNT => self.handle(args, Process::get_badges_amount, true),
                    GET_BADGES_RESOURCE => self.handle(args, Process::get_badges_resource, true),
                    WITHDRAW_TOKENS => self.handle(args, Process::withdraw_tokens, true),
                    DEPOSIT_TOKENS => self.handle(args, Process::deposit_tokens, true),
                    WITHDRAW_BADGES => self.handle(args, Process::withdraw_badges, true),
                    DEPOSIT_BADGES => self.handle(args, Process::deposit_badges, true),
                    EMIT_LOG => self.handle(args, Process::emit_log, false),
                    GET_CONTEXT_ADDRESS => self.handle(args, Process::get_context_address, true),
                    GET_CALL_DATA => self.handle(args, Process::get_call_data, true),
                    _ => Err(RuntimeError::InvalidOpCode(operation).into()),
                }
            }
            _ => Err(RuntimeError::UnknownHostFunction(index).into()),
        }
    }
}
