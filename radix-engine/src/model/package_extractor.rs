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

#[derive(Debug)]
pub enum ExtractAbiError {
    InvalidWasm(PrepareError),
    FailedToExportBlueprintAbi,
    InvalidBlueprintAbi,
}

fn extract_abi(
    code: &[u8],
) -> Result<HashMap<String, (Type, Vec<Function>, Vec<Method>)>, ExtractAbiError> {
    // TODO: A bit of a code smell to have validation here, remove at some point.
    let function_exports = ScryptoModule::init(code)
        .and_then(ScryptoModule::to_bytes)
        .map_err(ExtractAbiError::InvalidWasm)?
        .1;

    let runtime = NopWasmRuntime::new(EXPORT_ABI_TBD_LIMIT);
    let mut runtime_boxed: Box<dyn WasmRuntime> = Box::new(runtime);
    let mut wasm_engine = WasmiEngine::new();
    let mut instance = wasm_engine.instantiate(code);
    let mut blueprints = HashMap::new();
    for method_name in function_exports {
        let rtn = instance
            .invoke_export(&method_name, &ScryptoValue::unit(), &mut runtime_boxed)
            .map_err(|_| ExtractAbiError::FailedToExportBlueprintAbi)?;

        let abi: (Type, Vec<Function>, Vec<Method>) =
            scrypto_decode(&rtn.raw).map_err(|_| ExtractAbiError::InvalidBlueprintAbi)?;

        if let Type::Struct { name, fields: _ } = &abi.0 {
            blueprints.insert(name.clone(), abi);
        } else {
            return Err(ExtractAbiError::InvalidBlueprintAbi);
        }
    }
    Ok(blueprints)
}

pub fn extract_package(code: Vec<u8>) -> Result<Package, ExtractAbiError> {
    let blueprints = extract_abi(&code)?;
    let package = Package { code, blueprints };
    Ok(package)
}
