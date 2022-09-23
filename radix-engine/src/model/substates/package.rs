use sbor::rust::fmt::Debug;

use crate::types::*;

/// A collection of blueprints, compiled and published as a single unit.
#[derive(Debug, Clone, TypeId, Encode, Decode, PartialEq, Eq)]
pub struct PackageSubstate {
    code: Vec<u8>,
    blueprint_abis: HashMap<String, BlueprintAbi>,
}
