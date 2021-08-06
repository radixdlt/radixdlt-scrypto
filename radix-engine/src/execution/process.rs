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

    MemoryCopyError(Error),

    NoValidBlueprintReturn,

    InvalidOpCode(u32),

    InvalidRequest,

    UnknownHostFunction(usize),
}

impl fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl HostError for RuntimeError {}

pub struct Process<'a, L: Ledger> {
    context: &'a mut TransactionContext<L>,
    module: &'a ModuleRef,
    memory: &'a MemoryRef,
    blueprint: Address,
    component: String,
    method: String,
    args: Vec<Vec<u8>>,
    depth: usize,
    buckets: HashMap<BID, Bucket>,
    buckets_lent: HashMap<BID, Bucket>,
    buckets_borrowed: HashMap<BID, Bucket>,
}

impl<'a, L: Ledger> Process<'a, L> {
    pub fn new(
        context: &'a mut TransactionContext<L>,
        module: &'a ModuleRef,
        memory: &'a MemoryRef,
        blueprint: Address,
        component: String,
        method: String,
        args: Vec<Vec<u8>>,
        depth: usize,
    ) -> Self {
        Self {
            context,
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

    pub fn run(&mut self) -> Result<Vec<u8>, RuntimeError> {
        let func = format!("{}_{}", self.component, "main");
        let result = self.module.invoke_export(func.as_str(), &[], self);

        let output = result.map_err(|e| RuntimeError::ExecutionError(e))?;
        match output {
            Some(RuntimeValue::I32(ptr)) => {
                let buf = self
                    .memory
                    .get((ptr - 4) as u32, 4)
                    .map_err(|e| RuntimeError::MemoryCopyError(e))?;
                let len = u32::from_le_bytes([buf[0], buf[1], buf[2], buf[3]]);
                let bytes = self
                    .memory
                    .get(ptr as u32, len as usize)
                    .map_err(|e| RuntimeError::MemoryCopyError(e))?;
                Ok(bytes)
            }
            _ => Err(RuntimeError::NoValidBlueprintReturn),
        }
    }

    pub fn log_user<S: ToString>(&self, level: Level, msg: S) {
        self.context
            .logger()
            .log(self.depth, level, msg.to_string());
    }

    pub fn log_kernel<S: ToString>(&self, level: Level, msg: S) {
        self.context
            .logger()
            .log(self.depth, level, msg.to_string());
    }

    pub fn finalize(&self) {
        for (_, bucket) in &self.buckets {
            if bucket.amount() != U256::zero() {
                self.log_kernel(Level::Error, format!("Burning bucket: {:?}", bucket));
            }
        }
        for (_, bucket) in &self.buckets_lent {
            self.log_kernel(Level::Error, format!("Bucket lent: {:?}", bucket));
        }
        for (_, bucket_ref) in &self.buckets_borrowed {
            self.log_kernel(Level::Warn, format!("Bucket borrowed: {:?}", bucket_ref));
        }
    }

    pub fn publish_blueprint(
        &mut self,
        input: PublishBlueprintInput,
    ) -> Result<PublishBlueprintOutput, RuntimeError> {
        todo!()
    }

    pub fn call_blueprint(
        &mut self,
        input: CallBlueprintInput,
    ) -> Result<CallBlueprintOutput, RuntimeError> {
        todo!()
    }

    pub fn create_component(
        &mut self,
        input: CreateComponentInput,
    ) -> Result<CreateComponentOutput, RuntimeError> {
        todo!()
    }

    pub fn get_component_info(
        &mut self,
        input: GetComponentInfoInput,
    ) -> Result<GetComponentInfoOutput, RuntimeError> {
        todo!()
    }

    pub fn get_component_state(
        &mut self,
        input: GetComponentStateInput,
    ) -> Result<GetComponentStateOutput, RuntimeError> {
        todo!()
    }

    pub fn put_component_state(
        &mut self,
        input: PutComponentStateInput,
    ) -> Result<PutComponentStateOutput, RuntimeError> {
        todo!()
    }

    pub fn create_resource(
        &mut self,
        input: CreateResourceInput,
    ) -> Result<CreateResourceOutput, RuntimeError> {
        todo!()
    }

    pub fn get_resource_info(
        &mut self,
        input: GetResourceInfoInput,
    ) -> Result<GetResourceInfoOutput, RuntimeError> {
        todo!()
    }

    pub fn mint_tokens(
        &mut self,
        input: MintTokensInput,
    ) -> Result<MintTokensOutput, RuntimeError> {
        todo!()
    }

    pub fn combine_tokens(
        &mut self,
        input: CombineTokensInput,
    ) -> Result<CombineTokensOutput, RuntimeError> {
        todo!()
    }

    pub fn split_tokens(
        &mut self,
        input: SplitTokensInput,
    ) -> Result<SplitTokensOutput, RuntimeError> {
        todo!()
    }

    pub fn mint_badges(
        &mut self,
        input: MintBadgesInput,
    ) -> Result<MintBadgesOutput, RuntimeError> {
        todo!()
    }

    pub fn combine_badges(
        &mut self,
        input: CombineBadgesInput,
    ) -> Result<CombineBadgesOutput, RuntimeError> {
        todo!()
    }

    pub fn split_badges(
        &mut self,
        input: SplitBadgesInput,
    ) -> Result<SplitBadgesOutput, RuntimeError> {
        todo!()
    }

    pub fn borrow_badges(
        &mut self,
        input: BorrowBadgesInput,
    ) -> Result<BorrowBadgesOutput, RuntimeError> {
        todo!()
    }

    pub fn return_badges(
        &mut self,
        input: ReturnBadgesInput,
    ) -> Result<ReturnBadgesOutput, RuntimeError> {
        todo!()
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
        self.log_user(input.level.into(), input.message);

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

    fn send_bytes(&mut self, bytes: &[u8]) -> u32 {
        let result = self.module.invoke_export(
            "scrypto_alloc",
            &[RuntimeValue::I32((bytes.len()) as i32)],
            &mut NopExternals,
        );

        match result.unwrap().unwrap() {
            RuntimeValue::I32(pointer) => {
                self.memory.set(pointer as u32, bytes).unwrap();
                pointer as u32
            }
            _ => panic!("Failed to allocate memory in process"),
        }
    }

    fn trap(error: RuntimeError) -> Trap {
        Trap::new(TrapKind::Host(Box::new(error)))
    }

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
            .map_err(|e| Trap::new(TrapKind::MemoryAccessOutOfBounds))?;
        let input: I =
            scrypto_decode(&input_bytes).map_err(|e| Self::trap(RuntimeError::InvalidRequest))?;
        if trace {
            self.log_kernel(Level::Trace, format!("{:?}", input));
        }

        let output: O = handler(self, input).map_err(|e| Self::trap(e))?;
        let output_bytes = scrypto_encode(&output);
        let output_ptr = self.send_bytes(&output_bytes);
        if trace {
            self.log_kernel(
                Level::Trace,
                format!(
                    "output = {:?}, time = {} ms",
                    output,
                    now.elapsed().as_millis()
                ),
            );
        }

        Ok(Some(RuntimeValue::I32(output_ptr as i32)))
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
                    _ => Err(Self::trap(RuntimeError::InvalidOpCode(operation))),
                }
            }
            _ => Err(Self::trap(RuntimeError::UnknownHostFunction(index))),
        }
    }
}
