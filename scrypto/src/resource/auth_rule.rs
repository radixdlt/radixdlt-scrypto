use crate::resource::*;
use crate::rust::vec::Vec;
use sbor::*;

/// Authorization Rule
#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub enum AuthRule {
    NonFungible(NonFungibleAddress),
    OneOf(Vec<AuthRule>),
}

impl AuthRule {
    pub fn just(non_fungible_address: NonFungibleAddress) -> Self {
        AuthRule::NonFungible(non_fungible_address)
    }
}
