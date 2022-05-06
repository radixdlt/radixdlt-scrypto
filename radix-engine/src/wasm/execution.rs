use super::env_resolver::*;
use super::errors::*;
use wasmi::{ExternVal, ImportsBuilder, MemoryRef, Module, ModuleInstance, ModuleRef};

pub fn instantiate_module(module: &Module) -> Result<(ModuleRef, MemoryRef), WasmValidationError> {
    // Instantiate
    let instance = ModuleInstance::new(
        module,
        &ImportsBuilder::new().with_resolver("env", &EnvModuleResolver),
    )
    .map_err(|_| WasmValidationError::InvalidModule)?
    .assert_no_start();

    // Find memory export
    if let Some(ExternVal::Memory(memory)) = instance.export_by_name("memory") {
        Ok((instance, memory))
    } else {
        Err(WasmValidationError::NoValidMemoryExport)
    }
}
