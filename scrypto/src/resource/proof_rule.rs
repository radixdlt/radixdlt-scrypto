use crate::resource::*;
use crate::rust::vec;
use crate::rust::vec::Vec;
use sbor::*;
use scrypto::math::Decimal;

/// Authorization Rule
#[derive(Debug, Clone, PartialEq, Eq, Hash, Describe, TypeId, Encode, Decode)]
pub enum ProofRule {
    NonFungible(NonFungibleAddress),
    AnyOfResource(ResourceDefId),
    SomeOfResource(Decimal, ResourceDefId),
    AllOf(Vec<ProofRule>),
    OneOf(Vec<ProofRule>),
    CountOf { count: u8, rules: Vec<ProofRule> },
}

impl ProofRule {
    pub fn or(self, other: ProofRule) -> Self {
        match self {
            ProofRule::NonFungible(_) => ProofRule::OneOf(vec![self, other]),
            ProofRule::AnyOfResource(_) => ProofRule::OneOf(vec![self, other]),
            ProofRule::SomeOfResource(_, _) => ProofRule::OneOf(vec![self, other]),
            ProofRule::AllOf(_) => ProofRule::OneOf(vec![self, other]),
            ProofRule::OneOf(mut rules) => {
                rules.push(other);
                ProofRule::OneOf(rules)
            }
            ProofRule::CountOf { count: _, rules: _ } => ProofRule::OneOf(vec![self, other]),
        }
    }
}

impl From<NonFungibleAddress> for ProofRule {
    fn from(non_fungible_address: NonFungibleAddress) -> Self {
        ProofRule::NonFungible(non_fungible_address)
    }
}

impl From<ResourceDefId> for ProofRule {
    fn from(resource_def_id: ResourceDefId) -> Self {
        ProofRule::AnyOfResource(resource_def_id)
    }
}

#[macro_export]
macro_rules! any_of {
    ($rule:expr) => ({
        let auth: ProofRule = $rule.into();
        auth
    });
    ($left:expr, $($right:expr),+) => ({
        let auth: ProofRule = $left.into();
        auth.or(any_of!($($right),+))
    })
}

#[macro_export]
macro_rules! all_of {
    ($rule:expr) => ({
        let auth: ProofRule = $rule.into();
        auth
    });
    ($left:expr, $($right:expr),+) => (
        ProofRule::AllOf(vec![$left.into(), all_of!($($right),+)])
    );
}

#[macro_export]
macro_rules! min_n_of {
    ($count:expr, $rule:expr) => (
        ProofRule::CountOf {
            count: $count,
            rules: vec![$rule.into()]
        }
    );
    ($count:expr, $left:expr, $($right:expr),+) => ({
        let mut auth = min_n_of!($count, $($right),+);
        match auth {
            // TODO: retain original order
            ProofRule::CountOf { count, mut rules } => {
                rules.push($left.into());
                ProofRule::CountOf { count, rules }
            },
            _ => panic!("Should never get here.")
        }
    })
}
#[macro_export]
macro_rules! amount_of {
    ($amount:expr, $resource:expr) => {
        ProofRule::SomeOfResource($amount, $resource)
    };
}
