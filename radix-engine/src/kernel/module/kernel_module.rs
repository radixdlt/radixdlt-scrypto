use crate::errors::*;
use crate::kernel::*;
use crate::system::kernel_modules::execution_trace::ExecutionTraceModule;
use crate::system::kernel_modules::fee::CostingModule;
use crate::system::kernel_modules::fee::FeeReserve;
use crate::system::kernel_modules::kernel_trace::KernelTraceModule;
use crate::system::kernel_modules::royalty::RoyaltyModule;
use crate::transaction::ExecutionConfig;
use sbor::rust::vec::Vec;

use super::BaseModule;
use super::KernelApiCallInput;
use super::KernelApiCallOutput;

pub struct KernelModule {
    trace: bool,
    execution_trace: ExecutionTraceModule,
    costing: CostingModule,
    royalty: RoyaltyModule,
}

impl KernelModule {
    pub fn new(config: &ExecutionConfig) -> Self {
        Self {
            trace: config.trace,
            execution_trace: ExecutionTraceModule::new(config.max_sys_call_trace_depth),
            royalty: RoyaltyModule::default(),
            costing: CostingModule::new(config.max_call_depth),
        }
    }
}

impl KernelModule {
    pub fn collect_events(&mut self) -> Vec<TrackedEvent> {
        self.execution_trace.collect_events()
    }
}

impl<R: FeeReserve> BaseModule<R> for KernelModule {
    fn pre_kernel_api_call(
        &mut self,
        call_frame: &CallFrame,
        heap: &mut Heap,
        track: &mut Track<R>,
        input: KernelApiCallInput,
    ) -> Result<(), ModuleError> {
        if self.trace {
            KernelTraceModule.pre_kernel_api_call(call_frame, heap, track, input.clone())?;
        }
        self.costing
            .pre_kernel_api_call(call_frame, heap, track, input.clone())?;
        self.royalty
            .pre_kernel_api_call(call_frame, heap, track, input.clone())?;
        self.execution_trace
            .pre_kernel_api_call(call_frame, heap, track, input.clone())?;

        Ok(())
    }

    fn post_kernel_api_call(
        &mut self,
        call_frame: &CallFrame,
        heap: &mut Heap,
        track: &mut Track<R>,
        output: KernelApiCallOutput,
    ) -> Result<(), ModuleError> {
        if self.trace {
            KernelTraceModule.post_kernel_api_call(call_frame, heap, track, output.clone())?;
        }
        self.costing
            .post_kernel_api_call(call_frame, heap, track, output.clone())?;
        self.royalty
            .post_kernel_api_call(call_frame, heap, track, output.clone())?;
        self.execution_trace
            .post_kernel_api_call(call_frame, heap, track, output.clone())?;

        Ok(())
    }

    fn pre_execute_invocation(
        &mut self,
        actor: &ResolvedActor,
        call_frame_update: &CallFrameUpdate,
        call_frame: &CallFrame,
        heap: &mut Heap,
        track: &mut Track<R>,
    ) -> Result<(), ModuleError> {
        if self.trace {
            KernelTraceModule.pre_execute_invocation(
                actor,
                call_frame_update,
                call_frame,
                heap,
                track,
            )?;
        }
        self.costing
            .pre_execute_invocation(actor, call_frame_update, call_frame, heap, track)?;
        self.royalty
            .pre_execute_invocation(actor, call_frame_update, call_frame, heap, track)?;
        self.execution_trace.pre_execute_invocation(
            actor,
            call_frame_update,
            call_frame,
            heap,
            track,
        )?;

        Ok(())
    }

    fn post_execute_invocation(
        &mut self,
        caller: &ResolvedActor,
        update: &CallFrameUpdate,
        call_frame: &CallFrame,
        heap: &mut Heap,
        track: &mut Track<R>,
    ) -> Result<(), ModuleError> {
        if self.trace {
            KernelTraceModule.post_execute_invocation(caller, update, call_frame, heap, track)?;
        }
        self.costing
            .post_execute_invocation(caller, update, call_frame, heap, track)?;
        self.royalty
            .post_execute_invocation(caller, update, call_frame, heap, track)?;
        self.execution_trace
            .post_execute_invocation(caller, update, call_frame, heap, track)?;

        Ok(())
    }

    fn on_wasm_instantiation(
        &mut self,
        call_frame: &CallFrame,
        heap: &mut Heap,
        track: &mut Track<R>,
        code: &[u8],
    ) -> Result<(), ModuleError> {
        if self.trace {
            KernelTraceModule.on_wasm_instantiation(call_frame, heap, track, code)?;
        }
        self.costing
            .on_wasm_instantiation(call_frame, heap, track, code)?;
        self.royalty
            .on_wasm_instantiation(call_frame, heap, track, code)?;
        self.execution_trace
            .on_wasm_instantiation(call_frame, heap, track, code)?;

        Ok(())
    }

    fn on_wasm_costing(
        &mut self,
        call_frame: &CallFrame,
        heap: &mut Heap,
        track: &mut Track<R>,
        units: u32,
    ) -> Result<(), ModuleError> {
        if self.trace {
            KernelTraceModule.on_wasm_costing(call_frame, heap, track, units)?;
        }
        self.costing
            .on_wasm_costing(call_frame, heap, track, units)?;
        self.royalty
            .on_wasm_costing(call_frame, heap, track, units)?;
        self.execution_trace
            .on_wasm_costing(call_frame, heap, track, units)?;

        Ok(())
    }
}
