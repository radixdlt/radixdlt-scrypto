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

#[macro_export]
macro_rules! auth {
    ($rule:expr) => {{
        ::scrypto::resource::AuthRule::ProofRule($rule)
    }};
}

pub fn require<T>(resource: T) -> ProofRule where T: Into<SoftResourceOrNonFungible> {
    ProofRule::Require(resource.into())
}

pub fn require_any_of<T>(resources: T) -> ProofRule where T: Into<SoftResourceOrNonFungibleList> {
    ProofRule::AnyOf(resources.into())
}

pub fn require_all_of<T>(resources: T) -> ProofRule where T: Into<SoftResourceOrNonFungibleList> {
    ProofRule::AllOf(resources.into())
}

pub fn require_n_of<T>(count: u8, resources: T) -> ProofRule where T: Into<SoftResourceOrNonFungibleList> {
    ProofRule::CountOf(count, resources.into())
}

pub fn require_amount<T>(amount: Decimal, resource: T) -> ProofRule where T: Into<SoftResource> {
    ProofRule::AmountOf(amount, resource.into())
}

#[macro_export]
macro_rules! auth2 {
    ($rule:ident $args:tt) => {{
        ::scrypto::resource::AuthRule::ProofRule($rule $args)
    }};
    (($($tt:tt)+)) => {{
        auth2!($($tt)+)
    }};
    ($left:tt || $($right:tt)+) => {{ auth2!($left).or(auth2!($($right)+)) }};
    ($left_rule:ident $left:tt || $($right:tt)+) => {{ auth2!($left_rule $left).or(auth2!($($right)+)) }};

    ($left:tt && $right:tt) => {{ auth2!($left).and(auth2!($right)) }};
    ($left:tt && $right:tt && $($rest:tt)+) => {{ auth2!($left && $right).and(auth2!($($rest)+)) }};
    ($left:tt && $right:tt || $($rest:tt)+) => {{ auth2!($left && $right).or(auth2!($($rest)+)) }};

    ($left1:ident $left2:tt && $right:tt) => {{ auth2!($left1$left2).and(auth2!($right)) }};
    ($left1:ident $left2:tt && $right:tt && $($rest:tt)+) => {{ auth2!($left1$left2 && $right).and(auth2!($($rest)+)) }};
    ($left1:ident $left2:tt && $right:tt || $($rest:tt)+) => {{ auth2!($left1$left2 && $right).or(auth2!($($rest)+)) }};

    ($left:tt && $right1:ident $right2:tt) => {{ auth2!($left).and(auth2!($right1$right2)) }};
    ($left:tt && $right1:ident $right2:tt && $($rest:tt)+) => {{ auth2!($left && $right1$right2).and(auth2!($($rest)+)) }};
    ($left:tt && $right1:ident $right2:tt || $($rest:tt)+) => {{ auth2!($left && $right1$right2).or(auth2!($($rest)+)) }};

    ($left1:ident $left2:tt && $right1:ident $right2:tt) => {{ auth2!($left1$left2).and(auth2!($right1$right2)) }};
    ($left1:ident $left2:tt && $right1:ident $right2:tt && $($rest:tt)+) => {{ auth2!($left1$left2 && $right1$right2).and(auth2!($($rest)+)) }};
    ($left1:ident $left2:tt && $right1:ident $right2:tt || $($rest:tt)+) => {{ auth2!($left1$left2 && $right1$right2).or(auth2!($($rest)+)) }};
}
