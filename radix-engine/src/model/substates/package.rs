use sbor::rust::fmt::Debug;
use std::fmt::Formatter;

use crate::types::*;

/// A collection of blueprints, compiled and published as a single unit.
#[derive(Clone, TypeId, Encode, Decode, PartialEq, Eq)]
pub struct PackageSubstate {
    pub code: Vec<u8>,
    pub blueprint_abis: HashMap<String, BlueprintAbi>,
}

impl Debug for PackageSubstate {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("Package").finish()
    }
}

impl PackageSubstate {
    pub fn blueprint_abi(&self, blueprint_name: &str) -> Option<&BlueprintAbi> {
        self.blueprint_abis.get(blueprint_name)
    }

    pub fn code(&self) -> &[u8] {
        &self.code
    }
}
