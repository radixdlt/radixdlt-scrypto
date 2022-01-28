use wasmi::*;

use crate::engine::*;
use crate::model::*;

/// Instantiates a WASM module.
pub fn instantiate_module(module: &Module) -> Result<(ModuleRef, MemoryRef), WasmValidationError> {
    // Instantiate
    let instance = ModuleInstance::new(
        module,
        &ImportsBuilder::new().with_resolver("env", &EnvModuleResolver),
    )
    .map_err(WasmValidationError::InvalidModule)?
    .assert_no_start();

    // Find memory export
    if let Some(ExternVal::Memory(memory)) = instance.export_by_name("memory") {
        Ok((instance, memory))
    } else {
        Err(WasmValidationError::NoValidMemoryExport)
    }
}
