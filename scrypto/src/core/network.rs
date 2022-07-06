use core::str::FromStr;
use once_cell::unsync::Lazy;
use sbor::rust::string::String;
use sbor::{Decode, Encode, ToString, TypeId};

// TODO: we may be able to squeeze network identifier into the other fields, like the `v` byte in signature.
#[derive(Debug, Clone, TypeId, Encode, Decode, PartialEq, Eq)]
pub enum Network {
    LocalSimulator,
    InternalTestnet,
}

// TODO: Generate through macro.
impl FromStr for Network {
    type Err = NetworkError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "LocalSimulator" => Ok(Self::LocalSimulator),
            "InternalTestnet" => Ok(Self::InternalTestnet),
            _ => Err(NetworkError::InvalidNetworkString),
        }
    }
}

#[derive(Debug)]
pub enum NetworkError {
    InvalidNetworkString,
}

#[cfg(target_arch = "wasm32")]
pub const CURRENT_NETWORK: Lazy<Network> = Lazy::new(|| {
    use super::Runtime;
    Runtime::transaction_network()
});

#[cfg(not(target_arch = "wasm32"))]
pub const CURRENT_NETWORK: Lazy<Network> = Lazy::new(|| Network::LocalSimulator);
