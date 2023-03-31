use radix_engine_interface::schema::PackageSchema;

use crate::types::*;
use crate::wasm::*;

pub struct WasmValidator {
    pub max_initial_memory_size_pages: u32,
    pub max_initial_table_size: u32,
    pub max_number_of_br_table_targets: u32,
    pub max_number_of_functions: u32,
    pub max_number_of_globals: u32,
}

impl Default for WasmValidator {
    fn default() -> Self {
        Self {
            max_initial_memory_size_pages: DEFAULT_MAX_INITIAL_MEMORY_SIZE_PAGES,
            max_initial_table_size: DEFAULT_MAX_INITIAL_TABLE_SIZE,
            max_number_of_br_table_targets: DEFAULT_MAX_NUMBER_OF_BR_TABLE_TARGETS,
            max_number_of_functions: DEFAULT_MAX_NUMBER_OF_FUNCTIONS,
            max_number_of_globals: DEFAULT_MAX_NUMBER_OF_GLOBALS,
        }
    }
}

impl WasmValidator {
    pub fn validate(&self, code: &[u8], schema: &PackageSchema) -> Result<(), PrepareError> {
        // Not all "valid" wasm modules are instrumentable, with the instrumentation library
        // we are using. To deal with this, we attempt to instrument the input module with
        // some mocked parameters and reject it if fails to do so.
        let parameters = WasmMeteringConfig::V0.parameters();

        WasmModule::init(code)?
            .enforce_no_floating_point()?
            .enforce_no_start_function()?
            .enforce_import_limit()?
            .enforce_memory_limit(self.max_initial_memory_size_pages)?
            .enforce_table_limit(self.max_initial_table_size)?
            .enforce_br_table_limit(self.max_number_of_br_table_targets)?
            .enforce_function_limit(self.max_number_of_functions)?
            .enforce_global_limit(self.max_number_of_globals)?
            .enforce_export_constraints(schema)?
            .inject_instruction_metering(parameters.instruction_cost_rules())?
            .inject_stack_metering(parameters.max_stack_size())?
            .ensure_instantiatable()?
            .ensure_compilable()?
            .to_bytes()?;

        Ok(())
    }
}
