use sbor::rust::fmt;
use sbor::rust::string::String;
use scrypto::values::ParseScryptoValueError;
use wasmi::HostError;

use crate::engine::RuntimeError;

/// Represents an error when invoking an export of a Scrypto module.
#[derive(Debug, PartialEq, Clone)]
pub enum InvokeError {
    MemoryAllocError,

    MemoryAccessError,

    InvalidScryptoValue(ParseScryptoValueError),

    WasmError(String),

    RuntimeError(RuntimeError),

    FunctionNotFound,

    InvalidCallData,

    MissingReturnData,

    InvalidReturnData,

    OutOfTbd {
        limit: u32,
        balance: u32,
        required: u32,
    },
}

/// Represents an error when instrumenting a Scrypto module.
#[derive(Debug, PartialEq, Clone)]
pub enum InstrumentError {
    FailedToInjectInstructionMetering,

    FailedToInjectStackLimiter,

    FailedToExportModule,
}

/// Represents an error when validating a WASM file.
#[derive(Debug, PartialEq, Clone)]
pub enum WasmValidationError {
    /// Failed to parse.
    FailedToParse,
    /// Failed to instantiate.
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
    /// Failed to instrument wasm code.
    FailedToInstrumentCode,
    /// TODO: remove
    FailedToExportBlueprintAbi,
    // TODO: remove
    InvalidBlueprintAbi,
}

impl fmt::Display for InvokeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl HostError for InvokeError {}

#[cfg(not(feature = "alloc"))]
impl std::error::Error for InvokeError {}
