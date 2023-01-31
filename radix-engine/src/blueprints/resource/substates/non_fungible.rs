use crate::blueprints::resource::*;
use crate::types::*;

#[derive(Debug, Clone, Categorize, Encode, Decode, PartialEq, Eq)]
pub struct NonFungibleSubstate(pub Option<NonFungible>);
