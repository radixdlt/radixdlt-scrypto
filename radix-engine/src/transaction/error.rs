use sbor::describe::Type;
use scrypto::rust::borrow::ToOwned;
use scrypto::rust::fmt;
use scrypto::rust::format;
use scrypto::rust::string::String;
use scrypto::types::*;

/// Represents an error when parsing arguments.
#[derive(Clone)]
pub enum BuildArgsError {
    /// The argument is not provided.
    MissingArgument(usize, Type),

    /// The argument is of unsupported type.
    UnsupportedType(usize, Type),

    /// Failure when parsing an argument.
    ParseDataFailure(usize, Type, String),
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

impl fmt::Debug for BuildArgsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let msg = match self {
            Self::MissingArgument(i, ty) => {
                format!("The {} argument (type: {:?}) is missing", fmt_nth(*i), ty)
            }
            Self::UnsupportedType(i, ty) => format!(
                "The {} argument (type {:?}) is of unsupported type",
                fmt_nth(*i),
                ty
            ),
            Self::ParseDataFailure(i, ty, arg) => format!(
                "The {} argument (type {:?}) can't be parsed from {}",
                fmt_nth(*i),
                ty,
                arg
            ),
        };

        f.write_str(msg.as_str())
    }
}

impl fmt::Display for BuildArgsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

fn fmt_nth(i: usize) -> String {
    match i {
        0 => "1st".to_owned(),
        1 => "2nd".to_owned(),
        2 => "3rd".to_owned(),
        _ => format!("{}th", i + 1),
    }
}
