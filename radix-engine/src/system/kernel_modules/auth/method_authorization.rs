use crate::types::*;
use radix_engine_interface::blueprints::resource::NonFungibleGlobalId;

#[derive(Debug, Clone, PartialEq, Eq, Hash, ScryptoSbor)]
pub enum HardDecimal {
    Amount(Decimal),
    InvalidPath,
    NotDecimal,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Sbor)]
pub enum HardCount {
    Count(u8),
    InvalidPath,
    NotU8,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, ScryptoSbor)]
pub enum HardResourceOrNonFungible {
    NonFungible(NonFungibleGlobalId),
    Resource(ResourceAddress),
    InvalidPath,
    NotResourceAddress,
    NotResourceAddressOrNonFungibleGlobalId,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, ScryptoSbor)]
pub enum HardProofRuleResourceList {
    List(Vec<HardResourceOrNonFungible>),
    InvalidPath,
    NotResourceAddressOrNonFungibleGlobalIdArray,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, ScryptoSbor)]
pub enum HardProofRule {
    Require(HardResourceOrNonFungible),
    AmountOf(HardDecimal, HardResourceOrNonFungible),
    AllOf(HardProofRuleResourceList),
    AnyOf(HardProofRuleResourceList),
    CountOf(HardCount, HardProofRuleResourceList),
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
