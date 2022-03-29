use scrypto::buffer::scrypto_decode;
use scrypto::rust::vec;
use scrypto::rust::vec::Vec;
use scrypto::rust::string::String;
use wasmi::*;

use crate::engine::*;
use crate::errors::*;

/// Parses a WASM module.
pub fn parse_module(code: &[u8]) -> Result<Module, WasmValidationError> {
    Module::from_buffer(code).map_err(|_| WasmValidationError::InvalidModule)
}

/// Validates a WASM module.
pub fn initialize_package(code: &[u8]) -> Result<Vec<String>, WasmValidationError> {
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
    .map_err(|_| WasmValidationError::InvalidModule)?;

    // Check start function
    if instance.has_start() {
        return Err(WasmValidationError::StartFunctionNotAllowed);
    }
    let module = instance.assert_no_start();

    // Check memory export
    let memory = match module.export_by_name("memory") {
        Some(ExternVal::Memory(mem)) => mem,
        _ => return Err(WasmValidationError::NoValidMemoryExport)
    };

    let rtn = module.invoke_export("package_init", &[], &mut NopExternals)
        .map_err(|e| WasmValidationError::NoPackageInitExport(e.into()))?
        .ok_or(WasmValidationError::InvalidPackageInit)?;

    match rtn {
        RuntimeValue::I32(ptr) => {
            let len: u32 = memory
                .get_value(ptr as u32)
                .map_err(|_| WasmValidationError::InvalidPackageInit)?;

            // SECURITY: meter before allocating memory
            let mut data = vec![0u8; len as usize];
            memory
                .get_into((ptr + 4) as u32, &mut data)
                .map_err(|_| WasmValidationError::InvalidPackageInit)?;

            scrypto_decode(&data).map_err(|_| WasmValidationError::InvalidPackageInit)
        }
        _ => Err(WasmValidationError::InvalidPackageInit)
    }
}
