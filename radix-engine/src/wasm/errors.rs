use crate::errors::{CanBeAbortion, InvokeError, KernelError, RuntimeError, SelfError};
use crate::system::kernel_modules::costing::FeeReserveError;
use crate::transaction::AbortReason;
use crate::types::*;

/// Represents an error when validating a WASM file.
#[derive(Debug, PartialEq, Eq, Clone, Sbor)]
pub enum PrepareError {
    /// Failed to deserialize.
    /// See <https://webassembly.github.io/spec/core/syntax/index.html>
    DeserializationError,
    /// Failed to validate
    /// See <https://webassembly.github.io/spec/core/valid/index.html>
    ValidationError,
    /// Failed to serialize.
    SerializationError,
    /// The wasm module contains a start function.
    StartFunctionNotAllowed,
    /// The wasm module uses float points.
    FloatingPointNotAllowed,
    /// Invalid import section
    InvalidImport(InvalidImport),
    /// Invalid memory section
    InvalidMemory(InvalidMemory),
    /// Invalid table section
    InvalidTable(InvalidTable),
    /// Too many targets in the `br_table` instruction
    TooManyTargetsInBrTable,
    /// Too many functions
    TooManyFunctions,
    /// Too many globals
    TooManyGlobals,
    /// No export section
    NoExportSection,
    /// Missing export
    MissingExport { export_name: String },
    /// The wasm module does not have the `scrypto_alloc` export.
    NoScryptoAllocExport,
    /// The wasm module does not have the `scrypto_free` export.
    NoScryptoFreeExport,
    /// Failed to inject instruction metering
    RejectedByInstructionMetering,
    /// Failed to inject stack metering
    RejectedByStackMetering,
    /// Not instantiatable
    NotInstantiatable,
    /// Not compilable
    NotCompilable,
}

#[derive(Debug, PartialEq, Eq, Clone, Sbor)]
pub enum InvalidImport {
    /// The import is not allowed
    ImportNotAllowed,
    InvalidFunctionType(String),
}

#[derive(Debug, PartialEq, Eq, Clone, Sbor)]
pub enum InvalidMemory {
    /// The wasm module has no memory section.
    NoMemorySection,
    /// The memory section is empty.
    EmptyMemorySection,
    /// The memory section contains too many memory definitions.
    TooManyMemories,
    /// The initial memory size is too large.
    InitialMemorySizeLimitExceeded,
    /// The wasm module does not have the `memory` export.
    MemoryNotExported,
}

#[derive(Debug, PartialEq, Eq, Clone, Sbor)]
pub enum InvalidTable {
    /// More than one table defined, against WebAssembly MVP spec
    MoreThanOneTable,
    /// Initial table size too large
    InitialTableSizeLimitExceeded,
}

/// Represents an error when invoking an export of a Scrypto module.
#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum WasmRuntimeError {
    /// Error when reading wasm memory.
    MemoryAccessError,

    /// WASM attempted to call undefined host function, addressed by offset.
    UnknownHostFunction(usize),

    /// Host attempted to call unknown WASM function, addressed by name.
    UnknownWasmFunction(String),

    /// WASM interpreter error, such as traps.
    InterpreterError(String),

    /// WASM function return is not a `u64` fat pointer which points to a valid memory range.
    InvalidWasmPointer,

    Trap(String),

    //=================
    // Scrypto Runtime
    //=================
    /// Not implemented, no-op wasm runtime
    NotImplemented,
    /// Buffer not found
    BufferNotFound(BufferId),
    /// Invalid package address
    InvalidPackageAddress(DecodeError),
    /// Invalid method ident
    InvalidString,
    /// Invalid RE node ID
    InvalidNodeId,
    /// Invalid RE module ID
    InvalidModuleId(u32),
    /// Invalid substate offset
    InvalidSubstateKey,
    /// Invalid initial app states
    InvalidAppStates(DecodeError),
    /// Invalid access rules
    InvalidAccessRules(DecodeError),
    /// Invalid access rules
    InvalidSchema(DecodeError),
    /// Invalid modules
    InvalidModules(DecodeError),
    /// Invalid royalty config
    InvalidRoyaltyConfig(DecodeError),
    /// Invalid metadata
    InvalidMetadata(DecodeError),
    /// Invalid component id
    InvalidComponentId(DecodeError),
    InvalidKeyValueStoreSchema(DecodeError),
    InvalidValue(DecodeError),
    // Invalid EventSchema
    InvalidEventSchema(DecodeError),
    /// Invalid component address
    InvalidLockFlags,
    /// Invalid log level
    InvalidLogLevel(DecodeError),

    //=============
    // No-op Runtime
    //=============
    /// Costing error
    FeeReserveError(FeeReserveError),
}

impl SelfError for WasmRuntimeError {
    fn into_runtime_error(self) -> RuntimeError {
        RuntimeError::KernelError(KernelError::WasmRuntimeError(self))
    }
}

impl CanBeAbortion for WasmRuntimeError {
    fn abortion(&self) -> Option<&AbortReason> {
        match self {
            WasmRuntimeError::FeeReserveError(err) => err.abortion(),
            _ => None,
        }
    }
}

impl fmt::Display for WasmRuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[cfg(not(feature = "alloc"))]
impl std::error::Error for WasmRuntimeError {}

impl fmt::Display for InvokeError<WasmRuntimeError> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[cfg(not(feature = "alloc"))]
impl std::error::Error for InvokeError<WasmRuntimeError> {}
