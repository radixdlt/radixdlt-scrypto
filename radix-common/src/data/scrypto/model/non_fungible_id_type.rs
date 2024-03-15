use crate::*;
#[cfg(feature = "fuzzing")]
use arbitrary::Arbitrary;
use sbor::*;

/// Represents type of non-fungible id
#[cfg_attr(feature = "fuzzing", derive(Arbitrary))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Sbor)]
pub enum NonFungibleIdType {
    String,
    Integer,
    Bytes,
    RUID,
}
