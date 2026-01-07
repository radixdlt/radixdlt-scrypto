use crate::*;

use sbor::*;

/// Represents type of non-fungible id
#[cfg_attr(feature = "fuzzing", derive(::arbitrary::Arbitrary))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Sbor)]
pub enum NonFungibleIdType {
    String,
    Integer,
    Bytes,
    RUID,
}
