use crate::resource::proof_rule::SchemaSubPath::{Field, Index};
use crate::resource::*;
use crate::rust::string::String;
use crate::rust::string::ToString;
use crate::rust::vec;
use crate::rust::vec::Vec;
use sbor::describe::Fields;
use sbor::path::SborFullPath;
use sbor::*;
use scrypto::math::Decimal;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Describe, TypeId, Encode, Decode)]
pub enum SchemaSubPath {
    Index(usize),
    Field(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Describe, TypeId, Encode, Decode)]
pub struct SborPath(Vec<SchemaSubPath>);

impl SborPath {
    pub fn new() -> Self {
        SborPath(vec![])
    }

    pub fn field(mut self, field: &str) -> Self {
        self.0.push(Field(field.to_string()));
        self
    }

    pub fn index(mut self, index: usize) -> Self {
        self.0.push(Index(index));
        self
    }

    pub fn rel_path(&self, schema: &Type) -> Option<SborFullPath> {
        let length = self.0.len();
        let mut cur_type = schema;
        let mut sbor_path: Vec<usize> = vec![];

        for i in 0..length {
            match self.0.get(i).unwrap() {
                Index(index) => match cur_type {
                    Type::Vec { element } => {
                        cur_type = element.as_ref();
                        sbor_path.push(*index);
                    }
                    Type::Array { element, length: _ } => {
                        cur_type = element.as_ref();
                        sbor_path.push(*index);
                    }
                    _ => return Option::None,
                },
                Field(field) => {
                    if let Type::Struct { name: _, fields } = cur_type {
                        match fields {
                            Fields::Named { named } => {
                                if let Some(index) = named
                                    .iter()
                                    .position(|(field_name, _)| field_name.eq(field))
                                {
                                    let (_, next_type) = named.get(index).unwrap();
                                    cur_type = next_type;
                                    sbor_path.push(index);
                                } else {
                                    return Option::None;
                                }
                            }
                            _ => return Option::None,
                        }
                    } else {
                        return Option::None;
                    }
                }
            }
        }

        Option::Some(SborFullPath::new(sbor_path))
    }
}

impl From<&str> for SborPath {
    fn from(str: &str) -> Self {
        let path: Vec<SchemaSubPath> = str
            .split('.')
            .map(|s| match s.parse::<usize>() {
                Ok(usize) => Index(usize),
                Err(_) => Field(s.to_string()),
            })
            .collect();

        SborPath(path)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Describe, TypeId, Encode, Decode)]
pub enum ProofRuleResource {
    NonFungible(NonFungibleAddress),
    Resource(ResourceDefId),
    FromComponent(SborPath),
}

impl From<NonFungibleAddress> for ProofRuleResource {
    fn from(non_fungible_address: NonFungibleAddress) -> Self {
        ProofRuleResource::NonFungible(non_fungible_address)
    }
}

impl From<ResourceDefId> for ProofRuleResource {
    fn from(resource_def_id: ResourceDefId) -> Self {
        ProofRuleResource::Resource(resource_def_id)
    }
}

impl From<SborPath> for ProofRuleResource {
    fn from(path: SborPath) -> Self {
        ProofRuleResource::FromComponent(path)
    }
}

impl From<&str> for ProofRuleResource {
    fn from(path: &str) -> Self {
        ProofRuleResource::FromComponent(SborPath::from(path))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Describe, TypeId, Encode, Decode)]
pub enum ProofRuleResourceList {
    StaticList(Vec<ProofRuleResource>),
    FromComponent(SborPath),
}

impl From<SborPath> for ProofRuleResourceList {
    fn from(path: SborPath) -> Self {
        ProofRuleResourceList::FromComponent(path)
    }
}

/// Authorization Rule
#[derive(Debug, Clone, PartialEq, Eq, Hash, Describe, TypeId, Encode, Decode)]
pub enum ProofRule {
    This(ProofRuleResource),
    AmountOf(Decimal, ProofRuleResource),
    CountOf(u8, ProofRuleResourceList),
    AllOf(ProofRuleResourceList),
    AnyOf(ProofRuleResourceList),
}

impl From<NonFungibleAddress> for ProofRule {
    fn from(non_fungible_address: NonFungibleAddress) -> Self {
        ProofRule::This(non_fungible_address.into())
    }
}

impl From<ResourceDefId> for ProofRule {
    fn from(resource_def_id: ResourceDefId) -> Self {
        ProofRule::This(resource_def_id.into())
    }
}

#[macro_export]
macro_rules! resource_list {
  ($($resource: expr),*) => ({
      let mut list: Vec<::scrypto::resource::ProofRuleResource> = Vec::new();
      $(
        list.push($resource.into());
      )*
      ::scrypto::resource::ProofRuleResourceList::StaticList(list)
  });
}

#[macro_export]
macro_rules! this {
    ($resource:expr) => {{
        ::scrypto::resource::ProofRule::This($resource.into())
    }};
}

#[macro_export]
macro_rules! any_of {
    ($list:expr) => ({
        ::scrypto::resource::ProofRule::AnyOf($list.into())
    });
    ($left:expr, $($right:expr),+) => ({
        ::scrypto::resource::ProofRule::AnyOf(resource_list!($left, $($right),+))
    });
}

#[macro_export]
macro_rules! all_of {
    ($list:expr) => ({
        ::scrypto::resource::ProofRule::AllOf($list.into())
    });
    ($left:expr, $($right:expr),+) => ({
        ::scrypto::resource::ProofRule::AllOf(resource_list!($left, $($right),+))
    });
}

#[macro_export]
macro_rules! min_n_of {
    ($count:expr, $list:expr) => ({
        ::scrypto::resource::ProofRule::CountOf($count, $list.into())
    });
    ($count:expr, $left:expr, $($right:expr),+) => ({
        ::scrypto::resource::ProofRule::CountOf($count, resource_list!($left, $($right),+))
    });
}

#[macro_export]
macro_rules! min_amount_of {
    ($amount:expr, $resource:expr) => {
        ProofRule::AmountOf($amount, $resource.into())
    };
}
