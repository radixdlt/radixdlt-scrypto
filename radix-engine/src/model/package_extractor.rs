use sbor::rust::boxed::Box;
use sbor::rust::collections::HashMap;
use sbor::rust::string::String;
use sbor::rust::vec::Vec;
use sbor::Type;
use scrypto::abi::{Function, Method};
use scrypto::buffer::scrypto_decode;
use scrypto::prelude::Package;
use scrypto::values::ScryptoValue;

use crate::wasm::*;

fn extract_abi(
    code: &[u8],
) -> Result<HashMap<String, (Type, Vec<Function>, Vec<Method>)>, WasmValidationError> {
    let runtime = NopWasmRuntime::new(EXPORT_ABI_TBD_LIMIT);
    let mut runtime_boxed: Box<dyn WasmRuntime> = Box::new(runtime);
    let mut wasm_engine = WasmiEngine::new();
    // TODO: A bit of a code smell to have validation here, remove at some point.
    wasm_engine.validate(code)?;
    wasm_engine.instrument(code).unwrap();
    let mut instance = wasm_engine.instantiate(code);
    let exports: Vec<String> = instance
        .function_exports()
        .into_iter()
        .filter(|e| e.ends_with("_abi") && e.len() > 4)
        .collect();

    let mut blueprints = HashMap::new();
    for method_name in exports {
        let rtn = instance
            .invoke_export(&method_name, &ScryptoValue::unit(), &mut runtime_boxed)
            .map_err(|_| WasmValidationError::FailedToExportBlueprintAbi)?;

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

pub fn extract_package(code: Vec<u8>) -> Result<Package, WasmValidationError> {
    let blueprints = extract_abi(&code)?;
    let package = Package { code, blueprints };
    Ok(package)
}
