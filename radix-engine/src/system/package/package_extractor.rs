use radix_engine_interface::schema::BlueprintSchema;
use radix_engine_interface::schema::PackageSchema;

use crate::errors::InvokeError;
use crate::kernel::interpreters::NopWasmRuntime;
use crate::system::kernel_modules::costing::SystemLoanFeeReserve;
use crate::types::*;
use crate::wasm::*;

#[derive(Debug)]
pub enum ExtractAbiError {
    InvalidWasm(PrepareError),
    FailedToExportBlueprintAbi(InvokeError<WasmRuntimeError>),
    AbiDecodeError(DecodeError),
    InvalidBlueprintAbi,
}

pub fn extract_schema(code: &[u8]) -> Result<PackageSchema, ExtractAbiError> {
    let function_exports = WasmModule::init(code)
        .and_then(WasmModule::to_bytes)
        .map_err(ExtractAbiError::InvalidWasm)?
        .1
        .into_iter()
        .filter(|s| s.ends_with("_abi"));

    let wasm_engine = DefaultWasmEngine::default();
    let wasm_instrumenter = WasmInstrumenter::default();
    let instrumented_code = wasm_instrumenter.instrument(
        PackageAddress::Normal([0u8; 26]),
        code,
        WasmMeteringConfig::V0,
    );
    let fee_reserve = SystemLoanFeeReserve::no_fee();
    let mut runtime: Box<dyn WasmRuntime> = Box::new(NopWasmRuntime::new(fee_reserve));
    let mut instance = wasm_engine.instantiate(&instrumented_code);
    let mut blueprints = BTreeMap::new();
    for function_export in function_exports {
        let rtn = instance
            .invoke_export(&function_export, vec![], &mut runtime)
            .map_err(ExtractAbiError::FailedToExportBlueprintAbi)?;

        let name = function_export.replace("_abi", "").to_string();
        let schema: BlueprintSchema =
            scrypto_decode(rtn.as_slice()).map_err(ExtractAbiError::AbiDecodeError)?;

        blueprints.insert(name, schema);
    }
    Ok(PackageSchema { blueprints })
}
