use sbor::describe::Type;
use scrypto::engine::types::*;
use scrypto::rust::string::String;

use crate::engine::RuntimeError;

/// Represents an error when parsing arguments.
#[derive(Debug, Clone)]
pub enum BuildArgsError {
    /// The argument is not provided.
    MissingArgument(usize, Type),

    /// The argument is of unsupported type.
    UnsupportedType(usize, Type),

    /// Failure when parsing an argument.
    FailedToParse(usize, Type, String),
}

/// Represents an error when building a transaction.
#[derive(Debug, Clone)]
pub enum CallWithAbiError {
    /// The given blueprint function does not exist.
    FunctionNotFound(String),

    /// The given component method does not exist.
    MethodNotFound(String),

    /// The provided arguments do not match ABI.
    FailedToBuildArgs(BuildArgsError),

    /// Failed to export the ABI of a function.
    FailedToExportFunctionAbi(PackageAddress, String, String, RuntimeError),

    /// Failed to export the ABI of a method.
    FailedToExportMethodAbi(ComponentAddress, String),

    /// Account is required but not provided.
    AccountNotProvided,
}
