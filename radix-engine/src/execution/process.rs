use std::time::Instant;

use hashbrown::HashMap;
use scrypto::buffer::*;
use scrypto::kernel::*;
use scrypto::types::*;
use wasmi::*;

use crate::execution::*;
use crate::ledger::*;
use crate::model::*;

#[derive(Debug)]
pub enum RuntimeError {
    InterpretationError(Error),

    NoBlueprintReturn,

    MemoryCopyError(Error),

    InvalidBlueprintReturn,
}

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

        let output = result.map_err(|e| RuntimeError::InterpretationError(e))?;
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
            _ => Err(RuntimeError::InvalidBlueprintReturn),
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

    pub fn publish_blueprint(&self, input: PublishBlueprintInput) -> PublishBlueprintOutput {
        todo!()
    }

    pub fn call_blueprint(&mut self, input: CallBlueprintInput) -> CallBlueprintOutput {
        todo!()
    }

    pub fn create_component(&mut self, input: CreateComponentInput) -> CreateComponentOutput {
        todo!()
    }

    pub fn get_component_info(&self, input: GetComponentInfoInput) -> GetComponentInfoOutput {
        todo!()
    }

    pub fn get_component_state(
        &mut self,
        input: GetComponentStateInput,
    ) -> GetComponentStateOutput {
        todo!()
    }

    pub fn put_component_state(
        &mut self,
        input: PutComponentStateInput,
    ) -> PutComponentStateOutput {
        todo!()
    }

    pub fn create_resource(&self, input: CreateResourceInput) -> CreateResourceOutput {
        todo!()
    }

    pub fn get_resource_info(&self, input: GetResourceInfoInput) -> GetResourceInfoOutput {
        todo!()
    }

    pub fn mint_tokens(&mut self, input: MintTokensInput) -> MintTokensOutput {
        todo!()
    }

    pub fn combine_tokens(&mut self, input: CombineTokensInput) -> CombineTokensOutput {
        todo!()
    }

    pub fn split_tokens(&mut self, input: SplitTokensInput) -> SplitTokensOutput {
        todo!()
    }

    pub fn mint_badges(&mut self, input: MintBadgesInput) -> MintBadgesOutput {
        todo!()
    }

    pub fn combine_badges(&mut self, input: CombineBadgesInput) -> CombineBadgesOutput {
        todo!()
    }

    pub fn split_badges(&mut self, input: SplitBadgesInput) -> SplitBadgesOutput {
        todo!()
    }

    pub fn borrow_badges(&mut self, input: BorrowBadgesInput) -> BorrowBadgesOutput {
        todo!()
    }

    pub fn return_badges(&mut self, input: ReturnBadgesInput) -> ReturnBadgesOutput {
        todo!()
    }

    pub fn get_tokens_amount(&mut self, input: GetTokensAmountInput) -> GetTokensAmountOutput {
        todo!()
    }

    pub fn get_tokens_resource(
        &mut self,
        input: GetTokensResourceInput,
    ) -> GetTokensResourceOutput {
        todo!()
    }

    pub fn get_badges_amount(&mut self, input: GetBadgesAmountInput) -> GetBadgesAmountOutput {
        todo!()
    }

    pub fn get_badges_resource(
        &mut self,
        input: GetBadgesResourceInput,
    ) -> GetBadgesResourceOutput {
        todo!()
    }

    pub fn withdraw_tokens(&mut self, input: WithdrawTokensInput) -> WithdrawTokensOutput {
        todo!()
    }

    pub fn deposit_tokens(&mut self, input: DepositTokensInput) -> DepositTokensOutput {
        todo!()
    }

    pub fn withdraw_badges(&mut self, input: WithdrawBadgesInput) -> WithdrawBadgesOutput {
        todo!()
    }

    pub fn deposit_badges(&mut self, input: DepositBadgesInput) -> DepositBadgesOutput {
        todo!()
    }

    pub fn emit_log(&self, input: EmitLogInput) -> EmitLogOutput {
        self.log_user(input.level.into(), input.message);

        EmitLogOutput {}
    }

    pub fn get_context_address(&self, _input: GetContextAddressInput) -> GetContextAddressOutput {
        GetContextAddressOutput {
            address: self.blueprint,
        }
    }

    pub fn get_call_data(&self, _input: GetCallDataInput) -> GetCallDataOutput {
        GetCallDataOutput {
            method: self.method.clone(),
            args: self.args.clone(),
        }
    }

    fn send_bytes(&self, bytes: &[u8]) -> u32 {
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
}

