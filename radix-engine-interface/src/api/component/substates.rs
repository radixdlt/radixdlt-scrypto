use crate::api::types::*;
use crate::data::model::Own;
use radix_engine_derive::*;

#[derive(Debug, Clone, Sbor, PartialEq, Eq)]
pub struct ComponentStateSubstate {
    pub raw: Vec<u8>,
}

impl ComponentStateSubstate {
    pub fn new(raw: Vec<u8>) -> Self {
        ComponentStateSubstate { raw }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub struct TypeInfoSubstate {
    pub package_address: PackageAddress,
    pub blueprint_name: String,
}

impl TypeInfoSubstate {
    pub fn new(package_address: PackageAddress, blueprint_name: String) -> Self {
        Self {
            package_address,
            blueprint_name,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub struct ComponentRoyaltyConfigSubstate {
    pub royalty_config: RoyaltyConfig,
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub struct ComponentRoyaltyAccumulatorSubstate {
    pub royalty: Own,
}
