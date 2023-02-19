use sbor::rust::str::FromStr;
use sbor::rust::string::String;
use sbor::*;

/// Network Definition is intended to be the actual definition of a network
#[derive(Debug, Clone, Sbor, PartialEq, Eq)]
pub struct NetworkDefinition {
    // TODO: we may be able to squeeze network identifier into the other fields, like the `v` byte in signature.
    pub id: u8,
    pub logical_name: String,
    pub hrp_suffix: String,
}

// NOTE: Most Network Definitions live in the node codebase
// Some are duplicated here so that they can be easily used by scrypto and resim
impl NetworkDefinition {
    pub fn simulator() -> NetworkDefinition {
        NetworkDefinition {
            id: 242,
            logical_name: String::from("simulator"),
            hrp_suffix: String::from("sim"),
        }
    }

    pub fn adapanet() -> NetworkDefinition {
        NetworkDefinition {
            id: 0x0a,
            logical_name: String::from("adapanet"),
            hrp_suffix: String::from("tdx_a_"),
        }
    }

    pub fn nebunet() -> NetworkDefinition {
        NetworkDefinition {
            id: 0x0b,
            logical_name: String::from("nebunet"),
            hrp_suffix: String::from("tdx_b_"),
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
    type Err = ParseNetworkError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "simulator" => Ok(NetworkDefinition::simulator()),
            "adapanet" => Ok(NetworkDefinition::adapanet()),
            "nebunet" => Ok(NetworkDefinition::nebunet()),
            "mainnet" => Ok(NetworkDefinition::mainnet()),
            _ => Err(ParseNetworkError::InvalidNetworkString),
        }
    }
}

#[derive(Debug)]
pub enum ParseNetworkError {
    InvalidNetworkString,
}
