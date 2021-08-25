use scrypto::rust::borrow::ToOwned;
use wasmi::*;

use crate::execution::*;

/// Validate and instantiate a WASM module.
pub fn load_module(code: &[u8]) -> Result<(ModuleRef, MemoryRef), RuntimeError> {
    // Parse
    let parsed = Module::from_buffer(code).map_err(|e| RuntimeError::InvalidModule(e))?;
    parsed
        .deny_floating_point()
        .map_err(|_| RuntimeError::FloatingPointNotAllowed)?;

    // Instantiate
    let instance = ModuleInstance::new(
        &parsed,
        &ImportsBuilder::new().with_resolver("env", &EnvModuleResolver),
    )
    .map_err(|e| RuntimeError::UnableToInstantiate(e))?;

    // Check start function
    if instance.has_start() {
        return Err(RuntimeError::StartFunctionNotAllowed);
    }
    let module_ref = instance.assert_no_start();

    // Check memory export
    if let Some(ExternVal::Memory(memory)) = module_ref.export_by_name("memory") {
        Ok((module_ref, memory.to_owned()))
    } else {
        Err(RuntimeError::NoValidMemoryExport)
    }
}
