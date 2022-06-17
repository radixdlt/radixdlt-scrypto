use crate::wasm::{PrepareError, WasmMeteringParams, WasmModule};

pub struct WasmValidator {}

impl WasmValidator {
    pub fn validate(code: &[u8]) -> Result<(), PrepareError> {
        // Not all "valid" wasm modules are instrumentable, with the instrumentation library
        // we are using. To deal with this, we attempt to instrument the input module with
        // some mocked parameters and reject it if fails to do so.
        let mocked_wasm_metering_params = WasmMeteringParams::new(1, 1, 100, 500);

        WasmModule::init(code)?
            .enforce_no_floating_point()?
            .enforce_no_start_function()?
            .enforce_import_limit()?
            .enforce_memory_limit()?
            .enforce_table_limit()?
            .enforce_br_table_limit()?
            .enforce_function_limit()?
            .enforce_global_limit()?
            .enforce_local_limit()?
            .inject_instruction_metering(
                mocked_wasm_metering_params.instruction_cost(),
                mocked_wasm_metering_params.grow_memory_cost(),
            )?
            .inject_stack_metering(mocked_wasm_metering_params.max_stack_size())?
            .to_bytes()?;

        Ok(())
    }
}
