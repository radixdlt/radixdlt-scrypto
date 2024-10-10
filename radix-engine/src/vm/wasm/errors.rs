use crate::errors::{CanBeAbortion, InvokeError, RuntimeError, SelfError, VmError};
use crate::internal_prelude::*;
use crate::system::system_modules::costing::FeeReserveError;
use crate::transaction::AbortReason;

/// Represents an error when validating a WASM file.
#[derive(Debug, PartialEq, Eq, Clone, Sbor)]
pub enum PrepareError {
    /// Failed to deserialize.
    /// See <https://webassembly.github.io/spec/core/syntax/index.html>
    DeserializationError,
    /// Failed to validate
    /// See <https://webassembly.github.io/spec/core/valid/index.html>
    ValidationError(String),
    /// Failed to serialize.
    SerializationError,
    /// The wasm module contains a start function.
    StartFunctionNotAllowed,
    /// Invalid import section
    InvalidImport(InvalidImport),
    /// Invalid memory section
    InvalidMemory(InvalidMemory),
    /// Invalid table section
    InvalidTable(InvalidTable),
    /// Invalid export symbol name
    InvalidExportName(String),
    /// Too many targets in the `br_table` instruction
    TooManyTargetsInBrTable,
    /// Too many functions
    TooManyFunctions,
    /// Too many function parameters
    TooManyFunctionParams,
    /// Too many function local variables
    TooManyFunctionLocals { max: u32, actual: u32 },
    /// Too many globals
    TooManyGlobals { max: u32, current: u32 },
    /// No export section
    NoExportSection,
    /// Missing export
    MissingExport { export_name: String },
    /// The wasm module does not have the `scrypto_alloc` export.
    NoScryptoAllocExport,
    /// The wasm module does not have the `scrypto_free` export.
    NoScryptoFreeExport,
    /// Failed to inject instruction metering
    RejectedByInstructionMetering { reason: String },
    /// Failed to inject stack metering
    RejectedByStackMetering { reason: String },
    /// Not instantiatable
    NotInstantiatable { reason: String },
    /// Not compilable
    NotCompilable,
    /// Wrap errors returned by WasmInstrument::ModuleInfoError
    /// It is wrapped to String, because it's members cannot derive: Sbor, Eq and PartialEq
    ModuleInfoError(String),
    /// Wrap errors returned by wasmparser
    /// It is wrapped to String, because wasmparser error (BinaryReaderError) members cannot derive: Sbor, Eq and PartialEq
    WasmParserError(String),
    /// An overflow occurred in some of the internal math
    Overflow,
}

#[derive(Debug, PartialEq, Eq, Clone, Sbor)]
pub enum InvalidImport {
    /// The import is not allowed
    ImportNotAllowed(String),
    /// Scrypto VM version protocol mismatch
    ProtocolVersionMismatch {
        name: String,
        current_version: u64,
        expected_version: u64,
    },
    InvalidFunctionType(String),
}

#[derive(Debug, PartialEq, Eq, Clone, Sbor)]
pub enum InvalidMemory {
    /// The wasm module has no memory section.
    MissingMemorySection,
    /// The memory section is empty.
    NoMemoryDefinition,
    /// The memory section contains too many memory definitions.
    TooManyMemoryDefinition,
    /// The memory size exceeds the limit.
    MemorySizeLimitExceeded,
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
    /// Host attempted to call unknown WASM function, addressed by name.
    UnknownExport(String),

    /// Error when reading wasm memory.
    MemoryAccessError,

    /// WASM function return is not a `u64` fat pointer which points to a valid memory range.
    InvalidWasmPointer,

    /// WASM execution error, including trap.
    ExecutionError(String),

    /// Not implemented, no-op wasm runtime
    NotImplemented,

    /// Buffer not found
    BufferNotFound(BufferId),

    InvalidAddress(DecodeError),

    /// Invalid method ident
    InvalidString,

    /// Invalid RE node ID
    InvalidNodeId,

    InvalidGlobalAddressReservation,

    /// Invalid reference type
    InvalidReferenceType(u32),

    /// Invalid RE module ID
    InvalidAttachedModuleId(u32),

    /// Invalid initial app states
    InvalidObjectStates(DecodeError),

    /// Invalid access rules
    InvalidAccessRule(DecodeError),

    /// Invalid modules
    InvalidModules(DecodeError),

    InvalidTemplateArgs(DecodeError),

    InvalidKeyValueStoreSchema(DecodeError),

    /// Invalid component address
    InvalidLockFlags,

    /// Invalid log level
    InvalidLogLevel(DecodeError),

    /// Costing error (no-op runtime only!)
    FeeReserveError(FeeReserveError),

    InvalidEventFlags(u32),

    InvalidPackageAddress,

    TooManyBuffers,

    InvalidBlsPublicKey(DecodeError),
    InvalidBlsSignature(DecodeError),
    InvalidBlsPublicKeyOrMessage(DecodeError),

    InputDataEmpty,

    InvalidEd25519PublicKey(ParseEd25519PublicKeyError),
    InvalidEd25519Signature(ParseEd25519SignatureError),

    InvalidSecp256k1PublicKey(ParseSecp256k1PublicKeyError),
    InvalidSecp256k1Signature(ParseSecp256k1SignatureError),

    InvalidHash(ParseHashError),
    Secp256k1KeyRecoveryError,
}

impl SelfError for WasmRuntimeError {
    fn into_runtime_error(self) -> RuntimeError {
        RuntimeError::VmError(VmError::Wasm(self))
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
