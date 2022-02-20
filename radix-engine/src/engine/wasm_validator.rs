use wasmi::*;

use crate::engine::*;
use crate::errors::*;

/// Parses a WASM module.
pub fn parse_module(code: &[u8]) -> Result<Module, WasmValidationError> {
    Module::from_buffer(code).map_err(WasmValidationError::InvalidModule)
}

/// Validates a WASM module.
pub fn validate_module(code: &[u8]) -> Result<(), WasmValidationError> {
    // Parse
    let parsed = parse_module(code)?;

    // check floating point
    parsed
        .deny_floating_point()
        .map_err(|_| WasmValidationError::FloatingPointNotAllowed)?;

    // Instantiate
    let instance = ModuleInstance::new(
        &parsed,
        &ImportsBuilder::new().with_resolver("env", &EnvModuleResolver),
    )
    .map_err(WasmValidationError::InvalidModule)?;

    // Check start function
    if instance.has_start() {
        return Err(WasmValidationError::StartFunctionNotAllowed);
    }
    let module = instance.assert_no_start();

    // Check memory export
    if let Some(ExternVal::Memory(_)) = module.export_by_name("memory") {
        Ok(())
    } else {
        Err(WasmValidationError::NoValidMemoryExport)
    }
}
