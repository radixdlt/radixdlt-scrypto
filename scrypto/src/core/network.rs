use core::str::FromStr;
use sbor::{Decode, Encode, TypeId};

/// Generates an Enum and defines its `FromStr` implementation.
macro_rules! network_enum_with_from_str {
    (
        $(#[$meta:meta])*
        $vis:vis enum $enum_name:ident {
            $($variant:ident),*
        }
    ) => {
        $(#[$meta])*
        $vis enum $enum_name{
            $(
                $variant,
            )*
        }

        impl FromStr for $enum_name {
            type Err = NetworkError;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                match s {
                    $(
                        stringify!($variant) => Ok(Self::$variant),
                    )*
                    _ => Err(NetworkError::InvalidNetworkString),
                }
            }
        }
    };
}

network_enum_with_from_str! {
    // TODO: we may be able to squeeze network identifier into the other fields, like the `v` byte in signature.
    #[derive(Debug, Clone, TypeId, Encode, Decode, PartialEq, Eq)]
    pub enum Network {
        LocalSimulator,
        InternalTestnet
    }
}

#[derive(Debug)]
pub enum NetworkError {
    InvalidNetworkString,
}
