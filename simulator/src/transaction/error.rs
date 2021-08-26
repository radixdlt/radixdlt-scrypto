use radix_engine::execution::*;
use sbor::model::Type;
use sbor::DecodeError;
use scrypto::rust::fmt;
use scrypto::types::*;

/// Represents an error when parsing arguments.
#[derive(Debug)]
pub enum ParseArgError {
    /// The argument is not provided.
    MissingArgument(usize, Type),

    /// The argument is of unsupported type.
    UnsupportedType(usize, Type),

    /// Failed to parse argument
    UnableToParse(usize, Type, String),
}

/// Represents an error when construction a transaction.
#[derive(Debug)]
pub enum TxnConstructionError {
    /// The given packet does not exist.
    PackageNotFound(Address),

    /// The given component does not exist.
    ComponentNotFound(Address),

    /// The given blueprint function does not exist.
    FunctionNotFound(String),

    /// The given component method does not exist.
    MethodNotFound(String),

    /// The provided arguments do not match ABI.
    InvalidArguments(ParseArgError),

    /// Failed to export the blueprint ABI.
    FailedToExportAbi(RuntimeError),

    /// Failed to parse the ABI returned.
    FailedToParseAbi(DecodeError),
}

impl fmt::Display for ParseArgError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let msg = match self {
            Self::MissingArgument(i, ty) => {
                format!("Missing the {} argument of type {:?}", fmt_nth(*i), ty)
            }
            Self::UnsupportedType(i, ty) => format!(
                "The {} argument is of unsupported type {:?}",
                fmt_nth(*i),
                ty
            ),
            Self::UnableToParse(i, ty, arg) => format!(
                "Unable to parse the {} argument of type {:?} from {}",
                fmt_nth(*i),
                ty,
                arg
            ),
        };

        f.write_str(msg.as_str())
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
