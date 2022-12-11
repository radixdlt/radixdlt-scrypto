use crate::engine::call_frame::RENodeLocation;
use crate::engine::*;
use crate::fee::FeeReserve;
use crate::model::Resource;
use crate::types::*;
use radix_engine_interface::api::types::{
    Level, LockHandle, RENodeId, SubstateId, SubstateOffset, VaultId,
};
use sbor::rust::fmt::Debug;

#[derive(Clone)]
pub enum SysCallInput<'a> {
    Invoke {
        invocation: &'a dyn Debug,
        input_size: u32,
        value_count: u32,
        depth: usize,
    },
    ReadOwnedNodes,
    BorrowNode {
        node_id: &'a RENodeId,
    },
    DropNode {
        node_id: &'a RENodeId,
    },
    CreateNode {
        node: &'a RENode,
    },
    LockSubstate {
        node_id: &'a RENodeId,
        offset: &'a SubstateOffset,
        flags: &'a LockFlags,
    },
    GetRef {
        lock_handle: &'a LockHandle,
    },
    GetRefMut {
        lock_handle: &'a LockHandle,
    },
    DropLock {
        lock_handle: &'a LockHandle,
    },
    TakeSubstate {
        substate_id: &'a SubstateId,
    },
    ReadBlob {
        blob_hash: &'a Hash,
    },
    EmitLog {
        level: &'a Level,
        message: &'a String,
    },
    EmitEvent {
        event: &'a Event<'a>,
    },
}

