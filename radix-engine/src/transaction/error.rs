use sbor::describe::Type;
use scrypto::rust::string::String;
use scrypto::types::*;

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
pub enum BuildTransactionError {
    /// The given blueprint function does not exist.
    FunctionNotFound(String),

    /// The given component method does not exist.
    MethodNotFound(String),

    /// The provided arguments do not match ABI.
    FailedToBuildArgs(BuildArgsError),

    /// Failed to export the ABI of a function.
    FailedToExportFunctionAbi(Address, String, String),

    /// Failed to export the ABI of a method.
    FailedToExportMethodAbi(Address, String),

    /// Account is required but not provided.
    AccountNotProvided,
}
