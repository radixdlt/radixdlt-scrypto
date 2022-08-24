use core::str::FromStr;
use sbor::rust::string::String;
use sbor::{Decode, Encode, TypeId};

/// Network Definition is intended to be the actual definition of a network
#[derive(Debug, Clone, TypeId, Encode, Decode, PartialEq, Eq)]
pub struct NetworkDefinition {
    // TODO: we may be able to squeeze network identifier into the other fields, like the `v` byte in signature.
    pub id: u8,
    pub logical_name: String,
    pub hrp_suffix: String,
}

// NOTE: Most Network Definitions live in the node codebase
// Some are duplicated here so that they can be easily used by scrypto and resim
impl NetworkDefinition {
    pub fn local_simulator() -> NetworkDefinition {
        NetworkDefinition {
            id: 242,
            logical_name: String::from("simulator"),
            hrp_suffix: String::from("sim"),
        }
    }

    pub fn mainnet() -> NetworkDefinition {
        NetworkDefinition {
            id: 1,
            logical_name: String::from("mainnet"),
            hrp_suffix: String::from("rdx"),
        }
    }
}

impl FromStr for NetworkDefinition {
    type Err = NetworkError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "localsimulator" => Ok(NetworkDefinition::local_simulator()),
            "local_simulator" => Ok(NetworkDefinition::local_simulator()),
            "simulator" => Ok(NetworkDefinition::local_simulator()),
            "mainnet" => Ok(NetworkDefinition::mainnet()),
            _ => Err(NetworkError::InvalidNetworkString),
        }
    }
}

#[derive(Debug)]
pub enum NetworkError {
    InvalidNetworkString,
}
