use crate::model::NonFungible;
use crate::types::*;

#[derive(Debug, Clone, Categorize, Encode, Decode, PartialEq, Eq)]
pub struct NonFungibleSubstate(pub Option<NonFungible>);
