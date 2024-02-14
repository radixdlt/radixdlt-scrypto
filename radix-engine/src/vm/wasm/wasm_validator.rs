use crate::internal_prelude::*;
use crate::vm::wasm::*;
use crate::vm::ScryptoVmVersion;
use radix_engine_interface::blueprints::package::BlueprintDefinitionInit;

pub struct ScryptoV1WasmValidator {
    pub max_memory_size_in_pages: u32,
    pub max_initial_table_size: u32,
    pub max_number_of_br_table_targets: u32,
    pub max_number_of_functions: u32,
    pub max_number_of_function_params: u32,
    pub max_number_of_function_locals: u32,
    pub max_number_of_globals: u32,
    pub instrumenter_config: WasmValidatorConfigV1,
    pub version: ScryptoVmVersion,
}

impl ScryptoV1WasmValidator {
    pub fn new(version: ScryptoVmVersion) -> Self {
        if version > ScryptoVmVersion::latest() {
            panic!("Invalid minor version: {:?}", version);
        }

        Self {
            max_memory_size_in_pages: MAX_MEMORY_SIZE_IN_PAGES,
            max_initial_table_size: MAX_INITIAL_TABLE_SIZE,
            max_number_of_br_table_targets: MAX_NUMBER_OF_BR_TABLE_TARGETS,
            max_number_of_functions: MAX_NUMBER_OF_FUNCTIONS,
            max_number_of_function_params: MAX_NUMBER_OF_FUNCTION_PARAMS,
            max_number_of_function_locals: MAX_NUMBER_OF_FUNCTION_LOCALS,
            max_number_of_globals: MAX_NUMBER_OF_GLOBALS,
            instrumenter_config: WasmValidatorConfigV1::new(),
            version,
        }
    }
}

impl ScryptoV1WasmValidator {
    pub fn validate<'a, I: Iterator<Item = &'a BlueprintDefinitionInit>>(
        &self,
        code: &[u8],
        blueprints: I,
    ) -> Result<(Vec<u8>, Vec<String>), PrepareError> {
        WasmModule::init(code)?
            .enforce_no_start_function()?
            .enforce_import_constraints(self.version)?
            .enforce_export_names()?
            .enforce_memory_limit_and_inject_max(self.max_memory_size_in_pages)?
            .enforce_table_limit(self.max_initial_table_size)?
            .enforce_br_table_limit(self.max_number_of_br_table_targets)?
            .enforce_function_limit(
                self.max_number_of_functions,
                self.max_number_of_function_params,
                self.max_number_of_function_locals,
            )?
            .enforce_global_limit(self.max_number_of_globals)?
            .enforce_export_constraints(blueprints)?
            .inject_instruction_metering(&self.instrumenter_config)?
            .inject_stack_metering(self.instrumenter_config.max_stack_size())?
            .ensure_instantiatable()?
            .ensure_compilable()?
            .to_bytes()
    }
}

#[cfg(test)]
mod tests {
    use radix_engine_interface::blueprints::package::PackageDefinition;
    use wabt::{wasm2wat, wat2wasm};

    use super::ScryptoV1WasmValidator;
    use super::ScryptoVmVersion;

    #[test]
    fn test_validate() {
        let code = wat2wasm(
            r#"
        (module

            ;; Simple function that always returns `()`
            (func $Test_f (param $0 i64) (result i64)
              ;; Grow memory
              (drop
                (memory.grow (i32.const 1000000))
              )

              ;; Encode () in SBOR at address 0x0
              (i32.const 0)
              (i32.const 92)  ;; prefix
              (i32.store8)
              (i32.const 1)
              (i32.const 33)  ;; tuple value kind
              (i32.store8)
              (i32.const 2)
              (i32.const 0)  ;; tuple length
              (i32.store8)

              ;; Return slice (ptr = 0, len = 3)
              (i64.const 3)
            )

            (memory $0 1)
            (export "memory" (memory $0))
            (export "Test_f" (func $Test_f))
        )"#,
        )
        .unwrap();

        let instrumented_code = wasm2wat(
            ScryptoV1WasmValidator::new(ScryptoVmVersion::latest())
                .validate(
                    &code,
                    PackageDefinition::new_single_function_test_definition("Test", "f")
                        .blueprints
                        .values(),
                )
                .unwrap()
                .0,
        )
        .unwrap();

        assert_eq!(
            instrumented_code,
            r#"(module
  (type (;0;) (func (param i64) (result i64)))
  (type (;1;) (func (param i64)))
  (import "env" "gas" (func (;0;) (type 1)))
  (func (;1;) (type 0) (param i64) (result i64)
    i64.const 14788284
    call 0
    i32.const 1000000
    memory.grow
    drop
    i32.const 0
    i32.const 92
    i32.store8
    i32.const 1
    i32.const 33
    i32.store8
    i32.const 2
    i32.const 0
    i32.store8
    i64.const 3)
  (func (;2;) (type 0) (param i64) (result i64)
    local.get 0
    global.get 0
    i32.const 4
    i32.add
    global.set 0
    global.get 0
    i32.const 1024
    i32.gt_u
    if  ;; label = @1
      unreachable
    end
    call 1
    global.get 0
    i32.const 4
    i32.sub
    global.set 0)
  (memory (;0;) 1 64)
  (global (;0;) (mut i32) (i32.const 0))
  (export "memory" (memory 0))
  (export "Test_f" (func 2)))
"#
        )
    }
}
