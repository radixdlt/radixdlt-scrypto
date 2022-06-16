use crate::wasm::{PrepareError, WasmMeteringParams, WasmModule};
use sbor::rust::collections::HashMap;
use sbor::rust::string::String;
use scrypto::abi::BlueprintAbi;

pub struct WasmValidator {}

impl WasmValidator {
    pub fn validate(
        code: &[u8],
        blueprints: &HashMap<String, BlueprintAbi>,
    ) -> Result<(), PrepareError> {
        // Not all "valid" wasm modules are instrumentable, with the instrumentation library
        // we are using. To deal with this, we attempt to instrument the input module with
        // some mocked parameters and reject it if fails to do so.
        let mocked_wasm_metering_params = WasmMeteringParams::new(1, 1, 100, 500);

        WasmModule::init(code)?
            .reject_floating_point()?
            .reject_start_function()?
            .check_imports()?
            .check_exports(blueprints)?
            .check_memory()?
            .enforce_initial_memory_limit()?
            .enforce_functions_limit()?
            .enforce_locals_limit()?
            .inject_instruction_metering(
                mocked_wasm_metering_params.instruction_cost(),
                mocked_wasm_metering_params.grow_memory_cost(),
            )?
            .inject_stack_metering(mocked_wasm_metering_params.max_stack_size())?
            .to_bytes()?;

        Ok(())
    }
}
