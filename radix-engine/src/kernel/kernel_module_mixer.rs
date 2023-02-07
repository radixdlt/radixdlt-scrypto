use super::KernelModule;
use crate::blueprints::fee_reserve::FeeReserveSubstate;
use crate::errors::*;
use crate::kernel::*;
use crate::system::kernel_modules::auth::auth_module::AuthModule;
use crate::system::kernel_modules::costing::CostingModule;
use crate::system::kernel_modules::costing::ExecutionCostingModule;
use crate::system::kernel_modules::costing::FeeTable;
use crate::system::kernel_modules::costing::RoyaltyCostingModule;
use crate::system::kernel_modules::costing::SystemLoanFeeReserve;
use crate::system::kernel_modules::execution_trace::ExecutionTraceModule;
use crate::system::kernel_modules::execution_trace::VaultOp;
use crate::system::kernel_modules::kernel_trace::KernelTraceModule;
use crate::system::kernel_modules::logger::LoggerModule;
use crate::system::kernel_modules::node_move::NodeMoveModule;
use crate::system::kernel_modules::transaction_runtime::TransactionRuntimeModule;
use crate::system::node::RENodeInit;
use crate::system::node::RENodeModuleInit;
use crate::transaction::ExecutionConfig;
use radix_engine_interface::api::types::FnIdentifier;
use radix_engine_interface::api::types::LockHandle;
use radix_engine_interface::api::types::NodeModuleId;
use radix_engine_interface::api::types::RENodeId;
use radix_engine_interface::api::types::RENodeType;
use radix_engine_interface::api::types::SubstateOffset;
use radix_engine_interface::api::types::VaultId;
use radix_engine_interface::crypto::Hash;
use sbor::rust::collections::BTreeMap;
use sbor::rust::vec::Vec;
use transaction::model::AuthZoneParams;

pub struct KernelModuleMixer {
    kernel_trace: bool,
    execution_trace: ExecutionTraceModule,
    execution_costing: ExecutionCostingModule,
    royalty_costing: RoyaltyCostingModule,
}

impl KernelModuleMixer {
    pub fn new(config: &ExecutionConfig) -> Self {
        Self {
            kernel_trace: config.kernel_trace,
            execution_trace: ExecutionTraceModule::new(config.max_kernel_call_depth_traced),
            execution_costing: ExecutionCostingModule::new(config.max_call_depth),
            royalty_costing: RoyaltyCostingModule::default(),
        }
    }
}

impl KernelModuleMixer {
    pub fn destroy(self) -> (Vec<(ResolvedActor, VaultId, VaultOp)>, Vec<TrackedEvent>) {
        self.execution_trace.destroy()
    }
}

impl KernelModule for KernelModuleMixer {
    fn pre_kernel_invoke(
        &mut self,
        current_frame: &CallFrame,
        heap: &mut Heap,
        track: &mut Track,
        fn_identifier: &FnIdentifier,
        input_size: usize,
    ) -> Result<(), ModuleError> {
        if self.kernel_trace {
            KernelTraceModule.pre_kernel_invoke(
                current_frame,
                heap,
                track,
                fn_identifier,
                input_size,
            )?;
        }
        self.execution_costing.pre_kernel_invoke(
            current_frame,
            heap,
            track,
            fn_identifier,
            input_size,
        )?;
        self.royalty_costing.pre_kernel_invoke(
            current_frame,
            heap,
            track,
            fn_identifier,
            input_size,
        )?;
        self.execution_trace.pre_kernel_invoke(
            current_frame,
            heap,
            track,
            fn_identifier,
            input_size,
        )?;

        Ok(())
    }

    fn post_kernel_invoke(
        &mut self,
        current_frame: &CallFrame,
        heap: &mut Heap,
        track: &mut Track,
        output_size: usize,
    ) -> Result<(), ModuleError> {
        if self.kernel_trace {
            KernelTraceModule.post_kernel_invoke(current_frame, heap, track, output_size)?;
        }
        self.execution_costing
            .post_kernel_invoke(current_frame, heap, track, output_size)?;
        self.royalty_costing
            .post_kernel_invoke(current_frame, heap, track, output_size)?;
        self.execution_trace
            .post_kernel_invoke(current_frame, heap, track, output_size)?;

        Ok(())
    }

