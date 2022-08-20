use core::str::FromStr;
use sbor::{Decode, Encode, TypeId};

/// Network Definition is intended to be the actual definition of a network
#[derive(Debug, Clone, TypeId, Encode, Decode, PartialEq, Eq)]
pub struct NetworkDefinition {
    pub id: u8,
    pub name: String,
    pub hrp_suffix: String,
}

// TODO: we may be able to squeeze network identifier into the other fields, like the `v` byte in signature.
/// Network is intended to be an "easy" method to get common Network configuration for (eg) tests
#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub enum Network {
    LocalSimulator,
    Mainnet,
    Custom(NetworkDefinition),
}

impl Network {
    pub fn get_definition(&self) -> NetworkDefinition {
        match self {
            Network::LocalSimulator => NetworkDefinition {
                id: 242,
                name: String::from("LocalSimulator"),
                hrp_suffix: String::from("sim"),
            },
            Network::Mainnet => NetworkDefinition {
                id: 1,
                name: String::from("Mainnet"),
                hrp_suffix: String::from("rdx"),
            },
            Network::Custom(definition) => definition.clone(),
        }
    }

    pub fn get_id(&self) -> u8 {
        return self.get_definition().id;
    }
}

impl PartialEq for Network {
    fn eq(&self, other: &Self) -> bool {
        self.get_definition() == other.get_definition()
    }
}

impl Eq for Network {}

impl FromStr for Network {
    type Err = NetworkError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "LocalSimulator" => Ok(Network::LocalSimulator),
            _ => Err(NetworkError::InvalidNetworkString),
        }
    }
}

#[derive(Debug)]
pub enum NetworkError {
    InvalidNetworkString,
}
