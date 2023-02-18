use crate::blueprints::resource::*;
use crate::types::*;

#[derive(Debug, Clone, Sbor, PartialEq, Eq)]
pub struct NonFungibleSubstate(pub Option<NonFungible>);
