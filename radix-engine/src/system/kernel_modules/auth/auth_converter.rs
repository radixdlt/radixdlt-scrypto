use super::method_authorization::{
    HardAuthRule, HardCount, HardDecimal, HardProofRule, HardProofRuleResourceList,
    HardResourceOrNonFungible, MethodAuthorization,
};
use crate::types::*;
use radix_engine_interface::{blueprints::resource::*, schema::BlueprintSchema};

// FIXME: support schema path!

#[allow(unused_variables)]
fn soft_to_hard_decimal(
    schema: &BlueprintSchema,
    soft_decimal: &SoftDecimal,
    value: &IndexedScryptoValue,
) -> HardDecimal {
    match soft_decimal {
        SoftDecimal::Static(amount) => HardDecimal::Amount(amount.clone()),
        SoftDecimal::Dynamic(schema_path) => HardDecimal::InvalidSchemaPath,
    }
}

#[allow(unused_variables)]
fn soft_to_hard_count(
    schema: &BlueprintSchema,
    soft_count: &SoftCount,
    value: &IndexedScryptoValue,
) -> HardCount {
    match soft_count {
        SoftCount::Static(count) => HardCount::Count(count.clone()),
        SoftCount::Dynamic(schema_path) => HardCount::InvalidSchemaPath,
    }
}

#[allow(unused_variables)]
fn soft_to_hard_resource_list(
    schema: &BlueprintSchema,
    list: &SoftResourceOrNonFungibleList,
    value: &IndexedScryptoValue,
) -> HardProofRuleResourceList {
    match list {
        SoftResourceOrNonFungibleList::Static(resources) => {
            let mut hard_resources = Vec::new();
            for soft_resource in resources {
                let resource = soft_to_hard_resource_or_non_fungible(schema, soft_resource, value);
                hard_resources.push(resource);
            }
            HardProofRuleResourceList::List(hard_resources)
        }
        SoftResourceOrNonFungibleList::Dynamic(schema_path) => {
            HardProofRuleResourceList::InvalidSchemaPath
        }
    }
}

#[allow(unused_variables)]
fn soft_to_hard_resource(
    schema: &BlueprintSchema,
    soft_resource: &SoftResource,
    value: &IndexedScryptoValue,
) -> HardResourceOrNonFungible {
    match soft_resource {
        SoftResource::Dynamic(schema_path) => HardResourceOrNonFungible::InvalidSchemaPath,
        SoftResource::Static(resource_def_id) => {
            HardResourceOrNonFungible::Resource(resource_def_id.clone())
        }
    }
}

#[allow(unused_variables)]
fn soft_to_hard_resource_or_non_fungible(
    schema: &BlueprintSchema,
    soft_resource_or_non_fungible: &SoftResourceOrNonFungible,
    value: &IndexedScryptoValue,
) -> HardResourceOrNonFungible {
    match soft_resource_or_non_fungible {
        SoftResourceOrNonFungible::Dynamic(schema_path) => {
            HardResourceOrNonFungible::InvalidSchemaPath
        }
        SoftResourceOrNonFungible::StaticNonFungible(non_fungible_global_id) => {
            HardResourceOrNonFungible::NonFungible(non_fungible_global_id.clone())
        }
        SoftResourceOrNonFungible::StaticResource(resource_def_id) => {
            HardResourceOrNonFungible::Resource(resource_def_id.clone())
        }
    }
}

#[allow(unused_variables)]
fn soft_to_hard_proof_rule(
    schema: &BlueprintSchema,
    proof_rule: &ProofRule,
    value: &IndexedScryptoValue,
) -> HardProofRule {
    match proof_rule {
        ProofRule::Require(soft_resource_or_non_fungible) => {
            let resource =
                soft_to_hard_resource_or_non_fungible(schema, soft_resource_or_non_fungible, value);
            HardProofRule::Require(resource)
        }
        ProofRule::AmountOf(soft_decimal, soft_resource) => {
            let resource = soft_to_hard_resource(schema, soft_resource, value);
            let hard_decimal = soft_to_hard_decimal(schema, soft_decimal, value);
            HardProofRule::AmountOf(hard_decimal, resource)
        }
        ProofRule::AllOf(resources) => {
            let hard_resources = soft_to_hard_resource_list(schema, resources, value);
            HardProofRule::AllOf(hard_resources)
        }
        ProofRule::AnyOf(resources) => {
            let hard_resources = soft_to_hard_resource_list(schema, resources, value);
            HardProofRule::AnyOf(hard_resources)
        }
        ProofRule::CountOf(soft_count, resources) => {
            let hard_count = soft_to_hard_count(schema, soft_count, value);
            let hard_resources = soft_to_hard_resource_list(schema, resources, value);
            HardProofRule::CountOf(hard_count, hard_resources)
        }
    }
}

#[allow(unused_variables)]
fn soft_to_hard_auth_rule(
    schema: &BlueprintSchema,
    auth_rule: &AccessRuleNode,
    value: &IndexedScryptoValue,
) -> HardAuthRule {
    match auth_rule {
        AccessRuleNode::ProofRule(proof_rule) => {
            HardAuthRule::ProofRule(soft_to_hard_proof_rule(schema, proof_rule, value))
        }
        AccessRuleNode::AnyOf(rules) => {
            let hard_rules = rules
                .iter()
                .map(|r| soft_to_hard_auth_rule(schema, r, value))
                .collect();
            HardAuthRule::AnyOf(hard_rules)
        }
        AccessRuleNode::AllOf(rules) => {
            let hard_rules = rules
                .iter()
                .map(|r| soft_to_hard_auth_rule(schema, r, value))
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
    schema: &BlueprintSchema,
    value: &IndexedScryptoValue,
    method_auth: &AccessRule,
) -> MethodAuthorization {
    match method_auth {
        AccessRule::Protected(auth_rule) => {
            MethodAuthorization::Protected(soft_to_hard_auth_rule(schema, auth_rule, value))
        }
        AccessRule::AllowAll => MethodAuthorization::AllowAll,
        AccessRule::DenyAll => MethodAuthorization::DenyAll,
    }
}

pub fn convert_contextless(method_auth: &AccessRule) -> MethodAuthorization {
    convert(
        &BlueprintSchema::default(),
        &IndexedScryptoValue::unit(),
        method_auth,
    )
}
