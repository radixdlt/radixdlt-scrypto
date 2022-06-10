use sbor::rust::boxed::Box;
use sbor::rust::collections::HashMap;
use sbor::rust::string::String;
use sbor::rust::vec::Vec;
use sbor::{DecodeError, Type};
use scrypto::abi::BlueprintAbi;
use scrypto::buffer::scrypto_decode;
use scrypto::prelude::Package;
use scrypto::values::ScryptoValue;

use crate::engine::NopWasmRuntime;
use crate::fee::{
    MAX_EXTRACT_ABI_COST, WASM_GROW_MEMORY, WASM_INSTRUCTION, WASM_MAX_STACK_SIZE, WASM_METERING_V1,
};
use crate::wasm::*;

#[derive(Debug)]
pub enum ExtractAbiError {
    InvalidWasm(PrepareError),
    FailedToExportBlueprintAbi(InvokeError),
    AbiDecodeError(DecodeError),
    InvalidBlueprintAbi,
}

fn extract_abi(code: &[u8]) -> Result<HashMap<String, BlueprintAbi>, ExtractAbiError> {
    let function_exports = WasmModule::init(code)
        .and_then(WasmModule::to_bytes)
        .map_err(ExtractAbiError::InvalidWasm)?
        .1
        .into_iter()
        .filter(|s| s.ends_with("_abi"));

    let metering_params = WasmMeteringParams::new(
        WASM_METERING_V1,
        WASM_INSTRUCTION,
        WASM_GROW_MEMORY,
        WASM_MAX_STACK_SIZE,
    );
    let instrumented_code = WasmInstrumenter::instrument(code, &metering_params);
    let mut runtime: Box<dyn WasmRuntime> = Box::new(NopWasmRuntime::new(MAX_EXTRACT_ABI_COST));
    let mut wasm_engine = WasmiEngine::new();
    let mut instance = wasm_engine.instantiate(&instrumented_code);
    let mut blueprints = HashMap::new();
    for method_name in function_exports {
        let rtn = instance
            .invoke_export(&method_name, "", &ScryptoValue::unit(), &mut runtime)
            .map_err(ExtractAbiError::FailedToExportBlueprintAbi)?;

        let abi: BlueprintAbi =
            scrypto_decode(&rtn.raw).map_err(ExtractAbiError::AbiDecodeError)?;

        if let Type::Struct { name, fields: _ } = &abi.structure {
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
