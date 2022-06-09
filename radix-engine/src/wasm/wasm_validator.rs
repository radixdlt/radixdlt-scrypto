use crate::wasm::{PrepareError, WasmFeeTable, WasmModule};

pub struct WasmValidator {}

impl WasmValidator {
    pub fn validate(code: &[u8]) -> Result<(), PrepareError> {
        // Not all "valid" wasm modules are instrumentable, with the instrumentation library
        // we are using. To deal with this, we attempt to instrument the input module with
        // some mocked parameters and reject it if fails to do so.
        let mocked_wasm_fee_table = WasmFeeTable::new(1, 100);
        let mocked_wasm_max_stack_size = 100;

        WasmModule::init(code)?
            .reject_floating_point()?
            .reject_start_function()?
            .check_imports()?
            .check_memory()?
            .enforce_initial_memory_limit()?
            .enforce_functions_limit()?
            .enforce_locals_limit()?
            .inject_instruction_metering(&mocked_wasm_fee_table)?
            .inject_stack_metering(mocked_wasm_max_stack_size)?
            .to_bytes()?;

        Ok(())
    }
}
