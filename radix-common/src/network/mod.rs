use sbor::rust::prelude::*;
use sbor::*;

/// Network Definition is intended to be the actual definition of a network
#[derive(Debug, Clone, Sbor, PartialEq, Eq)]
pub struct NetworkDefinition {
    pub id: u8,
    pub logical_name: Cow<'static, str>,
    pub hrp_suffix: Cow<'static, str>,
}

// NOTE: Most Network Definitions live in the node codebase
// Some are duplicated here so that they can be easily used by scrypto and resim
impl NetworkDefinition {
    /// Used when running resim, and for engine/scrypto tests
    pub const fn simulator() -> NetworkDefinition {
        NetworkDefinition {
            id: 242,
            logical_name: Cow::Borrowed("simulator"),
            hrp_suffix: Cow::Borrowed("sim"),
        }
    }

    /// Used for running a local node
    pub const fn localnet() -> NetworkDefinition {
        NetworkDefinition {
            id: 240,
            logical_name: Cow::Borrowed("localnet"),
            hrp_suffix: Cow::Borrowed("loc"),
        }
    }

    /// The network definition for Alphanet
    pub const fn adapanet() -> NetworkDefinition {
        NetworkDefinition {
            id: 0x0a,
            logical_name: Cow::Borrowed("adapanet"),
            hrp_suffix: Cow::Borrowed("tdx_a_"),
        }
    }

    /// The network definition for Betanet
    pub const fn nebunet() -> NetworkDefinition {
        NetworkDefinition {
            id: 0x0b,
            logical_name: Cow::Borrowed("nebunet"),
            hrp_suffix: Cow::Borrowed("tdx_b_"),
        }
    }

    /// The network definition for RCnet v1
    pub const fn kisharnet() -> NetworkDefinition {
        NetworkDefinition {
            id: 0x0c,
            logical_name: Cow::Borrowed("kisharnet"),
            hrp_suffix: Cow::Borrowed("tdx_c_"),
        }
    }

    /// The network definition for RCnet v2
    pub const fn ansharnet() -> NetworkDefinition {
        NetworkDefinition {
            id: 0x0d,
            logical_name: Cow::Borrowed("ansharnet"),
            hrp_suffix: Cow::Borrowed("tdx_d_"),
        }
    }

    /// The network definition for RCnet v3
    pub const fn zabanet() -> NetworkDefinition {
        NetworkDefinition {
            id: 0x0e,
            logical_name: Cow::Borrowed("zabanet"),
            hrp_suffix: Cow::Borrowed("tdx_e_"),
        }
    }

    pub const fn stokenet() -> NetworkDefinition {
        NetworkDefinition {
            id: 2,
            logical_name: Cow::Borrowed("stokenet"),
            hrp_suffix: Cow::Borrowed("tdx_2_"),
        }
    }

    pub const fn mainnet() -> NetworkDefinition {
        NetworkDefinition {
            id: 1,
            logical_name: Cow::Borrowed("mainnet"),
            hrp_suffix: Cow::Borrowed("rdx"),
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
            "kisharnet" => Ok(NetworkDefinition::kisharnet()),
            "ansharnet" => Ok(NetworkDefinition::ansharnet()),
            "zabanet" => Ok(NetworkDefinition::zabanet()),
            "stokenet" => Ok(NetworkDefinition::stokenet()),
            "mainnet" => Ok(NetworkDefinition::mainnet()),
            _ => Err(ParseNetworkError::InvalidNetworkString),
        }
    }
}

#[derive(Debug)]
pub enum ParseNetworkError {
    InvalidNetworkString,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::internal_prelude::*;

    #[test]
    fn network_from_string_fail() {
        assert_matches!(
            NetworkDefinition::from_str("non_existing_network").unwrap_err(),
            ParseNetworkError::InvalidNetworkString
        );
    }

    #[test]
    fn network_ids() {
        let array = [
            ("mainnet", 1),
            ("Simulator", 242),
            ("Adapanet", 10),
            ("NEBUNET", 11),
            ("Kisharnet", 12),
            ("ansharnet", 13),
            ("zabanet", 14),
        ];

        for (name, id) in array {
            assert_eq!(NetworkDefinition::from_str(name).unwrap().id, id)
        }
    }
}