    fn pre_kernel_execute(
        &mut self,
        current_frame: &CallFrame,
        heap: &mut Heap,
        track: &mut Track,
        callee: &ResolvedActor,
        update: &CallFrameUpdate,
    ) -> Result<(), ModuleError> {
        if self.kernel_trace {
            KernelTraceModule.pre_kernel_execute(current_frame, heap, track, callee, update)?;
        }
        self.execution_costing
            .pre_kernel_execute(current_frame, heap, track, callee, update)?;
        self.royalty_costing
            .pre_kernel_execute(current_frame, heap, track, callee, update)?;
        self.execution_trace
            .pre_kernel_execute(current_frame, heap, track, callee, update)?;

        Ok(())
    }

    fn post_kernel_execute(
        &mut self,
        current_frame: &CallFrame,
        heap: &mut Heap,
        track: &mut Track,
        caller: &ResolvedActor,
        update: &CallFrameUpdate,
    ) -> Result<(), ModuleError> {
        if self.kernel_trace {
            KernelTraceModule.post_kernel_execute(current_frame, heap, track, caller, update)?;
        }
        self.execution_costing
            .post_kernel_execute(current_frame, heap, track, caller, update)?;
        self.royalty_costing
            .post_kernel_execute(current_frame, heap, track, caller, update)?;
        self.execution_trace
            .post_kernel_execute(current_frame, heap, track, caller, update)?;

        Ok(())
    }

    fn on_allocate_node_id(
        &mut self,
        current_frame: &CallFrame,
        heap: &mut Heap,
        track: &mut Track,
        node_type: &RENodeType,
    ) -> Result<(), ModuleError> {
        if self.kernel_trace {
            KernelTraceModule.on_allocate_node_id(current_frame, heap, track, node_type)?;
        }
        self.execution_costing
            .on_allocate_node_id(current_frame, heap, track, node_type)?;
        self.royalty_costing
            .on_allocate_node_id(current_frame, heap, track, node_type)?;
        self.execution_trace
            .on_allocate_node_id(current_frame, heap, track, node_type)?;
        Ok(())
    }

    fn pre_create_node(
        &mut self,
        current_frame: &CallFrame,
        heap: &mut Heap,
        track: &mut Track,
        node_id: &RENodeId,
        node_init: &RENodeInit,
        node_module_init: &BTreeMap<NodeModuleId, RENodeModuleInit>,
    ) -> Result<(), ModuleError> {
        if self.kernel_trace {
            KernelTraceModule.pre_create_node(
                current_frame,
                heap,
                track,
                node_id,
                node_init,
                node_module_init,
            )?;
        }
        self.execution_costing.pre_create_node(
            current_frame,
            heap,
            track,
            node_id,
            node_init,
            node_module_init,
        )?;
        self.royalty_costing.pre_create_node(
            current_frame,
            heap,
            track,
            node_id,
            node_init,
            node_module_init,
        )?;
        self.execution_trace.pre_create_node(
            current_frame,
            heap,
            track,
            node_id,
            node_init,
            node_module_init,
        )?;
        Ok(())
    }

    fn post_create_node(
        &mut self,
        current_frame: &CallFrame,
        heap: &mut Heap,
        track: &mut Track,
        node_id: &RENodeId,
    ) -> Result<(), ModuleError> {
        if self.kernel_trace {
            KernelTraceModule.post_create_node(current_frame, heap, track, node_id)?;
        }
        self.execution_costing
            .post_create_node(current_frame, heap, track, node_id)?;
        self.royalty_costing
            .post_create_node(current_frame, heap, track, node_id)?;
        self.execution_trace
            .post_create_node(current_frame, heap, track, node_id)?;
        Ok(())
    }

