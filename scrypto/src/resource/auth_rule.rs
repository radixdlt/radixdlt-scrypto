use crate::resource::*;
use crate::rust::vec::Vec;
use sbor::*;

/// Authorization Rule
#[derive(Debug, Clone, Describe, TypeId, Encode, Decode)]
pub enum AuthRule {
    NonFungible(NonFungibleAddress),
    AnyOfResource(ResourceDefId),
    OneOf(Vec<AuthRule>),
}