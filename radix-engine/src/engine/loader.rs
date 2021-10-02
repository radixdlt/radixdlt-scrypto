use wasmi::*;

use crate::engine::*;

/// Parse a WASM module.
pub fn parse_module(code: &[u8]) -> Result<Module, RuntimeError> {
    Module::from_buffer(code).map_err(RuntimeError::InvalidModule)
}

/// Instantiate a WASM module.
pub fn instantiate_module(module: &Module) -> Result<(ModuleRef, MemoryRef), RuntimeError> {
    // Instantiate
    let instance = ModuleInstance::new(
        module,
        &ImportsBuilder::new().with_resolver("env", &EnvModuleResolver),
    )
    .map_err(RuntimeError::InvalidModule)?
    .assert_no_start();

    // Find memory export
    if let Some(ExternVal::Memory(memory)) = instance.export_by_name("memory") {
        Ok((instance, memory))
    } else {
        Err(RuntimeError::NoValidMemoryExport)
    }
}

/// Validate a WASM module.
pub fn validate_module(code: &[u8]) -> Result<(), RuntimeError> {
    // Parse
    let parsed = parse_module(code)?;

    // check floating point
    parsed
        .deny_floating_point()
        .map_err(|_| RuntimeError::FloatingPointNotAllowed)?;

    // Instantiate
    let instance = ModuleInstance::new(
        &parsed,
        &ImportsBuilder::new().with_resolver("env", &EnvModuleResolver),
    )
    .map_err(RuntimeError::InvalidModule)?;

    // Check start function
    if instance.has_start() {
        return Err(RuntimeError::StartFunctionNotAllowed);
    }
    let module = instance.assert_no_start();

    // Check memory export
    if let Some(ExternVal::Memory(_)) = module.export_by_name("memory") {
        Ok(())
    } else {
        Err(RuntimeError::NoValidMemoryExport)
    }
}
