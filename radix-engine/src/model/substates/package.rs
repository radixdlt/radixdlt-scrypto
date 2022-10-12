use sbor::rust::fmt::Debug;

use crate::types::*;

/// A collection of blueprints, compiled and published as a single unit.
#[derive(Debug, Clone, TypeId, Encode, Decode, PartialEq, Eq)]
pub struct PackageSubstate {
    pub code: Vec<u8>,
    pub blueprint_abis: HashMap<String, BlueprintAbi>,
}

impl PackageSubstate {
    pub fn blueprint_abi(&self, blueprint_name: &str) -> Option<&BlueprintAbi> {
        self.blueprint_abis.get(blueprint_name)
    }

    pub fn code(&self) -> &[u8] {
        &self.code
    }
}
