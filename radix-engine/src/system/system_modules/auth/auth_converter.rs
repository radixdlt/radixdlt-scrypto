use super::authorization::{
    HardAuthRule, HardCount, HardDecimal, HardProofRule, HardProofRuleResourceList,
    HardResourceOrNonFungible, MethodAuthorization,
};
use crate::types::*;
use radix_engine_interface::blueprints::resource::*;

fn soft_to_hard_decimal(
    soft_decimal: &SoftDecimal,
) -> HardDecimal {
    match soft_decimal {
        SoftDecimal::Static(amount) => HardDecimal::Amount(amount.clone()),
    }
}

fn soft_to_hard_count(
    soft_count: &SoftCount,
) -> HardCount {
    match soft_count {
        SoftCount::Static(count) => HardCount::Count(count.clone()),
    }
}

fn soft_to_hard_resource_list(
    list: &SoftResourceOrNonFungibleList,
) -> HardProofRuleResourceList {
    match list {
        SoftResourceOrNonFungibleList::Static(resources) => {
            let mut hard_resources = Vec::new();
            for soft_resource in resources {
                let resource =
                    soft_to_hard_resource_or_non_fungible(soft_resource);
                hard_resources.push(resource);
            }
            HardProofRuleResourceList::List(hard_resources)
        }
    }
}

fn soft_to_hard_resource(
    soft_resource: &SoftResource,
) -> HardResourceOrNonFungible {
    match soft_resource {
        SoftResource::Static(resource) => HardResourceOrNonFungible::Resource(resource.clone()),
    }
}

fn soft_to_hard_resource_or_non_fungible(
    soft_resource_or_non_fungible: &SoftResourceOrNonFungible,
) -> HardResourceOrNonFungible {
    match soft_resource_or_non_fungible {
        SoftResourceOrNonFungible::StaticNonFungible(non_fungible_global_id) => {
            HardResourceOrNonFungible::NonFungible(non_fungible_global_id.clone())
        }
        SoftResourceOrNonFungible::StaticResource(resource_def_id) => {
            HardResourceOrNonFungible::Resource(resource_def_id.clone())
        }
    }
}

fn soft_to_hard_proof_rule(
    proof_rule: &ProofRule,
) -> HardProofRule {
    match proof_rule {
        ProofRule::Require(resource_or_non_fungible) => {
            let resource = soft_to_hard_resource_or_non_fungible(
                resource_or_non_fungible,
            );
            HardProofRule::Require(resource)
        }
        ProofRule::AmountOf(soft_decimal, resource) => {
            let resource = soft_to_hard_resource(resource);
            let hard_decimal = soft_to_hard_decimal(soft_decimal);
            HardProofRule::AmountOf(hard_decimal, resource)
        }
        ProofRule::AllOf(resources) => {
            let hard_resources = soft_to_hard_resource_list(resources);
            HardProofRule::AllOf(hard_resources)
        }
        ProofRule::AnyOf(resources) => {
            let hard_resources = soft_to_hard_resource_list(resources);
            HardProofRule::AnyOf(hard_resources)
        }
        ProofRule::CountOf(soft_count, resources) => {
            let hard_count = soft_to_hard_count(soft_count);
            let hard_resources = soft_to_hard_resource_list(resources);
            HardProofRule::CountOf(hard_count, hard_resources)
        }
    }
}

fn soft_to_hard_auth_rule(
    auth_rule: &AccessRuleNode,
) -> HardAuthRule {
    match auth_rule {
        AccessRuleNode::ProofRule(proof_rule) => HardAuthRule::ProofRule(soft_to_hard_proof_rule(
            proof_rule
        )),
        AccessRuleNode::AnyOf(rules) => {
            let hard_rules = rules
                .iter()
                .map(|r| soft_to_hard_auth_rule(r))
                .collect();
            HardAuthRule::AnyOf(hard_rules)
        }
        AccessRuleNode::AllOf(rules) => {
            let hard_rules = rules
                .iter()
                .map(|r| soft_to_hard_auth_rule(r))
                .collect();
            HardAuthRule::AllOf(hard_rules)
        }
    }
}

/// Converts an `AccessRule` into a `MethodAuthorization`, with the given context of
/// Scrypto value and schema.
///
/// This method assumes that the value matches with the schema.
pub fn convert(
    method_auth: &AccessRule,
) -> MethodAuthorization {
    match method_auth {
        AccessRule::Protected(auth_rule) => MethodAuthorization::Protected(soft_to_hard_auth_rule(
            auth_rule
        )),
        AccessRule::AllowAll => MethodAuthorization::AllowAll,
        AccessRule::DenyAll => MethodAuthorization::DenyAll,
    }
}
