use crate::model::method_authorization::{
    HardAuthRule, HardCount, HardDecimal, HardProofRule, HardProofRuleResourceList,
    HardResourceOrNonFungible,
};
use crate::model::MethodAuthorization;
use crate::types::*;
use radix_engine_interface::data::IndexedScryptoValue;
use radix_engine_interface::model::*;

fn soft_to_hard_decimal(
    schema: &Type,
    soft_decimal: &SoftDecimal,
    value: &IndexedScryptoValue,
) -> HardDecimal {
    match soft_decimal {
        SoftDecimal::Static(amount) => HardDecimal::Amount(amount.clone()),
        SoftDecimal::Dynamic(schema_path) => {
            if let Some((sbor_path, ty)) = schema_path.to_sbor_path(schema) {
                match &ty {
                    Type::Decimal => {
                        let v = sbor_path
                            .get_from_value(value.as_value())
                            .expect(format!("Value missing at {:?}", schema_path).as_str());

                        HardDecimal::Amount(
                            scrypto_decode(&scrypto_encode(v).unwrap()).expect(
                                format!("Unexpected value type at {:?}", schema_path).as_str(),
                            ),
                        )
                    }
                    _ => HardDecimal::DisallowdValueType,
                }
            } else {
                HardDecimal::InvalidSchemaPath
            }
        }
    }
}

fn soft_to_hard_count(
    schema: &Type,
    soft_count: &SoftCount,
    value: &IndexedScryptoValue,
) -> HardCount {
    match soft_count {
        SoftCount::Static(count) => HardCount::Count(count.clone()),
        SoftCount::Dynamic(schema_path) => {
            if let Some((sbor_path, ty)) = schema_path.to_sbor_path(schema) {
                match &ty {
                    Type::U8 => {
                        let v = sbor_path
                            .get_from_value(value.as_value())
                            .expect(format!("Value missing at {:?}", schema_path).as_str());

                        HardCount::Count(
                            scrypto_decode(&scrypto_encode(v).unwrap()).expect(
                                format!("Unexpected value type at {:?}", schema_path).as_str(),
                            ),
                        )
                    }
                    _ => HardCount::DisallowdValueType,
                }
            } else {
                HardCount::InvalidSchemaPath
            }
        }
    }
}

fn soft_to_hard_resource_list(
    schema: &Type,
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
            if let Some((sbor_path, ty)) = schema_path.to_sbor_path(schema) {
                match &ty {
                    Type::Array { element_type, .. } | Type::Vec { element_type }
                        if matches!(element_type.as_ref(), Type::ResourceAddress) =>
                    {
                        let v = sbor_path
                            .get_from_value(value.as_value())
                            .expect(format!("Value missing at {:?}", schema_path).as_str());

                        HardProofRuleResourceList::List(
                            scrypto_decode::<Vec<ResourceAddress>>(&scrypto_encode(v).unwrap())
                                .expect(
                                    format!("Unexpected value type at {:?}", schema_path).as_str(),
                                )
                                .into_iter()
                                .map(|e| HardResourceOrNonFungible::Resource(e))
                                .collect(),
                        )
                    }
                    Type::Array { element_type, .. } | Type::Vec { element_type }
                        if matches!(element_type.as_ref(), Type::NonFungibleGlobalId) =>
                    {
                        let v = sbor_path
                            .get_from_value(value.as_value())
                            .expect(format!("Value missing at {:?}", schema_path).as_str());

                        HardProofRuleResourceList::List(
                            scrypto_decode::<Vec<NonFungibleGlobalId>>(&scrypto_encode(v).unwrap())
                                .expect(
                                    format!("Unexpected value type at {:?}", schema_path).as_str(),
                                )
                                .into_iter()
                                .map(|e| HardResourceOrNonFungible::NonFungible(e))
                                .collect(),
                        )
                    }
                    _ => HardProofRuleResourceList::DisallowdValueType,
                }
            } else {
                HardProofRuleResourceList::InvalidSchemaPath
            }
        }
    }
}

fn soft_to_hard_resource(
    schema: &Type,
    soft_resource: &SoftResource,
    value: &IndexedScryptoValue,
) -> HardResourceOrNonFungible {
    match soft_resource {
        SoftResource::Dynamic(schema_path) => {
            if let Some((sbor_path, ty)) = schema_path.to_sbor_path(schema) {
                match &ty {
                    Type::ResourceAddress => {
                        let v = sbor_path
                            .get_from_value(value.as_value())
                            .expect(format!("Value missing at {:?}", schema_path).as_str());

                        HardResourceOrNonFungible::Resource(
                            scrypto_decode(&scrypto_encode(v).unwrap()).expect(
                                format!("Unexpected value type at {:?}", schema_path).as_str(),
                            ),
                        )
                    }
                    _ => HardResourceOrNonFungible::DisallowdValueType,
                }
            } else {
                HardResourceOrNonFungible::InvalidSchemaPath
            }
        }
        SoftResource::Static(resource_def_id) => {
            HardResourceOrNonFungible::Resource(resource_def_id.clone())
        }
    }
}

fn soft_to_hard_resource_or_non_fungible(
    schema: &Type,
    soft_resource_or_non_fungible: &SoftResourceOrNonFungible,
    value: &IndexedScryptoValue,
) -> HardResourceOrNonFungible {
    match soft_resource_or_non_fungible {
        SoftResourceOrNonFungible::Dynamic(schema_path) => {
            if let Some((sbor_path, ty)) = schema_path.to_sbor_path(schema) {
                match &ty {
                    Type::ResourceAddress => {
                        let v = sbor_path
                            .get_from_value(value.as_value())
                            .expect(format!("Value missing at {:?}", schema_path).as_str());

                        HardResourceOrNonFungible::Resource(
                            scrypto_decode(&scrypto_encode(v).unwrap()).expect(
                                format!("Unexpected value type at {:?}", schema_path).as_str(),
                            ),
                        )
                    }
                    Type::NonFungibleGlobalId => {
                        let v = sbor_path
                            .get_from_value(value.as_value())
                            .expect(format!("Value missing at {:?}", schema_path).as_str());

                        HardResourceOrNonFungible::NonFungible(
                            scrypto_decode(&scrypto_encode(v).unwrap()).expect(
                                format!("Unexpected value type at {:?}", schema_path).as_str(),
                            ),
                        )
                    }
                    _ => HardResourceOrNonFungible::DisallowdValueType,
                }
            } else {
                HardResourceOrNonFungible::InvalidSchemaPath
            }
        }
        SoftResourceOrNonFungible::StaticNonFungible(non_fungible_global_id) => {
            HardResourceOrNonFungible::NonFungible(non_fungible_global_id.clone())
        }
        SoftResourceOrNonFungible::StaticResource(resource_def_id) => {
            HardResourceOrNonFungible::Resource(resource_def_id.clone())
        }
    }
}

fn soft_to_hard_proof_rule(
    schema: &Type,
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

fn soft_to_hard_auth_rule(
    schema: &Type,
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
    schema: &Type,
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
        &Type::Tuple {
            element_types: Vec::new(),
        },
        &IndexedScryptoValue::unit(),
        method_auth,
    )
}
