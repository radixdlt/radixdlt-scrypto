use crate::internal_prelude::*;
use crate::system::system_modules::costing::SystemLoanFeeReserve;
use crate::vm::wasm::*;
use crate::vm::wasm_runtime::NoOpWasmRuntime;
use crate::vm::ScryptoVmVersion;
use crate::{errors::InvokeError, transaction::CostingParameters};
use radix_engine_interface::blueprints::package::*;
use radix_transactions::prelude::TransactionCostingParameters;
use sbor::rust::iter;

#[derive(Debug)]
pub enum ExtractSchemaError {
    InvalidWasm(PrepareError),
    RunSchemaGenError(InvokeError<WasmRuntimeError>),
    SchemaDecodeError(DecodeError),
}

impl From<PrepareError> for ExtractSchemaError {
    fn from(value: PrepareError) -> Self {
        ExtractSchemaError::InvalidWasm(value)
    }
}

pub fn extract_definition(code: &[u8]) -> Result<PackageDefinition, ExtractSchemaError> {
    let function_exports = WasmModule::init(code)
        .and_then(WasmModule::to_bytes)?
        .1
        .into_iter()
        .filter(|s| s.ends_with("_schema"));

    // Validate WASM
    let validator = ScryptoV1WasmValidator::new(ScryptoVmVersion::latest());
    let code_hash = CodeHash(Hash([0u8; 32]));
    let instrumented_code = validator
        .validate(&code, iter::empty())
        .map_err(|e| ExtractSchemaError::InvalidWasm(e))?
        .0;

    // Execute with empty state (with default cost unit limit)
    let wasm_engine = DefaultWasmEngine::default();
    let fee_reserve = SystemLoanFeeReserve::new(
        CostingParameters::babylon_genesis(),
        TransactionCostingParameters {
            tip: Default::default(),
            free_credit_in_xrd: Decimal::try_from(PREVIEW_CREDIT_IN_XRD).unwrap(),
        },
        false,
    );
    let mut wasm_execution_units_consumed = 0;
    let mut runtime: Box<dyn WasmRuntime> = Box::new(NoOpWasmRuntime::new(
        fee_reserve,
        &mut wasm_execution_units_consumed,
    ));
    let mut instance = wasm_engine.instantiate(code_hash, &instrumented_code);
    let mut blueprints = index_map_new();
    for function_export in function_exports {
        let rtn = instance
            .invoke_export(&function_export, vec![], &mut runtime)
            .map_err(ExtractSchemaError::RunSchemaGenError)?;

        let name = function_export.replace("_schema", "").to_string();
        let blueprint_setup: BlueprintDefinitionInit =
            scrypto_decode(rtn.as_slice()).map_err(ExtractSchemaError::SchemaDecodeError)?;
        blueprints.insert(name.clone(), blueprint_setup);
    }

    Ok(PackageDefinition { blueprints })
}
