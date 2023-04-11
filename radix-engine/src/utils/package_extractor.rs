use radix_engine_interface::schema::BlueprintSchema;
use radix_engine_interface::schema::PackageSchema;

use crate::errors::InvokeError;
use crate::kernel::interpreters::NopWasmRuntime;
use crate::system::kernel_modules::costing::SystemLoanFeeReserve;
use crate::types::*;
use crate::wasm::*;

#[derive(Debug)]
pub enum ExtractSchemaError {
    InvalidWasm(PrepareError),
    RunSchemaGenError(InvokeError<WasmRuntimeError>),
    DecodeError(DecodeError),
}

pub fn extract_schema(code: &[u8]) -> Result<PackageSchema, ExtractSchemaError> {
    let function_exports = WasmModule::init(code)
        .and_then(WasmModule::to_bytes)
        .map_err(ExtractSchemaError::InvalidWasm)?
        .1
        .into_iter()
        .filter(|s| s.ends_with("_schema"));

    let wasm_engine = DefaultWasmEngine::default();
    let wasm_instrumenter = WasmInstrumenter::default();
    let instrumented_code = wasm_instrumenter.instrument(
        PackageAddress::new_unchecked([0u8; 27]),
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
            .map_err(ExtractSchemaError::RunSchemaGenError)?;

        let name = function_export.replace("_schema", "").to_string();
        let schema: BlueprintSchema =
            scrypto_decode(rtn.as_slice()).map_err(ExtractSchemaError::DecodeError)?;

        blueprints.insert(name, schema);
    }
    Ok(PackageSchema { blueprints })
}
