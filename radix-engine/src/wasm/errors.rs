use scrypto::rust::fmt;
use scrypto::rust::string::String;
use scrypto::values::ParseScryptoValueError;
use wasmi::HostError;

use crate::engine::RuntimeError;

/// Represents an error when invoking an export of a scrypto module.
#[derive(Debug, PartialEq, Clone)]
pub enum InvokeError {
    MemoryAllocError,

    MemoryAccessError,

    InvalidScryptoValue(ParseScryptoValueError),

    WasmError,

    RuntimeError(RuntimeError),

    FunctionNotFound,

    InvalidCallData,

    MissingReturnData,

    InvalidReturnData,
}

impl fmt::Display for InvokeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl HostError for InvokeError {}

/// Represents an error when validating a WASM file.
#[derive(Debug, PartialEq, Clone)]
pub enum WasmValidationError {
    /// Failed to parse.
    FailedToParse,
    // Failed to instantiate.
    FailedToInstantiate(String),
    /// The wasm module contains a start function.
    StartFunctionNotAllowed,
    /// The wasm module uses float points.
    FloatingPointNotAllowed,
    /// The wasm module does not have the `memory` export.
    NoMemoryExport,
    /// The wasm module does not have the `scrypto_alloc` export.
    NoScryptoAllocExport,
    /// The wasm module does not have the `scrypto_free` export.
    NoScryptoFreeExport,
    /// TODO: remove
    UnableToExportBlueprintAbi,
    // TODO: remove
    InvalidBlueprintAbi,
}

/// Represents an error when instrumenting a WASM module.
#[derive(Debug, PartialEq, Clone)]
pub enum WasmInstrumentationError {}

/// Represents an error when executing a WASM module.
#[derive(Debug, PartialEq, Clone)]
pub enum WasmExecutionError {}
