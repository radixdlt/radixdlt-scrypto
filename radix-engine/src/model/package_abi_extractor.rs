use sbor::rust::string::String;
use sbor::rust::string::ToString;
use sbor::rust::vec::Vec;
use scrypto::prelude::Package;

use crate::wasm::*;

pub fn extract_abi(code: &[u8]) -> Result<Vec<String>, WasmValidationError> {
    let mut wasm_engine = WasmiEngine::new();
    // TODO: A bit of a code smell to have validation here, remove at some point.
    wasm_engine.validate(code)?;
    let module = wasm_engine.instantiate(code);
    let exports: Vec<String> = module
        .function_exports()
        .into_iter()
        .filter(|e| e.ends_with("_abi") && e.len() > 4)
        .map(|s| s.split_at(s.len() - 4).0.to_string())
        .collect();
    Ok(exports)
}

pub fn new_extracted_package(code: Vec<u8>) -> Result<Package, WasmValidationError> {
    let blueprints = extract_abi(&code)?;
    let package = Package::new(code, blueprints);
    Ok(package)
}