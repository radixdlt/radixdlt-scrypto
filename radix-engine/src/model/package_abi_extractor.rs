use sbor::rust::string::String;
use sbor::rust::vec::Vec;
use sbor::rust::collections::HashMap;
use sbor::Type;
use scrypto::abi::{Function, Method};
use scrypto::buffer::scrypto_decode;
use scrypto::prelude::Package;
use scrypto::values::ScryptoValue;

use crate::wasm::*;

fn extract_abi(code: &[u8]) -> Result<HashMap<String, (Type, Vec<Function>, Vec<Method>)>, WasmValidationError> {
    let mut wasm_engine = WasmiEngine::new();
    // TODO: A bit of a code smell to have validation here, remove at some point.
    wasm_engine.validate(code)?;
    let module = wasm_engine.instantiate(code);
    let exports: Vec<String> = module
        .function_exports()
        .into_iter()
        .filter(|e| e.ends_with("_abi") && e.len() > 4)
        .collect();

    let mut blueprints = HashMap::new();
    for method_name in exports {
        let rtn = module
            .invoke_export(
                &method_name,
                &ScryptoValue::unit(),
                &mut NopScryptoRuntime {},
            )
            .map_err(|_| WasmValidationError::UnableToExportBlueprintAbi)?;

        let abi: (Type, Vec<Function>, Vec<Method>) =
            scrypto_decode(&rtn.raw).map_err(|_| WasmValidationError::InvalidBlueprintAbi)?;

        if let Type::Struct { name, fields: _ } = &abi.0 {
            blueprints.insert(name.clone(), abi);
        } else {
            return Err(WasmValidationError::InvalidBlueprintAbi);
        }
    }
    Ok(blueprints)
}

pub fn new_extracted_package(code: Vec<u8>) -> Result<Package, WasmValidationError> {
    let blueprints = extract_abi(&code)?;
    let package = Package{
        code,
        blueprints
    };
    Ok(package)
}