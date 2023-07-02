use crate::errors::InvokeError;
use crate::system::system_modules::costing::*;
use crate::types::*;
use crate::vm::wasm::*;
use sbor::rust::vec::Vec;

/// A `Nop` runtime accepts any external function calls by doing nothing and returning void.
pub struct NoOpWasmRuntime<'a> {
    pub fee_reserve: SystemLoanFeeReserve,
    pub wasm_execution_units_consumed: &'a mut u64,
    pub fee_table: FeeTable,
}

impl<'a> NoOpWasmRuntime<'a> {
    pub fn new(
        fee_reserve: SystemLoanFeeReserve,
        wasm_execution_units_consumed: &'a mut u64,
    ) -> Self {
        Self {
            fee_reserve,
            wasm_execution_units_consumed,
            fee_table: FeeTable::new(),
        }
    }
}

#[allow(unused_variables)]
impl<'a> WasmRuntime for NoOpWasmRuntime<'a> {
    fn allocate_buffer(
        &mut self,
        buffer: Vec<u8>,
    ) -> Result<Buffer, InvokeError<WasmRuntimeError>> {
        Err(InvokeError::SelfError(WasmRuntimeError::NotImplemented))
    }

    fn consume_buffer(
        &mut self,
        buffer_id: BufferId,
    ) -> Result<Vec<u8>, InvokeError<WasmRuntimeError>> {
        Err(InvokeError::SelfError(WasmRuntimeError::NotImplemented))
    }

    fn actor_call_module_method(
        &mut self,
        object_handle: u32,
        module_id: u32,
        ident: Vec<u8>,
        args: Vec<u8>,
    ) -> Result<Buffer, InvokeError<WasmRuntimeError>> {
        Err(InvokeError::SelfError(WasmRuntimeError::NotImplemented))
    }

    fn call_method(
        &mut self,
        receiver: Vec<u8>,
        direct_access: u32,
        module_id: u32,
        ident: Vec<u8>,
        args: Vec<u8>,
    ) -> Result<Buffer, InvokeError<WasmRuntimeError>> {
        Err(InvokeError::SelfError(WasmRuntimeError::NotImplemented))
    }

    fn call_function(
        &mut self,
        package_address: Vec<u8>,
        blueprint_ident: Vec<u8>,
        ident: Vec<u8>,
        args: Vec<u8>,
    ) -> Result<Buffer, InvokeError<WasmRuntimeError>> {
        Err(InvokeError::SelfError(WasmRuntimeError::NotImplemented))
    }

    fn new_object(
        &mut self,
        blueprint_ident: Vec<u8>,
        object_states: Vec<u8>,
    ) -> Result<Buffer, InvokeError<WasmRuntimeError>> {
        Err(InvokeError::SelfError(WasmRuntimeError::NotImplemented))
    }

    fn allocate_global_address(
        &mut self,
        blueprint_id: Vec<u8>,
    ) -> Result<Buffer, InvokeError<WasmRuntimeError>> {
        Err(InvokeError::SelfError(WasmRuntimeError::NotImplemented))
    }

    fn globalize_object(
        &mut self,
        modules: Vec<u8>,
        address: Vec<u8>,
    ) -> Result<Buffer, InvokeError<WasmRuntimeError>> {
        Err(InvokeError::SelfError(WasmRuntimeError::NotImplemented))
    }

    fn drop_object(&mut self, node_id: Vec<u8>) -> Result<(), InvokeError<WasmRuntimeError>> {
        Err(InvokeError::SelfError(WasmRuntimeError::NotImplemented))
    }

    fn key_value_store_new(
        &mut self,
        schema: Vec<u8>,
    ) -> Result<Buffer, InvokeError<WasmRuntimeError>> {
        Err(InvokeError::SelfError(WasmRuntimeError::NotImplemented))
    }

    fn key_value_store_open_entry(
        &mut self,
        node_id: Vec<u8>,
        offset: Vec<u8>,
        flags: u32,
    ) -> Result<LockHandle, InvokeError<WasmRuntimeError>> {
        Err(InvokeError::SelfError(WasmRuntimeError::NotImplemented))
    }

    fn key_value_entry_get(
        &mut self,
        handle: u32,
    ) -> Result<Buffer, InvokeError<WasmRuntimeError>> {
        Err(InvokeError::SelfError(WasmRuntimeError::NotImplemented))
    }

    fn key_value_entry_set(
        &mut self,
        handle: u32,
        data: Vec<u8>,
    ) -> Result<(), InvokeError<WasmRuntimeError>> {
        Err(InvokeError::SelfError(WasmRuntimeError::NotImplemented))
    }

    fn key_value_entry_release(
        &mut self,
        handle: u32,
    ) -> Result<(), InvokeError<WasmRuntimeError>> {
        Err(InvokeError::SelfError(WasmRuntimeError::NotImplemented))
    }