    fn pre_drop_node(
        &mut self,
        current_frame: &CallFrame,
        heap: &mut Heap,
        track: &mut Track,
        node_id: &RENodeId,
    ) -> Result<(), ModuleError> {
        if self.kernel_trace {
            KernelTraceModule.pre_drop_node(current_frame, heap, track, node_id)?;
        }
        self.execution_costing
            .pre_drop_node(current_frame, heap, track, node_id)?;
        self.royalty_costing
            .pre_drop_node(current_frame, heap, track, node_id)?;
        self.execution_trace
            .pre_drop_node(current_frame, heap, track, node_id)?;
        Ok(())
    }

    fn post_drop_node(
        &mut self,
        current_frame: &CallFrame,
        heap: &mut Heap,
        track: &mut Track,
    ) -> Result<(), ModuleError> {
        if self.kernel_trace {
            KernelTraceModule.post_drop_node(current_frame, heap, track)?;
        }
        self.execution_costing
            .post_drop_node(current_frame, heap, track)?;
        self.royalty_costing
            .post_drop_node(current_frame, heap, track)?;
        self.execution_trace
            .post_drop_node(current_frame, heap, track)?;
        Ok(())
    }

    fn on_lock_substate(
        &mut self,
        current_frame: &CallFrame,
        heap: &mut Heap,
        track: &mut Track,
        node_id: &RENodeId,
        module_id: &NodeModuleId,
        offset: &SubstateOffset,
        flags: &LockFlags,
    ) -> Result<(), ModuleError> {
        if self.kernel_trace {
            KernelTraceModule.on_lock_substate(
                current_frame,
                heap,
                track,
                node_id,
                module_id,
                offset,
                flags,
            )?;
        }
        self.execution_costing.on_lock_substate(
            current_frame,
            heap,
            track,
            node_id,
            module_id,
            offset,
            flags,
        )?;
        self.royalty_costing.on_lock_substate(
            current_frame,
            heap,
            track,
            node_id,
            module_id,
            offset,
            flags,
        )?;
        self.execution_trace.on_lock_substate(
            current_frame,
            heap,
            track,
            node_id,
            module_id,
            offset,
            flags,
        )?;
        Ok(())
    }

    fn on_read_substate(
        &mut self,
        current_frame: &CallFrame,
        heap: &mut Heap,
        track: &mut Track,
        lock_handle: LockHandle,
        size: usize,
    ) -> Result<(), ModuleError> {
        if self.kernel_trace {
            KernelTraceModule.on_read_substate(current_frame, heap, track, lock_handle, size)?;
        }
        self.execution_costing
            .on_read_substate(current_frame, heap, track, lock_handle, size)?;
        self.royalty_costing
            .on_read_substate(current_frame, heap, track, lock_handle, size)?;
        self.execution_trace
            .on_read_substate(current_frame, heap, track, lock_handle, size)?;
        Ok(())
    }

    fn on_write_substate(
        &mut self,
        current_frame: &CallFrame,
        heap: &mut Heap,
        track: &mut Track,
        lock_handle: LockHandle,
        size: usize,
    ) -> Result<(), ModuleError> {
        if self.kernel_trace {
            KernelTraceModule.on_write_substate(current_frame, heap, track, lock_handle, size)?;
        }
        self.execution_costing
            .on_write_substate(current_frame, heap, track, lock_handle, size)?;
        self.royalty_costing
            .on_write_substate(current_frame, heap, track, lock_handle, size)?;
        self.execution_trace
            .on_write_substate(current_frame, heap, track, lock_handle, size)?;
        Ok(())
    }

    fn on_drop_lock(
        &mut self,
        current_frame: &CallFrame,
        heap: &mut Heap,
        track: &mut Track,
        lock_handle: LockHandle,
    ) -> Result<(), ModuleError> {
        if self.kernel_trace {
            KernelTraceModule.on_drop_lock(current_frame, heap, track, lock_handle)?;
        }
        self.execution_costing
            .on_drop_lock(current_frame, heap, track, lock_handle)?;
        self.royalty_costing
            .on_drop_lock(current_frame, heap, track, lock_handle)?;
        self.execution_trace
            .on_drop_lock(current_frame, heap, track, lock_handle)?;
        Ok(())
    }

