use crate::types::*;
use sbor::rust::fmt::{Debug, Formatter};

/// A collection of blueprints, compiled and published as a single unit.
#[derive(Clone, TypeId, Encode, Decode, PartialEq, Eq)]
pub struct PackageInfoSubstate {
    pub code: Vec<u8>,
    pub blueprint_abis: BTreeMap<String, BlueprintAbi>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[scrypto(TypeId, Encode, Decode)]
pub struct PackageRoyaltyConfigSubstate {
    pub royalty_config: BTreeMap<String, RoyaltyConfig>,
}

impl Debug for PackageInfoSubstate {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("PackageInfoSubstate")
            .field("blueprint_abis", &self.blueprint_abis)
            .finish()
    }
}

impl PackageInfoSubstate {
    pub fn blueprint_abi(&self, blueprint_name: &str) -> Option<&BlueprintAbi> {
        self.blueprint_abis.get(blueprint_name)
    }

    pub fn code(&self) -> &[u8] {
        &self.code
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[scrypto(TypeId, Encode, Decode)]
pub struct PackageRoyaltyAccumulatorSubstate {
    pub royalty: Own,
}
