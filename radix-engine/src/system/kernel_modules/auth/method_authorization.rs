use radix_engine_interface::math::Decimal;
use radix_engine_interface::model::*;
use radix_engine_interface::*;
use sbor::rust::vec::Vec;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Categorize, Encode, Decode)]
pub enum MethodAuthorizationError {
    NotAuthorized,
    UnsupportedMethod,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub enum HardDecimal {
    Amount(Decimal),
    InvalidSchemaPath,
    DisallowdValueType,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Categorize, Encode, Decode)]
pub enum HardCount {
    Count(u8),
    InvalidSchemaPath,
    DisallowdValueType,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub enum HardResourceOrNonFungible {
    NonFungible(NonFungibleGlobalId),
    Resource(ResourceAddress),
    InvalidSchemaPath,
    DisallowdValueType,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub enum HardProofRuleResourceList {
    List(Vec<HardResourceOrNonFungible>),
    InvalidSchemaPath,
    DisallowdValueType,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub enum HardProofRule {
    Require(HardResourceOrNonFungible),
    AmountOf(HardDecimal, HardResourceOrNonFungible),
    AllOf(HardProofRuleResourceList),
    AnyOf(HardProofRuleResourceList),
    CountOf(HardCount, HardProofRuleResourceList),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub enum HardAuthRule {
    ProofRule(HardProofRule),
    AnyOf(Vec<HardAuthRule>),
    AllOf(Vec<HardAuthRule>),
}

/// Authorization of a method call
#[derive(Debug, Clone, PartialEq, Eq, Hash, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub enum MethodAuthorization {
    Protected(HardAuthRule),
    AllowAll,
    DenyAll,
    Unsupported,
}
