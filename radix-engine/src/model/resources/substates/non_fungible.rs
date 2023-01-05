use crate::model::NonFungible;
use crate::types::*;
use radix_engine_interface::*;

#[scrypto(Debug, Clone, TypeId, Encode, Decode, PartialEq, Eq)]
pub struct NonFungibleSubstate(pub Option<NonFungible>);
