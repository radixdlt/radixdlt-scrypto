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

/// Authorization Rule
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
    ($list:expr) => ({
        ::scrypto::resource::ProofRule::AnyOf($list.into())
    });
    ($left:expr, $($right:expr),+) => ({
        ::scrypto::resource::ProofRule::AnyOf(resource_list!($left, $($right),+))
    });
}

#[macro_export]
macro_rules! require_all_of {
    ($list:expr) => ({
        ::scrypto::resource::ProofRule::AllOf($list.into())
    });
    ($left:expr, $($right:expr),+) => ({
        ::scrypto::resource::ProofRule::AllOf(resource_list!($left, $($right),+))
    });
}

#[macro_export]
macro_rules! require_n_of {
    ($count:expr, $list:expr) => ({
        ::scrypto::resource::ProofRule::CountOf($count, $list.into())
    });
    ($count:expr, $left:expr, $($right:expr),+) => ({
        ::scrypto::resource::ProofRule::CountOf($count, resource_list!($left, $($right),+))
    });
}

#[macro_export]
macro_rules! require_amount {
    ($amount:expr, $resource:expr) => {
        ProofRule::AmountOf($amount, $resource.into())
    };
}
