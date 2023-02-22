use radix_engine_interface::api::types::*;
use radix_engine_interface::math::Decimal;
use radix_engine_interface::*;
use sbor::rust::vec::Vec;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Sbor)]
pub enum MethodAuthorizationError {
    NotAuthorized,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, ScryptoSbor)]
pub enum HardDecimal {
    Amount(Decimal),
    InvalidSchemaPath,
    DisallowdValueType,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Sbor)]
pub enum HardCount {
    Count(u8),
    InvalidSchemaPath,
    DisallowdValueType,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, ScryptoSbor)]
pub enum HardResourceOrNonFungible {
    NonFungible(NonFungibleGlobalId),
    Resource(ResourceAddress),
    InvalidSchemaPath,
    DisallowdValueType,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, ScryptoSbor)]
pub enum HardProofRuleResourceList {
    List(Vec<HardResourceOrNonFungible>),
    InvalidSchemaPath,
    DisallowdValueType,
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
