use crate::errors::*;
use scrypto::rust::fmt;
use scrypto::values::ParseScryptoValueError;

pub use wasmi::HostError;

/// Represents an error when invoking an export of a scrypto module.
#[derive(Debug, PartialEq, Clone)]
pub enum InvokeError {
    MemoryAllocError,

    MemoryAccessError,

    InvalidScryptoValue(ParseScryptoValueError),

    WasmError,

    HostError(RuntimeError),

    ExportNotFound,

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
    FailedToInstantiate,
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
