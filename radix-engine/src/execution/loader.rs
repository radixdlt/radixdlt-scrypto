use wasmi::*;

use crate::execution::*;

pub fn load_module(code: &[u8]) -> Result<(ModuleRef, MemoryRef), RuntimeError> {
    // Parse
    let parsed = Module::from_buffer(code).map_err(|e| RuntimeError::InvalidModule(e))?;

    // Instantiate
    let not_started = ModuleInstance::new(
        &parsed,
        &ImportsBuilder::new().with_resolver("env", &EnvModuleResolver),
    )
    .map_err(|e| RuntimeError::UnableToInstantiate(e))?;

    // Check start function
    if not_started.has_start() {
        return Err(RuntimeError::HasStartFunction);
    }
    let module_ref = not_started.assert_no_start();

    // Check memory export
    if let Some(ExternVal::Memory(memory)) = module_ref.export_by_name("memory") {
        Ok((module_ref, memory.to_owned()))
    } else {
        Err(RuntimeError::NoValidMemoryExport)
    }
}
