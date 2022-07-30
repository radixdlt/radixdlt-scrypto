use sbor::describe::{Fields, Variant};
use sbor::rust::borrow::ToOwned;
use sbor::rust::boxed::Box;
use sbor::rust::string::ToString;
use sbor::rust::vec;
use sbor::rust::vec::Vec;
use sbor::*;
use scrypto::math::Decimal;

use crate::engine::api::RadixEngineInput;
use crate::engine::call_engine;
use crate::engine::types::ProofId;
use crate::resource::AccessRuleNode::{AllOf, AnyOf};
use crate::resource::*;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Describe, TypeId, Encode, Decode)]
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

#[derive(Debug, Clone, PartialEq, Eq, Hash, Describe, TypeId, Encode, Decode)]
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

#[derive(Debug, Clone, PartialEq, Eq, Hash, Describe, TypeId, Encode, Decode)]
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

#[derive(Debug, Clone, PartialEq, Eq, Hash, Describe, TypeId, Encode, Decode)]
pub enum SoftResourceOrNonFungible {
    StaticNonFungible(NonFungibleAddress),
    StaticResource(ResourceAddress),
    Dynamic(SchemaPath),
}

impl From<NonFungibleAddress> for SoftResourceOrNonFungible {
    fn from(non_fungible_address: NonFungibleAddress) -> Self {
        SoftResourceOrNonFungible::StaticNonFungible(non_fungible_address)
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

#[derive(Debug, Clone, PartialEq, Eq, Hash, Describe, TypeId, Encode, Decode)]
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
#[derive(Debug, Clone, PartialEq, Eq, Hash, TypeId, Encode, Decode, Describe)]
pub enum ProofRule {
    Require(SoftResourceOrNonFungible),
    AmountOf(SoftDecimal, SoftResource),
    CountOf(SoftCount, SoftResourceOrNonFungibleList),
    AllOf(SoftResourceOrNonFungibleList),
    AnyOf(SoftResourceOrNonFungibleList),
}

impl From<NonFungibleAddress> for ProofRule {
    fn from(non_fungible_address: NonFungibleAddress) -> Self {
        ProofRule::Require(non_fungible_address.into())
    }
}

impl From<ResourceAddress> for ProofRule {
    fn from(resource_address: ResourceAddress) -> Self {
        ProofRule::Require(resource_address.into())
    }
}

#[macro_export]
macro_rules! resource_list {
  ($($resource: expr),*) => ({
      let mut list: Vec<::scrypto::resource::SoftResourceOrNonFungible> = Vec::new();
      $(
        list.push($resource.into());
      )*
      ::scrypto::resource::SoftResourceOrNonFungibleList::Static(list)
  });
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, TypeId, Encode, Decode)]
pub enum AccessRuleNode {
    ProofRule(ProofRule),
    AnyOf(Vec<AccessRuleNode>),
    AllOf(Vec<AccessRuleNode>),
}

// FIXME: describe types with cycles
impl Describe for AccessRuleNode {
    fn describe() -> sbor::describe::Type {
        sbor::describe::Type::Enum {
            name: "AccessRuleNode".to_owned(),
            variants: vec![
                Variant {
                    name: "ProofRule".to_string(),
                    fields: Fields::Unnamed {
                        unnamed: vec![Type::Any],
                    },
                },
                Variant {
                    name: "AnyOf".to_string(),
                    fields: Fields::Unnamed {
                        unnamed: vec![Type::Vec {
                            element: Box::new(Type::Any),
                        }],
                    },
                },
                Variant {
                    name: "AllOf".to_string(),
                    fields: Fields::Unnamed {
                        unnamed: vec![Type::Vec {
                            element: Box::new(Type::Any),
                        }],
                    },
                },
            ],
        }
    }
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

// TODO: Move this logic into preprocessor. It probably needs to be implemented as a procedural macro.
#[macro_export]
macro_rules! access_and_or {
    (|| $tt:tt) => {{
        let next = access_rule_node!($tt);
        move |e: AccessRuleNode| e.or(next)
    }};
    (|| $right1:ident $right2:tt) => {{
        let next = access_rule_node!($right1 $right2);
        move |e: AccessRuleNode| e.or(next)
    }};
    (|| $right:tt && $($rest:tt)+) => {{
        let f = access_and_or!(&& $($rest)+);
        let next = access_rule_node!($right);
        move |e: AccessRuleNode| e.or(f(next))
    }};
    (|| $right:tt || $($rest:tt)+) => {{
        let f = access_and_or!(|| $($rest)+);
        let next = access_rule_node!($right);
        move |e: AccessRuleNode| f(e.or(next))
    }};
    (|| $right1:ident $right2:tt && $($rest:tt)+) => {{
        let f = access_and_or!(&& $($rest)+);
        let next = access_rule_node!($right1 $right2);
        move |e: AccessRuleNode| e.or(f(next))
    }};
    (|| $right1:ident $right2:tt || $($rest:tt)+) => {{
        let f = access_and_or!(|| $($rest)+);
        let next = access_rule_node!($right1 $right2);
        move |e: AccessRuleNode| f(e.or(next))
    }};

    (&& $tt:tt) => {{
        let next = access_rule_node!($tt);
        move |e: AccessRuleNode| e.and(next)
    }};
    (&& $right1:ident $right2:tt) => {{
        let next = access_rule_node!($right1 $right2);
        move |e: AccessRuleNode| e.and(next)
    }};
    (&& $right:tt && $($rest:tt)+) => {{
        let f = access_and_or!(&& $($rest)+);
        let next = access_rule_node!($right);
        move |e: AccessRuleNode| f(e.and(next))
    }};
    (&& $right:tt || $($rest:tt)+) => {{
        let f = access_and_or!(|| $($rest)+);
        let next = access_rule_node!($right);
        move |e: AccessRuleNode| f(e.and(next))
    }};
    (&& $right1:ident $right2:tt && $($rest:tt)+) => {{
        let f = access_and_or!(&& $($rest)+);
        let next = access_rule_node!($right1 $right2);
        move |e: AccessRuleNode| f(e.and(next))
    }};
    (&& $right1:ident $right2:tt || $($rest:tt)+) => {{
        let f = access_and_or!(|| $($rest)+);
        let next = access_rule_node!($right1 $right2);
        move |e: AccessRuleNode| f(e.and(next))
    }};
}

#[macro_export]
macro_rules! access_rule_node {
    // Handle leaves
    ($rule:ident $args:tt) => {{ ::scrypto::resource::AccessRuleNode::ProofRule($rule $args) }};

    // Handle group
    (($($tt:tt)+)) => {{ access_rule_node!($($tt)+) }};

    // Handle and/or logic
    ($left1:ident $left2:tt $($right:tt)+) => {{
        let f = access_and_or!($($right)+);
        f(access_rule_node!($left1 $left2))
    }};
    ($left:tt $($right:tt)+) => {{
        let f = access_and_or!($($right)+);
        f(access_rule_node!($left))
    }};
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Describe, TypeId, Encode, Decode)]
pub enum AccessRule {
    AllowAll,
    DenyAll,
    Protected(AccessRuleNode),
}

impl AccessRule {
    pub fn check<'p, P: Into<AccessRuleCheckInput<'p>>>(&self, proofs: P) -> bool {
        let access_rule_check_input: AccessRuleCheckInput = proofs.into();
        let input =
            RadixEngineInput::CheckAccessRule(self.clone(), access_rule_check_input.proof_ids());
        let output: bool = call_engine(input);

        output
    }
}

#[macro_export]
macro_rules! rule {
    (allow_all) => {{
        ::scrypto::resource::AccessRule::AllowAll
    }};
    (deny_all) => {{
        ::scrypto::resource::AccessRule::DenyAll
    }};
    ($($tt:tt)+) => {{
        ::scrypto::resource::AccessRule::Protected(access_rule_node!($($tt)+))
    }};
}

pub enum AccessRuleCheckInput<'p> {
    Proofs(&'p [Proof]),
    ValidatedProofs(&'p [ValidatedProof]),
}

impl<'p> AccessRuleCheckInput<'p> {
    pub(crate) fn proof_ids(&self) -> Vec<ProofId> {
        match self {
            Self::Proofs(proofs) => proofs.iter().map(|proof| proof.0).collect(),
            Self::ValidatedProofs(proofs) => proofs.iter().map(|proof| proof.0.0).collect(),
        }
    }
}

impl<'p> From<&'p [Proof]> for AccessRuleCheckInput<'p> {
    fn from(proofs: &'p [Proof]) -> Self {
        Self::Proofs(proofs)
    }
}

impl<'a> From<&'a [ValidatedProof]> for AccessRuleCheckInput<'a> {
    fn from(proofs: &'a [ValidatedProof]) -> Self {
        Self::ValidatedProofs(proofs)
    }
}
