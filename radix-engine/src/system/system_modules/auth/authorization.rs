use crate::types::*;
use radix_engine_interface::blueprints::resource::ResourceOrNonFungible;

#[derive(Debug, Clone, PartialEq, Eq, Hash, ScryptoSbor)]
pub enum HardProofRule {
    Require(ResourceOrNonFungible),
    AmountOf(Decimal, ResourceAddress),
    AllOf(Vec<ResourceOrNonFungible>),
    AnyOf(Vec<ResourceOrNonFungible>),
    CountOf(u8, Vec<ResourceOrNonFungible>),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, ScryptoSbor)]
pub enum HardAuthRule {
    ProofRule(HardProofRule),
    AnyOf(Vec<HardAuthRule>),
    AllOf(Vec<HardAuthRule>),
}

/// Authorization of a method call
#[derive(Debug, Clone, PartialEq, Eq, Hash, ScryptoSbor)]
pub enum MethodAuthorization {
    AllowAll,
    DenyAll,
    Protected(HardAuthRule),
}
