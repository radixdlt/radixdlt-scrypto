use crate::blueprints::resource::AccessRuleNode::{AllOf, AnyOf};
use crate::blueprints::resource::*;
use crate::math::Decimal;
use crate::*;
use radix_engine_common::types::*;
use sbor::rust::vec;
use sbor::rust::vec::Vec;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Ord, PartialOrd, ScryptoSbor, ManifestSbor)]
pub enum SoftResourceOrNonFungible {
    StaticNonFungible(NonFungibleGlobalId),
    StaticResource(ResourceAddress),
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

#[derive(Debug, Clone, PartialEq, Eq, Hash, Ord, PartialOrd, ScryptoSbor, ManifestSbor)]
pub enum SoftResourceOrNonFungibleList {
    Static(Vec<SoftResourceOrNonFungible>),
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
    AmountOf(Decimal, ResourceAddress),
    CountOf(u8, SoftResourceOrNonFungibleList),
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
    C: Into<u8>,
    T: Into<SoftResourceOrNonFungibleList>,
{
    ProofRule::CountOf(count.into(), resources.into())
}

pub fn require_amount<D, T>(amount: D, resource: T) -> ProofRule
where
    D: Into<Decimal>,
    T: Into<ResourceAddress>,
{
    ProofRule::AmountOf(amount.into(), resource.into())
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Ord, PartialOrd, ScryptoSbor, ManifestSbor)]
pub enum AccessRule {
    AllowAll,
    DenyAll,
    Protected(AccessRuleNode),
}
