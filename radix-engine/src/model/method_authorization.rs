use crate::types::*;

#[derive(Debug, Clone, PartialEq, Eq, Hash, TypeId, Encode, Decode)]
pub enum MethodAuthorizationError {
    NotAuthorized,
    UnsupportedMethod,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, TypeId, Encode, Decode)]
pub enum HardDecimal {
    Amount(Decimal),
    SoftDecimalNotFound,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, TypeId, Encode, Decode)]
pub enum HardCount {
    Count(u8),
    SoftCountNotFound,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, TypeId, Encode, Decode)]
pub enum HardResourceOrNonFungible {
    NonFungible(NonFungibleAddress),
    Resource(ResourceAddress),
    SoftResourceNotFound,
}

impl From<NonFungibleAddress> for HardResourceOrNonFungible {
    fn from(non_fungible_address: NonFungibleAddress) -> Self {
        HardResourceOrNonFungible::NonFungible(non_fungible_address)
    }
}

impl From<ResourceAddress> for HardResourceOrNonFungible {
    fn from(resource_address: ResourceAddress) -> Self {
        HardResourceOrNonFungible::Resource(resource_address)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, TypeId, Encode, Decode)]
pub enum HardProofRuleResourceList {
    List(Vec<HardResourceOrNonFungible>),
    SoftResourceListNotFound,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, TypeId, Encode, Decode)]
pub enum HardProofRule {
    Require(HardResourceOrNonFungible),
    AmountOf(HardDecimal, HardResourceOrNonFungible),
    AllOf(HardProofRuleResourceList),
    AnyOf(HardProofRuleResourceList),
    CountOf(HardCount, HardProofRuleResourceList),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, TypeId, Encode, Decode)]
pub enum HardAuthRule {
    ProofRule(HardProofRule),
    AnyOf(Vec<HardAuthRule>),
    AllOf(Vec<HardAuthRule>),
}

/// Authorization of a method call
#[derive(Debug, Clone, PartialEq, Eq, Hash, TypeId, Encode, Decode)]
pub enum MethodAuthorization {
    Protected(HardAuthRule),
    AllowAll,
    DenyAll,
    Unsupported,
}
