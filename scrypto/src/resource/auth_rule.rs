use crate::resource::*;
use crate::rust::vec::Vec;
use sbor::*;
use scrypto::math::Decimal;

/// Authorization Rule
#[derive(Debug, Clone, Describe, TypeId, Encode, Decode)]
pub enum AuthRule {
    NonFungible(NonFungibleAddress),
    AnyOfResource(ResourceDefId),
    SomeOfResource(Decimal, ResourceDefId),
    AllOf(Vec<AuthRule>),
    OneOf(Vec<AuthRule>),
    CountOf { count: u8, rules: Vec<AuthRule> },
}