    fn key_value_store_remove_entry(
        &mut self,
        node_id: Vec<u8>,
        key: Vec<u8>,
    ) -> Result<Buffer, InvokeError<WasmRuntimeError>> {
        Err(InvokeError::SelfError(WasmRuntimeError::NotImplemented))
    }

    fn actor_open_field(
        &mut self,
        object_handle: u32,
        field: u8,
        flags: u32,
    ) -> Result<u32, InvokeError<WasmRuntimeError>> {
        Err(InvokeError::SelfError(WasmRuntimeError::NotImplemented))
    }

    fn field_lock_read(&mut self, handle: u32) -> Result<Buffer, InvokeError<WasmRuntimeError>> {
        Err(InvokeError::SelfError(WasmRuntimeError::NotImplemented))
    }

    fn field_lock_write(
        &mut self,
        handle: u32,
        data: Vec<u8>,
    ) -> Result<(), InvokeError<WasmRuntimeError>> {
        Err(InvokeError::SelfError(WasmRuntimeError::NotImplemented))
    }

    fn field_lock_release(&mut self, handle: u32) -> Result<(), InvokeError<WasmRuntimeError>> {
        Err(InvokeError::SelfError(WasmRuntimeError::NotImplemented))
    }

    fn get_node_id(&mut self) -> Result<Buffer, InvokeError<WasmRuntimeError>> {
        Err(InvokeError::SelfError(WasmRuntimeError::NotImplemented))
    }

    fn get_global_address(&mut self) -> Result<Buffer, InvokeError<WasmRuntimeError>> {
        Err(InvokeError::SelfError(WasmRuntimeError::NotImplemented))
    }

    fn get_blueprint(&mut self) -> Result<Buffer, InvokeError<WasmRuntimeError>> {
        Err(InvokeError::SelfError(WasmRuntimeError::NotImplemented))
    }

    fn get_auth_zone(&mut self) -> Result<Buffer, InvokeError<WasmRuntimeError>> {
        Err(InvokeError::SelfError(WasmRuntimeError::NotImplemented))
    }

    fn consume_wasm_execution_units(
        &mut self,
        n: u32,
    ) -> Result<(), InvokeError<WasmRuntimeError>> {
        self.wasm_execution_units_consumed.add_assign(n as u64);
        self.fee_reserve
            .consume_execution(
                self.fee_table
                    .run_wasm_code_cost(&PACKAGE_PACKAGE, "none", n),
            )
            .map_err(|e| InvokeError::SelfError(WasmRuntimeError::FeeReserveError(e)))
    }

    fn get_object_info(
        &mut self,
        component_id: Vec<u8>,
    ) -> Result<Buffer, InvokeError<WasmRuntimeError>> {
        Err(InvokeError::SelfError(WasmRuntimeError::NotImplemented))
    }

    fn emit_event(
        &mut self,
        event_name: Vec<u8>,
        event: Vec<u8>,
    ) -> Result<(), InvokeError<WasmRuntimeError>> {
        Err(InvokeError::SelfError(WasmRuntimeError::NotImplemented))
    }

    fn emit_log(
        &mut self,
        level: Vec<u8>,
        message: Vec<u8>,
    ) -> Result<(), InvokeError<WasmRuntimeError>> {
        Err(InvokeError::SelfError(WasmRuntimeError::NotImplemented))
    }

    fn panic(&mut self, message: Vec<u8>) -> Result<(), InvokeError<WasmRuntimeError>> {
        Err(InvokeError::SelfError(WasmRuntimeError::NotImplemented))
    }

    fn get_transaction_hash(&mut self) -> Result<Buffer, InvokeError<WasmRuntimeError>> {
        Err(InvokeError::SelfError(WasmRuntimeError::NotImplemented))
    }

    fn generate_ruid(&mut self) -> Result<Buffer, InvokeError<WasmRuntimeError>> {
        Err(InvokeError::SelfError(WasmRuntimeError::NotImplemented))
    }

    fn assert_access_rule(&mut self, rule: Vec<u8>) -> Result<(), InvokeError<WasmRuntimeError>> {
        Err(InvokeError::SelfError(WasmRuntimeError::NotImplemented))
    }

    fn cost_unit_limit(&mut self) -> Result<u32, InvokeError<WasmRuntimeError>> {
        Err(InvokeError::SelfError(WasmRuntimeError::NotImplemented))
    }

    fn cost_unit_price(&mut self) -> Result<Buffer, InvokeError<WasmRuntimeError>> {
        Err(InvokeError::SelfError(WasmRuntimeError::NotImplemented))
    }

    fn tip_percentage(&mut self) -> Result<u32, InvokeError<WasmRuntimeError>> {
        Err(InvokeError::SelfError(WasmRuntimeError::NotImplemented))
    }

    fn fee_balance(&mut self) -> Result<Buffer, InvokeError<WasmRuntimeError>> {
        Err(InvokeError::SelfError(WasmRuntimeError::NotImplemented))
    }
}
