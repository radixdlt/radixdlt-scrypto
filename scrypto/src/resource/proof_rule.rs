use crate::prelude::AuthRule::{AllOf, AnyOf};
use crate::resource::*;
use crate::rust::vec;
use crate::rust::vec::Vec;
use sbor::*;
use scrypto::math::Decimal;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Describe, TypeId, Encode, Decode)]
pub enum SoftResource {
    Static(ResourceDefId),
    Dynamic(SchemaPath),
}

impl From<ResourceDefId> for SoftResource {
    fn from(resource_def_id: ResourceDefId) -> Self {
        SoftResource::Static(resource_def_id)
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
    StaticResource(ResourceDefId),
    Dynamic(SchemaPath),
}

impl From<NonFungibleAddress> for SoftResourceOrNonFungible {
    fn from(non_fungible_address: NonFungibleAddress) -> Self {
        SoftResourceOrNonFungible::StaticNonFungible(non_fungible_address)
    }
}

impl From<ResourceDefId> for SoftResourceOrNonFungible {
    fn from(resource_def_id: ResourceDefId) -> Self {
        SoftResourceOrNonFungible::StaticResource(resource_def_id)
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
#[derive(Debug, Clone, PartialEq, Eq, Hash, Describe, TypeId, Encode, Decode)]
pub enum ProofRule {
    Require(SoftResourceOrNonFungible),
    AmountOf(Decimal, SoftResource),
    CountOf(u8, SoftResourceOrNonFungibleList),
    AllOf(SoftResourceOrNonFungibleList),
    AnyOf(SoftResourceOrNonFungibleList),
}

impl From<NonFungibleAddress> for ProofRule {
    fn from(non_fungible_address: NonFungibleAddress) -> Self {
        ProofRule::Require(non_fungible_address.into())
    }
}

impl From<ResourceDefId> for ProofRule {
    fn from(resource_def_id: ResourceDefId) -> Self {
        ProofRule::Require(resource_def_id.into())
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

#[derive(Debug, Clone, PartialEq, Eq, Hash, Describe, TypeId, Encode, Decode)]
pub enum AuthRule {
    ProofRule(ProofRule),
    AnyOf(Vec<AuthRule>),
    AllOf(Vec<AuthRule>),
}

impl AuthRule {
    pub fn or(self, other: AuthRule) -> Self {
        match self {
            AuthRule::AnyOf(mut rules) => {
                rules.push(other);
                AnyOf(rules)
            }
            _ => AnyOf(vec![self, other]),
        }
    }

    pub fn and(self, other: AuthRule) -> Self {
        match self {
            AuthRule::AllOf(mut rules) => {
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

pub fn require_n_of<T>(count: u8, resources: T) -> ProofRule
where
    T: Into<SoftResourceOrNonFungibleList>,
{
    ProofRule::CountOf(count, resources.into())
}

pub fn require_amount<T>(amount: Decimal, resource: T) -> ProofRule
where
    T: Into<SoftResource>,
{
    ProofRule::AmountOf(amount, resource.into())
}

// TODO: Move this logic into preprocessor. It probably needs to be implemented as a procedural macro.
#[macro_export]
macro_rules! auth_and_or {
    (|| $tt:tt) => {{
        let next = auth!($tt);
        move |e: AuthRule| e.or(next)
    }};
    (|| $right1:ident $right2:tt) => {{
        let next = auth!($right1 $right2);
        move |e: AuthRule| e.or(next)
    }};
    (|| $right:tt && $($rest:tt)+) => {{
        let f = auth_and_or!(&& $($rest)+);
        let next = auth!($right);
        move |e: AuthRule| e.or(f(next))
    }};
    (|| $right:tt || $($rest:tt)+) => {{
        let f = auth_and_or!(|| $($rest)+);
        let next = auth!($right);
        move |e: AuthRule| f(e.or(next))
    }};
    (|| $right1:ident $right2:tt && $($rest:tt)+) => {{
        let f = auth_and_or!(&& $($rest)+);
        let next = auth!($right1 $right2);
        move |e: AuthRule| e.or(f(next))
    }};
    (|| $right1:ident $right2:tt || $($rest:tt)+) => {{
        let f = auth_and_or!(|| $($rest)+);
        let next = auth!($right1 $right2);
        move |e: AuthRule| f(e.or(next))
    }};

    (&& $tt:tt) => {{
        let next = auth!($tt);
        move |e: AuthRule| e.and(next)
    }};
    (&& $right1:ident $right2:tt) => {{
        let next = auth!($right1 $right2);
        move |e: AuthRule| e.and(next)
    }};
    (&& $right:tt && $($rest:tt)+) => {{
        let f = auth_and_or!(&& $($rest)+);
        let next = auth!($right);
        move |e: AuthRule| f(e.and(next))
    }};
    (&& $right:tt || $($rest:tt)+) => {{
        let f = auth_and_or!(|| $($rest)+);
        let next = auth!($right);
        move |e: AuthRule| f(e.and(next))
    }};
    (&& $right1:ident $right2:tt && $($rest:tt)+) => {{
        let f = auth_and_or!(&& $($rest)+);
        let next = auth!($right1 $right2);
        move |e: AuthRule| f(e.and(next))
    }};
    (&& $right1:ident $right2:tt || $($rest:tt)+) => {{
        let f = auth_and_or!(|| $($rest)+);
        let next = auth!($right1 $right2);
        move |e: AuthRule| f(e.and(next))
    }};
}

#[macro_export]
macro_rules! auth {
    // Handle leaves
    ($rule:ident $args:tt) => {{ ::scrypto::resource::AuthRule::ProofRule($rule $args) }};

    // Handle group
    (($($tt:tt)+)) => {{ auth!($($tt)+) }};

    // Handle and/or logic
    ($left1:ident $left2:tt $($right:tt)+) => {{
        let f = auth_and_or!($($right)+);
        f(auth!($left1 $left2))
    }};
    ($left:tt $($right:tt)+) => {{
        let f = auth_and_or!($($right)+);
        f(auth!($left))
    }};

}
