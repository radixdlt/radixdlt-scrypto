use crate::resource::AuthRuleNode::{AllOf, AnyOf};
use crate::resource::*;
use crate::rust::borrow::ToOwned;
use crate::rust::vec;
use crate::rust::vec::Vec;
use sbor::*;
use scrypto::math::Decimal;

/// TODO: add documentation for public types once they're stable.

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
#[derive(Debug, Clone, PartialEq, Eq, Hash, TypeId, Encode, Decode)]
pub enum ProofRule {
    Require(SoftResourceOrNonFungible),
    AmountOf(SoftDecimal, SoftResource),
    CountOf(SoftCount, SoftResourceOrNonFungibleList),
    AllOf(SoftResourceOrNonFungibleList),
    AnyOf(SoftResourceOrNonFungibleList),
}

// FIXME: describe types with cycles
impl Describe for ProofRule {
    fn describe() -> sbor::describe::Type {
        sbor::describe::Type::Custom {
            name: "ProofRule".to_owned(),
            generics: vec![],
        }
    }
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
pub enum AuthRuleNode {
    ProofRule(ProofRule),
    AnyOf(Vec<AuthRuleNode>),
    AllOf(Vec<AuthRuleNode>),
}

// FIXME: describe types with cycles
impl Describe for AuthRuleNode {
    fn describe() -> sbor::describe::Type {
        sbor::describe::Type::Custom {
            name: "AuthRuleNode".to_owned(),
            generics: vec![],
        }
    }
}

impl AuthRuleNode {
    pub fn or(self, other: AuthRuleNode) -> Self {
        match self {
            AuthRuleNode::AnyOf(mut rules) => {
                rules.push(other);
                AnyOf(rules)
            }
            _ => AnyOf(vec![self, other]),
        }
    }

    pub fn and(self, other: AuthRuleNode) -> Self {
        match self {
            AuthRuleNode::AllOf(mut rules) => {
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
macro_rules! auth_and_or {
    (|| $tt:tt) => {{
        let next = auth_rule_node!($tt);
        move |e: AuthRuleNode| e.or(next)
    }};
    (|| $right1:ident $right2:tt) => {{
        let next = auth_rule_node!($right1 $right2);
        move |e: AuthRuleNode| e.or(next)
    }};
    (|| $right:tt && $($rest:tt)+) => {{
        let f = auth_and_or!(&& $($rest)+);
        let next = auth_rule_node!($right);
        move |e: AuthRuleNode| e.or(f(next))
    }};
    (|| $right:tt || $($rest:tt)+) => {{
        let f = auth_and_or!(|| $($rest)+);
        let next = auth_rule_node!($right);
        move |e: AuthRuleNode| f(e.or(next))
    }};
    (|| $right1:ident $right2:tt && $($rest:tt)+) => {{
        let f = auth_and_or!(&& $($rest)+);
        let next = auth_rule_node!($right1 $right2);
        move |e: AuthRuleNode| e.or(f(next))
    }};
    (|| $right1:ident $right2:tt || $($rest:tt)+) => {{
        let f = auth_and_or!(|| $($rest)+);
        let next = auth_rule_node!($right1 $right2);
        move |e: AuthRuleNode| f(e.or(next))
    }};

    (&& $tt:tt) => {{
        let next = auth_rule_node!($tt);
        move |e: AuthRuleNode| e.and(next)
    }};
    (&& $right1:ident $right2:tt) => {{
        let next = auth_rule_node!($right1 $right2);
        move |e: AuthRuleNode| e.and(next)
    }};
    (&& $right:tt && $($rest:tt)+) => {{
        let f = auth_and_or!(&& $($rest)+);
        let next = auth_rule_node!($right);
        move |e: AuthRuleNode| f(e.and(next))
    }};
    (&& $right:tt || $($rest:tt)+) => {{
        let f = auth_and_or!(|| $($rest)+);
        let next = auth_rule_node!($right);
        move |e: AuthRuleNode| f(e.and(next))
    }};
    (&& $right1:ident $right2:tt && $($rest:tt)+) => {{
        let f = auth_and_or!(&& $($rest)+);
        let next = auth_rule_node!($right1 $right2);
        move |e: AuthRuleNode| f(e.and(next))
    }};
    (&& $right1:ident $right2:tt || $($rest:tt)+) => {{
        let f = auth_and_or!(|| $($rest)+);
        let next = auth_rule_node!($right1 $right2);
        move |e: AuthRuleNode| f(e.and(next))
    }};
}

#[macro_export]
macro_rules! auth_rule_node {
    // Handle leaves
    ($rule:ident $args:tt) => {{ ::scrypto::resource::AuthRuleNode::ProofRule($rule $args) }};

    // Handle group
    (($($tt:tt)+)) => {{ auth_rule_node!($($tt)+) }};

    // Handle and/or logic
    ($left1:ident $left2:tt $($right:tt)+) => {{
        let f = auth_and_or!($($right)+);
        f(auth_rule_node!($left1 $left2))
    }};
    ($left:tt $($right:tt)+) => {{
        let f = auth_and_or!($($right)+);
        f(auth_rule_node!($left))
    }};
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Describe, TypeId, Encode, Decode)]
pub enum MethodAuth {
    AllowAll,
    DenyAll,
    Protected(AuthRuleNode),
}

#[macro_export]
macro_rules! method_auth {
    (allow_all) => {{
        ::scrypto::resource::MethodAuth::AllowAll
    }};
    (deny_all) => {{
        ::scrypto::resource::MethodAuth::DenyAll
    }};
    ($($tt:tt)+) => {{
        ::scrypto::resource::MethodAuth::Protected(auth_rule_node!($($tt)+))
    }};
}
