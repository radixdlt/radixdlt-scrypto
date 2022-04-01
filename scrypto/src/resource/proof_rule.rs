use crate::resource::*;
use crate::rust::vec;
use crate::rust::vec::Vec;
use sbor::*;
use scrypto::math::Decimal;
use crate::prelude::AuthRule::{AllOf, AnyOf};

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

#[macro_export]
macro_rules! require {
    ($resource:expr) => {{
        ::scrypto::resource::ProofRule::Require($resource.into())
    }};
}

#[macro_export]
macro_rules! require_any_of {
    ($list:expr) => {{
        ::scrypto::resource::ProofRule::AnyOf($list.into())
    }};
}

#[macro_export]
macro_rules! require_all_of {
    ($list:expr) => {{
        ::scrypto::resource::ProofRule::AllOf($list.into())
    }};
}

#[macro_export]
macro_rules! require_n_of {
    ($count:expr, $list:expr) => {{
        ::scrypto::resource::ProofRule::CountOf($count, $list.into())
    }};
}

#[macro_export]
macro_rules! require_amount {
    ($amount:expr, $resource:expr) => {
        ProofRule::AmountOf($amount, $resource.into())
    };
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
            },
            _ => AnyOf(vec![self, other]),
        }
    }

    pub fn and(self, other: AuthRule) -> Self {
        match self {
            AuthRule::AllOf(mut rules) => {
                rules.push(other);
                AllOf(rules)
            },
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

#[macro_export]
macro_rules! auth2 {
    (require($rule:expr)) => {{
        ::scrypto::resource::AuthRule::ProofRule(::scrypto::resource::ProofRule::Require($rule.into()))
    }};
    (require_any_of($list:expr)) => {{
        ::scrypto::resource::AuthRule::ProofRule(::scrypto::resource::ProofRule::AnyOf($list.into()))
    }};
    (require_all_of($list:expr)) => {{
        ::scrypto::resource::AuthRule::ProofRule(::scrypto::resource::ProofRule::AllOf($list.into()))
    }};
    (require_n_of($count:expr, $list:expr)) => {{
        ::scrypto::resource::AuthRule::ProofRule(::scrypto::resource::ProofRule::CountOf($count, $list.into()))
    }};
    (require_amount($amount:expr, $resource:expr)) => {{
        ::scrypto::resource::AuthRule::ProofRule(::scrypto::resource::ProofRule::AmountOf($amount, $resource.into()))
    }};
    ($($tt1:tt$tt2:tt)||+) => {{
        let mut rule = ::scrypto::resource::AuthRule::ProofRule(::scrypto::resource::ProofRule::AnyOf(SoftResourceOrNonFungibleList::Static(vec![])));
        $(
            rule = rule.or(auth2!($tt1$tt2));
        )*
        rule
    }};
    (($($tt1:tt$tt2:tt)||+)) => {{
        let mut rule = ::scrypto::resource::AuthRule::ProofRule(::scrypto::resource::ProofRule::AnyOf(SoftResourceOrNonFungibleList::Static(vec![])));
        $(
            rule = rule.or(auth2!($tt1$tt2));
        )*
        rule
    }};
    ($tt1:tt || $tt2:tt$rule2:tt) => {{
        let rule = auth2!($tt1);
        rule.or(auth2!($tt2$rule2))
    }};
    (($tt1:tt$rule1:tt || $tt2:tt$rule2:tt)) => {{
        let rule = auth2!($tt1$rule1);
        rule.or(auth2!($tt2$rule2))
    }};
    ($tt1:tt$rule1:tt || $tt2:tt$rule2:tt) => {{
        let rule = auth2!($tt1$rule1);
        rule.or(auth2!($tt2$rule2))
    }};
}