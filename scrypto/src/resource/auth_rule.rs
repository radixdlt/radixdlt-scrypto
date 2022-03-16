use sbor::*;
use crate::resource::*;

/// Authorization Rule
#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub enum AuthRule {
    Just(NonFungibleAddress),
}

impl AuthRule {
    pub fn just(non_fungible_address: NonFungibleAddress) -> Self {
        AuthRule::Just(non_fungible_address)
    }
}
