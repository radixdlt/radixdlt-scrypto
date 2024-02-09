use crate::*;
#[cfg(feature = "radix_engine_fuzzing")]
use arbitrary::Arbitrary;
use sbor::rust::prelude::*;

/// The unique identifier of a substate within a node module.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Sbor)]
pub enum SubstateKey {
    Field(FieldKey),
    Map(MapKey),
    Sorted(SortedKey),
}

impl SubstateKey {
    pub fn for_field(&self) -> Option<&FieldKey> {
        match self {
            SubstateKey::Field(key) => Some(key),
            _ => None,
        }
    }

    pub fn for_map(&self) -> Option<&MapKey> {
        match self {
            SubstateKey::Map(key) => Some(key),
            _ => None,
        }
    }

    pub fn into_map(self) -> MapKey {
        match self {
            SubstateKey::Map(key) => key,
            _ => panic!("Not a Map Key"),
        }
    }

    pub fn for_sorted(&self) -> Option<&SortedKey> {
        match self {
            SubstateKey::Sorted(key) => Some(key),
            _ => None,
        }
    }
}

pub type FieldKey = u8;
pub type MapKey = Vec<u8>;
pub type SortedKey = ([u8; 2], Vec<u8>);