macro_rules! handle_operation {
    ($input_t: ident, $output_t: ident, $args: expr, $externals: expr, $handler: expr, $trace: expr) => {{
        let now = Instant::now();
        let input_ptr: u32 = $args.nth_checked(1)?;
        let input_len: u32 = $args.nth_checked(2)?;
        let input_bytes = $externals
            .memory
            .get(input_ptr, input_len as usize)
            .unwrap();
        let input: $input_t = scrypto_decode(&input_bytes).unwrap();
        if $trace {
            $externals.log_kernel(Level::Trace, format!("{:?}", input));
        }

        let output: $output_t = $handler($externals, input);
        let output_bytes = scrypto_encode(&output);
        let output_ptr = $externals.send_bytes(&output_bytes);
        if $trace {
            $externals.log_kernel(
                Level::Trace,
                format!(
                    "output = {:?}, time = {} ms",
                    output,
                    now.elapsed().as_millis()
                ),
            );
        }

        Ok(Some(RuntimeValue::I32(output_ptr as i32)))
    }};
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
                    PUBLISH_BLUEPRINT => {
                        handle_operation!(
                            PublishBlueprintInput,
                            PublishBlueprintOutput,
                            args,
                            self,
                            Process::publish_blueprint,
                            false
                        )
                    }
                    CALL_BLUEPRINT => {
                        handle_operation!(
                            CallBlueprintInput,
                            CallBlueprintOutput,
                            args,
                            self,
                            Process::call_blueprint,
                            true
                        )
                    }
                    CREATE_COMPONENT => {
                        handle_operation!(
                            CreateComponentInput,
                            CreateComponentOutput,
                            args,
                            self,
                            Process::create_component,
                            true
                        )
                    }
                    GET_COMPONENT_INFO => {
                        handle_operation!(
                            GetComponentInfoInput,
                            GetComponentInfoOutput,
                            args,
                            self,
                            Process::get_component_info,
                            true
                        )
                    }
                    GET_COMPONENT_STATE => {
                        handle_operation!(
                            GetComponentStateInput,
                            GetComponentStateOutput,
                            args,
                            self,
                            Process::get_component_state,
                            true
                        )
                    }
                    PUT_COMPONENT_STATE => {
                        handle_operation!(
                            PutComponentStateInput,
                            PutComponentStateOutput,
                            args,
                            self,
                            Process::put_component_state,
                            true
                        )
                    }
                    CREATE_RESOURCE => {
                        handle_operation!(
                            CreateResourceInput,
                            CreateResourceOutput,
                            args,
                            self,
                            Process::create_resource,
                            true
                        )
                    }
                    GET_RESOURCE_INFO => {
                        handle_operation!(
                            GetResourceInfoInput,
                            GetResourceInfoOutput,
                            args,
                            self,
                            Process::get_resource_info,
                            true
                        )
                    }
                    MINT_TOKENS => {
                        handle_operation!(
                            MintTokensInput,
                            MintTokensOutput,
                            args,
                            self,
                            Process::mint_tokens,
                            true
                        )
                    }
                    COMBINE_TOKENS => {
                        handle_operation!(
                            CombineTokensInput,
                            CombineTokensOutput,
                            args,
                            self,
                            Process::combine_tokens,
                            true
                        )
                    }
                    SPLIT_TOKENS => {
                        handle_operation!(
                            SplitTokensInput,
                            SplitTokensOutput,
                            args,
                            self,
                            Process::split_tokens,
                            true
                        )
                    }
                    MINT_BADGES => {
                        handle_operation!(
                            MintBadgesInput,
                            MintBadgesOutput,
                            args,
                            self,
                            Process::mint_badges,
                            true
                        )
                    }
                    COMBINE_BADGES => {
                        handle_operation!(
                            CombineBadgesInput,
                            CombineBadgesOutput,
                            args,
                            self,
                            Process::combine_badges,
                            true
                        )
                    }
                    SPLIT_BADGES => {
                        handle_operation!(
                            SplitBadgesInput,
                            SplitBadgesOutput,
                            args,
                            self,
                            Process::split_badges,
                            true
                        )
                    }
                    BORROW_BADGES => {
                        handle_operation!(
                            BorrowBadgesInput,
                            BorrowBadgesOutput,
                            args,
                            self,
                            Process::borrow_badges,
                            true
                        )
                    }
                    RETURN_BADGES => {
                        handle_operation!(
                            ReturnBadgesInput,
                            ReturnBadgesOutput,
                            args,
                            self,
                            Process::return_badges,
                            true
                        )
                    }
                    GET_TOKENS_AMOUNT => {
                        handle_operation!(
                            GetTokensAmountInput,
                            GetTokensAmountOutput,
                            args,
                            self,
                            Process::get_tokens_amount,
                            true
                        )
                    }
                    GET_TOKENS_RESOURCE => {
                        handle_operation!(
                            GetTokensResourceInput,
                            GetTokensResourceOutput,
                            args,
                            self,
                            Process::get_tokens_resource,
                            true
                        )
                    }
                    GET_BADGES_AMOUNT => {
                        handle_operation!(
                            GetBadgesAmountInput,
                            GetBadgesAmountOutput,
                            args,
                            self,
                            Process::get_badges_amount,
                            true
                        )
                    }
                    GET_BADGES_RESOURCE => {
                        handle_operation!(
                            GetBadgesResourceInput,
                            GetBadgesResourceOutput,
                            args,
                            self,
                            Process::get_badges_resource,
                            true
                        )
                    }
                    WITHDRAW_TOKENS => {
                        handle_operation!(
                            WithdrawTokensInput,
                            WithdrawTokensOutput,
                            args,
                            self,
                            Process::withdraw_tokens,
                            true
                        )
                    }
                    DEPOSIT_TOKENS => {
                        handle_operation!(
                            DepositTokensInput,
                            DepositTokensOutput,
                            args,
                            self,
                            Process::deposit_tokens,
                            true
                        )
                    }
                    WITHDRAW_BADGES => {
                        handle_operation!(
                            WithdrawBadgesInput,
                            WithdrawBadgesOutput,
                            args,
                            self,
                            Process::withdraw_badges,
                            true
                        )
                    }
                    DEPOSIT_BADGES => {
                        handle_operation!(
                            DepositBadgesInput,
                            DepositBadgesOutput,
                            args,
                            self,
                            Process::deposit_badges,
                            true
                        )
                    }
                    EMIT_LOG => {
                        handle_operation!(
                            EmitLogInput,
                            EmitLogOutput,
                            args,
                            self,
                            Process::emit_log,
                            false
                        )
                    }
                    GET_CONTEXT_ADDRESS => {
                        handle_operation!(
                            GetContextAddressInput,
                            GetContextAddressOutput,
                            args,
                            self,
                            Process::get_context_address,
                            true
                        )
                    }
                    GET_CALL_DATA => {
                        handle_operation!(
                            GetCallDataInput,
                            GetCallDataOutput,
                            args,
                            self,
                            Process::get_call_data,
                            true
                        )
                    }
                    _ => panic!("Unknown operation {}", operation),
                }
            }
            _ => panic!("Unimplemented function at {}", index),
        }
    }
}
