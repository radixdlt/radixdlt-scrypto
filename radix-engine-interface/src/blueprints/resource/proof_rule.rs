use crate::blueprints::resource::AccessRuleNode::{AllOf, AnyOf};
use crate::blueprints::resource::*;
use crate::data::scrypto::SchemaPath;
use crate::math::Decimal;
use crate::*;
use radix_engine_common::types::*;
use sbor::rust::vec;
use sbor::rust::vec::Vec;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Ord, PartialOrd, ScryptoSbor, ManifestSbor)]
pub enum SoftDecimal {
    Static(Decimal),
    Dynamic(SchemaPath),
}

impl From<Decimal> for SoftDecimal {
    fn from(amount: Decimal) -> Self {
        SoftDecimal::Static(amount)
    }
}

impl From<SchemaPath> for SoftDecimal {
    fn from(path: SchemaPath) -> Self {
        SoftDecimal::Dynamic(path)
    }
}

impl From<&str> for SoftDecimal {
    fn from(path: &str) -> Self {
        let schema_path: SchemaPath = path.parse().expect("Could not decode path");
        SoftDecimal::Dynamic(schema_path)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Ord, PartialOrd, ScryptoSbor, ManifestSbor)]
pub enum SoftCount {
    Static(u8),
    Dynamic(SchemaPath),
}

impl From<u8> for SoftCount {
    fn from(count: u8) -> Self {
        SoftCount::Static(count)
    }
}

impl From<SchemaPath> for SoftCount {
    fn from(path: SchemaPath) -> Self {
        SoftCount::Dynamic(path)
    }
}

impl From<&str> for SoftCount {
    fn from(path: &str) -> Self {
        let schema_path: SchemaPath = path.parse().expect("Could not decode path");
        SoftCount::Dynamic(schema_path)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Ord, PartialOrd, ScryptoSbor, ManifestSbor)]
pub enum SoftResource {
    Static(ResourceAddress),
    Dynamic(SchemaPath),
}

impl From<ResourceAddress> for SoftResource {
    fn from(resource_address: ResourceAddress) -> Self {
        SoftResource::Static(resource_address)
    }
}

impl From<SchemaPath> for SoftResource {
    fn from(path: SchemaPath) -> Self {
        SoftResource::Dynamic(path)
    }
}

impl From<&str> for SoftResource {
    fn from(path: &str) -> Self {
        let schema_path: SchemaPath = path.parse().expect("Could not decode path");
        SoftResource::Dynamic(schema_path)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Ord, PartialOrd, ScryptoSbor, ManifestSbor)]
pub enum SoftResourceOrNonFungible {
    StaticNonFungible(NonFungibleGlobalId),
    StaticResource(ResourceAddress),
    Dynamic(SchemaPath),
}

impl From<NonFungibleGlobalId> for SoftResourceOrNonFungible {
    fn from(non_fungible_global_id: NonFungibleGlobalId) -> Self {
        SoftResourceOrNonFungible::StaticNonFungible(non_fungible_global_id)
    }
}

impl From<ResourceAddress> for SoftResourceOrNonFungible {
    fn from(resource_address: ResourceAddress) -> Self {
        SoftResourceOrNonFungible::StaticResource(resource_address)
    }
}

impl From<SchemaPath> for SoftResourceOrNonFungible {
    fn from(path: SchemaPath) -> Self {
        SoftResourceOrNonFungible::Dynamic(path)
    }
}

impl From<&str> for SoftResourceOrNonFungible {
    fn from(path: &str) -> Self {
        let schema_path: SchemaPath = path.parse().expect("Could not decode path");
        SoftResourceOrNonFungible::Dynamic(schema_path)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Ord, PartialOrd, ScryptoSbor, ManifestSbor)]
pub enum SoftResourceOrNonFungibleList {
    Static(Vec<SoftResourceOrNonFungible>),
    Dynamic(SchemaPath),
}

impl From<SchemaPath> for SoftResourceOrNonFungibleList {
    fn from(path: SchemaPath) -> Self {
        SoftResourceOrNonFungibleList::Dynamic(path)
    }
}

impl From<&str> for SoftResourceOrNonFungibleList {
    fn from(path: &str) -> Self {
        let schema_path: SchemaPath = path.parse().expect("Could not decode path");
        SoftResourceOrNonFungibleList::Dynamic(schema_path)
    }
}

impl<T> From<Vec<T>> for SoftResourceOrNonFungibleList
where
    T: Into<SoftResourceOrNonFungible>,
{
    fn from(addresses: Vec<T>) -> Self {
        SoftResourceOrNonFungibleList::Static(addresses.into_iter().map(|a| a.into()).collect())
    }
}

/// Resource Proof Rules
#[derive(Debug, Clone, PartialEq, Eq, Hash, Ord, PartialOrd, ScryptoSbor, ManifestSbor)]
pub enum ProofRule {
    Require(SoftResourceOrNonFungible),
    AmountOf(SoftDecimal, SoftResource),
    CountOf(SoftCount, SoftResourceOrNonFungibleList),
    AllOf(SoftResourceOrNonFungibleList),
    AnyOf(SoftResourceOrNonFungibleList),
}

impl From<ResourceAddress> for ProofRule {
    fn from(resource_address: ResourceAddress) -> Self {
        ProofRule::Require(resource_address.into())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Ord, PartialOrd, ScryptoSbor, ManifestSbor)]
pub enum AccessRuleNode {
    ProofRule(ProofRule),
    AnyOf(Vec<AccessRuleNode>),
    AllOf(Vec<AccessRuleNode>),
}

impl AccessRuleNode {
    pub fn or(self, other: AccessRuleNode) -> Self {
        match self {
            AccessRuleNode::AnyOf(mut rules) => {
                rules.push(other);
                AnyOf(rules)
            }
            _ => AnyOf(vec![self, other]),
        }
    }

    pub fn and(self, other: AccessRuleNode) -> Self {
        match self {
            AccessRuleNode::AllOf(mut rules) => {
                rules.push(other);
                AllOf(rules)
            }
            _ => AllOf(vec![self, other]),
        }
    }
}

pub fn require<T>(resource: T) -> ProofRule
where
    T: Into<SoftResourceOrNonFungible>,
{
    ProofRule::Require(resource.into())
}

pub fn require_any_of<T>(resources: T) -> ProofRule
where
    T: Into<SoftResourceOrNonFungibleList>,
{
    ProofRule::AnyOf(resources.into())
}

pub fn require_all_of<T>(resources: T) -> ProofRule
where
    T: Into<SoftResourceOrNonFungibleList>,
{
    ProofRule::AllOf(resources.into())
}

pub fn require_n_of<C, T>(count: C, resources: T) -> ProofRule
where
    C: Into<SoftCount>,
    T: Into<SoftResourceOrNonFungibleList>,
{
    ProofRule::CountOf(count.into(), resources.into())
}

pub fn require_amount<D, T>(amount: D, resource: T) -> ProofRule
where
    D: Into<SoftDecimal>,
    T: Into<SoftResource>,
{
    ProofRule::AmountOf(amount.into(), resource.into())
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Ord, PartialOrd, ScryptoSbor, ManifestSbor)]
pub enum AccessRule {
    AllowAll,
    DenyAll,
    Protected(AccessRuleNode),
}
