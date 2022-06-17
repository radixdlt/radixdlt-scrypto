use sbor::rust::collections::HashMap;
use sbor::rust::string::String;
use scrypto::abi::BlueprintAbi;

use crate::wasm::*;

pub struct WasmValidator {
    pub max_initial_memory_size_pages: u32,
    pub max_initial_table_size: u32,
    pub max_number_of_br_table_targets: u32,
}

impl Default for WasmValidator {
    fn default() -> Self {
        Self {
            max_initial_memory_size_pages: DEFAULT_MAX_INITIAL_MEMORY_SIZE_PAGES,
            max_initial_table_size: DEFAULT_MAX_INITIAL_TABLE_SIZE,
            max_number_of_br_table_targets: DEFAULT_MAX_NUMBER_OF_BR_TABLE_TARGETS,
        }
    }
}

impl WasmValidator {
    pub fn validate(
        &self,
        code: &[u8],
        blueprints: &HashMap<String, BlueprintAbi>,
    ) -> Result<(), PrepareError> {
        // Not all "valid" wasm modules are instrumentable, with the instrumentation library
        // we are using. To deal with this, we attempt to instrument the input module with
        // some mocked parameters and reject it if fails to do so.
        let mocked_wasm_metering_params = WasmMeteringParams::new(1, 1, 100, 500);

        WasmModule::init(code)?
            .enforce_no_floating_point()?
            .enforce_no_start_function()?
            .enforce_import_limit()?
            .enforce_memory_limit(self.max_initial_memory_size_pages)?
            .enforce_table_limit(self.max_initial_table_size)?
            .enforce_br_table_limit(self.max_number_of_br_table_targets)?
            .enforce_function_limit()?
            .enforce_global_limit()?
            .enforce_local_limit()?
            .enforce_export_constraints(blueprints)?
            .inject_instruction_metering(
                mocked_wasm_metering_params.instruction_cost(),
                mocked_wasm_metering_params.grow_memory_cost(),
            )?
            .inject_stack_metering(mocked_wasm_metering_params.max_stack_size())?
            .to_bytes()?;

        Ok(())
    }
}
