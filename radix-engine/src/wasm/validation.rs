use super::env_resolver::*;
use super::errors::*;
use wasmi::{ExternVal, ImportsBuilder, MemoryRef, Module, ModuleInstance, ModuleRef};

pub fn parse_module(code: &[u8]) -> Result<Module, WasmValidationError> {
    Module::from_buffer(code).map_err(|_| WasmValidationError::InvalidModule)
}

pub fn validate_module(module: &Module) -> Result<(ModuleRef, MemoryRef), WasmValidationError> {
    // check floating point
    module
        .deny_floating_point()
        .map_err(|_| WasmValidationError::FloatingPointNotAllowed)?;

    // Instantiate
    let instance = ModuleInstance::new(
        &module,
        &ImportsBuilder::new().with_resolver("env", &EnvModuleResolver),
    )
    .map_err(|_| WasmValidationError::InvalidModule)?;

    // Check start function
    if instance.has_start() {
        return Err(WasmValidationError::StartFunctionNotAllowed);
    }
    let module_ref = instance.assert_no_start();

    // Check memory export
    let memory_ref = match module_ref.export_by_name("memory") {
        Some(ExternVal::Memory(mem)) => mem,
        _ => return Err(WasmValidationError::NoValidMemoryExport),
    };

    Ok((module_ref, memory_ref))
}
