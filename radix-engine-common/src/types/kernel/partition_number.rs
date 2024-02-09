use crate::*;
#[cfg(feature = "radix_engine_fuzzing")]
use arbitrary::Arbitrary;
use sbor::rust::prelude::*;

/// The unique identifier of a node module.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Sbor)]
#[sbor(transparent)]
pub struct PartitionNumber(pub u8);

impl PartitionNumber {
    pub const fn at_offset(self, offset: PartitionOffset) -> Option<Self> {
        match self.0.checked_add(offset.0) {
            Some(n) => Some(Self(n)),
            None => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Sbor)]
pub struct PartitionOffset(pub u8);