    fn on_wasm_instantiation(
        &mut self,
        current_frame: &CallFrame,
        heap: &mut Heap,
        track: &mut Track,
        code: &[u8],
    ) -> Result<(), ModuleError> {
        if self.kernel_trace {
            KernelTraceModule.on_wasm_instantiation(current_frame, heap, track, code)?;
        }
        self.execution_costing
            .on_wasm_instantiation(current_frame, heap, track, code)?;
        self.royalty_costing
            .on_wasm_instantiation(current_frame, heap, track, code)?;
        self.execution_trace
            .on_wasm_instantiation(current_frame, heap, track, code)?;

        Ok(())
    }

    fn on_wasm_costing(
        &mut self,
        current_frame: &CallFrame,
        heap: &mut Heap,
        track: &mut Track,
        units: u32,
    ) -> Result<(), ModuleError> {
        if self.kernel_trace {
            KernelTraceModule.on_wasm_costing(current_frame, heap, track, units)?;
        }
        self.execution_costing
            .on_wasm_costing(current_frame, heap, track, units)?;
        self.royalty_costing
            .on_wasm_costing(current_frame, heap, track, units)?;
        self.execution_trace
            .on_wasm_costing(current_frame, heap, track, units)?;

        Ok(())
    }
}

impl KernelModuleMixer {
    // Modules are initialized in the following order
    //  * CostingModule
    //  * TransactionRuntimeModule
    //  * LoggerModule
    //  * AuthModule
    //  * NodeMoveModule
    // and applied in the reverse order.

    pub fn initialize<Y: KernelNodeApi + KernelSubstateApi>(
        api: &mut Y,
        tx_hash: Hash,
        auth_zone_params: AuthZoneParams,
        fee_reserve: SystemLoanFeeReserve,
        fee_table: FeeTable,
    ) -> Result<(), RuntimeError> {
        // Module initialization order decodes when certain features are enabled or disabled.
        // See also `destroy()` implementation when reordering items.

        CostingModule::initialize(api, fee_reserve, fee_table)?;
        TransactionRuntimeModule::initialize(api, tx_hash)?;
        LoggerModule::initialize(api)?;
        AuthModule::initialize(api, auth_zone_params)?;

        Ok(())
    }

    pub fn teardown<Y: KernelNodeApi + KernelSubstateApi>(
        api: &mut Y,
    ) -> Result<FeeReserveSubstate, RuntimeError> {
        AuthModule::teardown(api)
            .and_then(|_| LoggerModule::teardown(api))
            .and_then(|_| TransactionRuntimeModule::teardown(api))
            .and_then(|_| CostingModule::teardown(api))
    }

    pub fn on_before_frame_start<Y>(actor: &ResolvedActor, api: &mut Y) -> Result<(), RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi,
    {
        AuthModule::on_before_frame_start(actor, api)
    }

    pub fn on_call_frame_enter<
        Y: KernelNodeApi + KernelSubstateApi + KernelActorApi<RuntimeError>,
    >(
        update: &mut CallFrameUpdate,
        actor: &ResolvedActor,
        api: &mut Y,
    ) -> Result<(), RuntimeError> {
        NodeMoveModule::on_call_frame_enter(update, &actor.identifier, api)
            .and_then(|_| AuthModule::on_call_frame_enter(update, actor, api))
            .and_then(|_| LoggerModule::on_call_frame_enter(update, actor, api))
            .and_then(|_| TransactionRuntimeModule::on_call_frame_enter(update, actor, api))
            .and_then(|_| CostingModule::on_call_frame_enter(update, actor, api))
    }

    pub fn on_call_frame_exit<Y>(update: &CallFrameUpdate, api: &mut Y) -> Result<(), RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + KernelActorApi<RuntimeError>,
    {
        NodeMoveModule::on_call_frame_exit(update, api)
            .and_then(|_| AuthModule::on_call_frame_exit(api))
    }
}
