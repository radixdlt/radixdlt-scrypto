use crate::resource::*;
use crate::rust::vec::Vec;
use crate::rust::vec;
use sbor::*;
use scrypto::math::Decimal;

/// Authorization Rule
#[derive(Debug, Clone, Describe, TypeId, Encode, Decode)]
pub enum AuthRule {
    NonFungible(NonFungibleAddress),
    AnyOfResource(ResourceDefId),
    SomeOfResource(Decimal, ResourceDefId),
    AllOf(Vec<AuthRule>),
    OneOf(Vec<AuthRule>),
    CountOf { count: u8, rules: Vec<AuthRule> },
}

impl AuthRule {
    pub fn or(self, other: AuthRule) -> Self {
        match self {
            AuthRule::NonFungible(_) => AuthRule::OneOf(vec![self, other]),
            AuthRule::AnyOfResource(_) => AuthRule::OneOf(vec![self, other]),
            AuthRule::SomeOfResource(_, _) => AuthRule::OneOf(vec![self, other]),
            AuthRule::AllOf(_) => AuthRule::OneOf(vec![self, other]),
            AuthRule::OneOf(mut rules) => {
                rules.push(other);
                AuthRule::OneOf(rules)
            }
            AuthRule::CountOf { count: _, rules: _ } => AuthRule::OneOf(vec![self, other]),
        }
    }
}

impl From<NonFungibleAddress> for AuthRule {
    fn from(non_fungible_address: NonFungibleAddress) -> Self {
        AuthRule::NonFungible(non_fungible_address)
    }
}

impl From<ResourceDefId> for AuthRule {
    fn from(resource_def_id: ResourceDefId) -> Self {
        AuthRule::AnyOfResource(resource_def_id)
    }
}

#[macro_export]
macro_rules! any_of {
    ($rule:expr) => ($rule.into());
    ($left:expr, $($right:expr),+) => ({
        let auth: AuthRule = $left.into();
        auth.or(any_of!($($right),+))
    })
}

#[macro_export]
macro_rules! all_of {
    ($rule:expr) => ($rule.into());
    ($left:expr, $($right:expr),+) => (
        AuthRule::AllOf(vec![$left.into(), all_of!($($right),+)])
    );
}

#[macro_export]
macro_rules! min_n_of {
    ($count:expr, $rule:expr) => (
        AuthRule::CountOf {
            count: $count,
            rules: vec![$rule.into()]
        }
    );
    ($count:expr, $left:expr, $($right:expr),+) => ({
        let mut auth = min_n_of!($count, $($right),+);
        match auth {
            AuthRule::CountOf { count, mut rules } => {
                rules.push($left.into());
                AuthRule::CountOf { count, rules }
            },
            _ => panic!("Should never get here.")
        }
    })
}
#[macro_export]
macro_rules! amount_of {
    ($amount:expr, $resource:expr) => (
        AuthRule::SomeOfResource($amount, $resource)
    );
}