#[derive(Debug, Clone)]
pub enum SysCallOutput<'a> {
    Invoke { rtn: &'a dyn Debug },
    ReadOwnedNodes,
    BorrowNode { node_pointer: &'a RENodeLocation },
    DropNode { node: &'a HeapRENode },
    CreateNode { node_id: &'a RENodeId },
    LockSubstate { lock_handle: LockHandle },
    GetRef { lock_handle: LockHandle },
    GetRefMut,
    DropLock,
    ReadBlob { blob: &'a [u8] },
    GenerateUuid { uuid: u128 },
    EmitLog,
    EmitEvent,
}

pub trait Module<R: FeeReserve> {
    fn pre_sys_call(
        &mut self,
        _call_frame: &CallFrame,
        _heap: &mut Heap,
        _track: &mut Track<R>,
        _input: SysCallInput,
    ) -> Result<(), ModuleError> {
        Ok(())
    }

    fn post_sys_call(
        &mut self,
        _call_frame: &CallFrame,
        _heap: &mut Heap,
        _track: &mut Track<R>,
        _output: SysCallOutput,
    ) -> Result<(), ModuleError> {
        Ok(())
    }

    fn pre_execute_invocation(
        &mut self,
        _actor: &ResolvedActor,
        _call_frame_update: &CallFrameUpdate,
        _call_frame: &CallFrame,
        _heap: &mut Heap,
        _track: &mut Track<R>,
    ) -> Result<(), ModuleError> {
        Ok(())
    }

    fn post_execute_invocation(
        &mut self,
        _caller: &ResolvedActor,
        _update: &CallFrameUpdate,
        _call_frame: &CallFrame,
        _heap: &mut Heap,
        _track: &mut Track<R>,
    ) -> Result<(), ModuleError> {
        Ok(())
    }

    fn on_wasm_instantiation(
        &mut self,
        _call_frame: &CallFrame,
        _heap: &mut Heap,
        _track: &mut Track<R>,
        _code: &[u8],
    ) -> Result<(), ModuleError> {
        Ok(())
    }

    fn on_wasm_costing(
        &mut self,
        _call_frame: &CallFrame,
        _heap: &mut Heap,
        _track: &mut Track<R>,
        _units: u32,
    ) -> Result<(), ModuleError> {
        Ok(())
    }

    fn on_lock_fee(
        &mut self,
        _call_frame: &CallFrame,
        _eap: &mut Heap,
        _rack: &mut Track<R>,
        _ault_id: VaultId,
        fee: Resource,
        _ontingent: bool,
    ) -> Result<Resource, ModuleError> {
        Ok(fee)
    }

    fn on_finished_processing(
        &mut self,
        _heap: &mut Heap,
        _track: &mut Track<R>,
    ) -> Result<(), ModuleError> {
        Ok(())
    }
}

pub struct KernelModule {
    trace: bool,
    execution_trace: ExecutionTraceModule,
    costing: CostingModule,
    royalty: RoyaltyModule,
}

impl KernelModule {
    pub fn new(trace: bool, max_sys_call_trace_depth: usize) -> Self {
        Self {
            trace,
            execution_trace: ExecutionTraceModule::new(max_sys_call_trace_depth),
            royalty: RoyaltyModule::default(),
            costing: CostingModule::default(),
        }
    }
}

impl<R: FeeReserve> Module<R> for KernelModule {
    fn pre_sys_call(
        &mut self,
        call_frame: &CallFrame,
        heap: &mut Heap,
        track: &mut Track<R>,
        input: SysCallInput,
    ) -> Result<(), ModuleError> {
        if self.trace {
            LoggerModule.pre_sys_call(call_frame, heap, track, input.clone())?;
        }
        self.costing
            .pre_sys_call(call_frame, heap, track, input.clone())?;
        self.royalty
            .pre_sys_call(call_frame, heap, track, input.clone())?;
        self.execution_trace
            .pre_sys_call(call_frame, heap, track, input.clone())?;

        Ok(())
    }

    fn post_sys_call(
        &mut self,
        call_frame: &CallFrame,
        heap: &mut Heap,
        track: &mut Track<R>,
        output: SysCallOutput,
    ) -> Result<(), ModuleError> {
        if self.trace {
            LoggerModule.post_sys_call(call_frame, heap, track, output.clone())?;
        }
        self.costing
            .post_sys_call(call_frame, heap, track, output.clone())?;
        self.royalty
            .post_sys_call(call_frame, heap, track, output.clone())?;
        self.execution_trace
            .post_sys_call(call_frame, heap, track, output.clone())?;

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
            LoggerModule.pre_execute_invocation(
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
            LoggerModule.post_execute_invocation(caller, update, call_frame, heap, track)?;
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
            LoggerModule.on_wasm_instantiation(call_frame, heap, track, code)?;
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
            LoggerModule.on_wasm_costing(call_frame, heap, track, units)?;
        }
        self.costing
            .on_wasm_costing(call_frame, heap, track, units)?;
        self.royalty
            .on_wasm_costing(call_frame, heap, track, units)?;
        self.execution_trace
            .on_wasm_costing(call_frame, heap, track, units)?;

        Ok(())
    }

    fn on_lock_fee(
        &mut self,
        call_frame: &CallFrame,
        heap: &mut Heap,
        track: &mut Track<R>,
        vault_id: VaultId,
        mut fee: Resource,
        contingent: bool,
    ) -> Result<Resource, ModuleError> {
        if self.trace {
            fee = LoggerModule.on_lock_fee(call_frame, heap, track, vault_id, fee, contingent)?;
        }
        fee = self
            .costing
            .on_lock_fee(call_frame, heap, track, vault_id, fee, contingent)?;
        fee = self
            .royalty
            .on_lock_fee(call_frame, heap, track, vault_id, fee, contingent)?;
        fee = self
            .execution_trace
            .on_lock_fee(call_frame, heap, track, vault_id, fee, contingent)?;

        Ok(fee)
    }

    fn on_finished_processing(
        &mut self,
        heap: &mut Heap,
        track: &mut Track<R>,
    ) -> Result<(), ModuleError> {
        if self.trace {
            LoggerModule.on_finished_processing(heap, track)?;
        }
        self.costing.on_finished_processing(heap, track)?;
        self.royalty.on_finished_processing(heap, track)?;
        self.execution_trace.on_finished_processing(heap, track)?;

        Ok(())
    }
}
