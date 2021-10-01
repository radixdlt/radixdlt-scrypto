use sbor::describe::Type;
use scrypto::rust::borrow::ToOwned;
use scrypto::rust::fmt;
use scrypto::rust::format;
use scrypto::rust::string::String;

/// Represents an error when parsing arguments.
#[derive(Debug, Clone)]
pub enum BuildArgsError {
    /// The argument is not provided.
    MissingArgument(usize, Type),

    /// The argument is of unsupported type.
    UnsupportedType(usize, Type),

    /// Failed to parse argument.
    UnableToParse(usize, Type, String),
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

    AccountNotProvided,
}

impl fmt::Display for BuildArgsError {
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
