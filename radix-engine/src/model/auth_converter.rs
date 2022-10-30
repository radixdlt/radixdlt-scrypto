use crate::model::method_authorization::{
    HardAuthRule, HardCount, HardDecimal, HardProofRule, HardProofRuleResourceList,
    HardResourceOrNonFungible,
};
use crate::model::MethodAuthorization;
use crate::types::*;

fn soft_to_hard_decimal(
    schema: &Type,
    soft_decimal: &SoftDecimal,
    value: &ScryptoValue,
) -> HardDecimal {
    match soft_decimal {
        SoftDecimal::Static(amount) => HardDecimal::Amount(amount.clone()),
        SoftDecimal::Dynamic(schema_path) => {
            if let Some(sbor_path) = schema_path.to_sbor_path(schema) {
                match sbor_path.get_from_value(&value.dom) {
                    Some(Value::Custom { type_id, bytes }) => {
                        match ScryptoType::from_id(*type_id)
                            .expect("ScryptoValue contains invalid SBOR type ID")
                        {
                            ScryptoType::Decimal => {
                                HardDecimal::Amount(Decimal::try_from(bytes.as_slice()).expect(
                                    "ScryptoValue contains mismatching SBOR type ID and value",
                                ))
                            }
                            _ => HardDecimal::SoftDecimalNotFound,
                        }
                    }
                    _ => HardDecimal::SoftDecimalNotFound,
                }
            } else {
                return HardDecimal::SoftDecimalNotFound;
            }
        }
    }
}

fn soft_to_hard_count(schema: &Type, soft_count: &SoftCount, value: &ScryptoValue) -> HardCount {
    match soft_count {
        SoftCount::Static(count) => HardCount::Count(count.clone()),
        SoftCount::Dynamic(schema_path) => {
            if let Some(sbor_path) = schema_path.to_sbor_path(schema) {
                match sbor_path.get_from_value(&value.dom) {
                    Some(Value::U8 { value }) => HardCount::Count(value.clone()),
                    _ => HardCount::SoftCountNotFound,
                }
            } else {
                return HardCount::SoftCountNotFound;
            }
        }
    }
}

fn soft_to_hard_resource_list(
    schema: &Type,
    list: &SoftResourceOrNonFungibleList,
    value: &ScryptoValue,
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
            if let Some(sbor_path) = schema_path.to_sbor_path(schema) {
                match sbor_path.get_from_value(&value.dom) {
                    Some(Value::List {
                        element_type_id,
                        elements,
                    }) => match ScryptoType::from_id(*element_type_id).expect("ScryptoValue contains invalid SBOR type ID") {
                        ScryptoType::ResourceAddress => HardProofRuleResourceList::List(
                            elements
                                .iter()
                                .map(|v| {
                                    if let Value::Custom { bytes, .. } = v {
                                        return ResourceAddress::try_from(bytes.as_slice())
                                            .expect("ScryptoValue contains mismatching SBOR type ID and value")
                                            .into();
                                    }
                                    panic!("Unexpected type");
                                })
                                .collect(),
                        ),
                        ScryptoType::NonFungibleAddress => HardProofRuleResourceList::List(
                            elements
                                .iter()
                                .map(|v| {
                                    if let Value::Custom { bytes, .. } = v {
                                        return NonFungibleAddress::try_from(bytes.as_slice())
                                            .expect("ScryptoValue contains mismatching SBOR type ID and value")
                                            .into();
                                    }
                                    panic!("Unexpected type");
                                })
                                .collect(),
                        ),
                        _ => HardProofRuleResourceList::SoftResourceListNotFound,
                    },
                    _ => HardProofRuleResourceList::SoftResourceListNotFound,
                }
            } else {
                return HardProofRuleResourceList::SoftResourceListNotFound;
            }
        }
    }
}

fn soft_to_hard_resource(
    schema: &Type,
    soft_resource: &SoftResource,
    value: &ScryptoValue,
) -> HardResourceOrNonFungible {
    match soft_resource {
        SoftResource::Dynamic(schema_path) => {
            if let Some(sbor_path) = schema_path.to_sbor_path(schema) {
                match sbor_path.get_from_value(&value.dom) {
                    Some(Value::Custom { type_id, bytes }) => {
                        match ScryptoType::from_id(*type_id)
                            .expect("ScryptoValue contains invalid SBOR type ID")
                        {
                            ScryptoType::ResourceAddress => {
                                ResourceAddress::try_from(bytes.as_slice())
                                    .expect(
                                        "ScryptoValue contains mismatching SBOR type ID and value",
                                    )
                                    .into()
                            }
                            _ => HardResourceOrNonFungible::SoftResourceNotFound,
                        }
                    }
                    _ => HardResourceOrNonFungible::SoftResourceNotFound,
                }
            } else {
                return HardResourceOrNonFungible::SoftResourceNotFound;
            }
        }
        SoftResource::Static(resource_def_id) => {
            HardResourceOrNonFungible::Resource(resource_def_id.clone())
        }
    }
}

fn soft_to_hard_resource_or_non_fungible(
    schema: &Type,
    proof_rule_resource: &SoftResourceOrNonFungible,
    value: &ScryptoValue,
) -> HardResourceOrNonFungible {
    match proof_rule_resource {
        SoftResourceOrNonFungible::Dynamic(schema_path) => {
            if let Some(sbor_path) = schema_path.to_sbor_path(schema) {
                match sbor_path.get_from_value(&value.dom) {
                    Some(Value::Custom { type_id, bytes }) => {
                        match ScryptoType::from_id(*type_id)
                            .expect("ScryptoValue contains invalid SBOR type ID")
                        {
                            ScryptoType::ResourceAddress => {
                                ResourceAddress::try_from(bytes.as_slice())
                                    .expect(
                                        "ScryptoValue contains mismatching SBOR type ID and value",
                                    )
                                    .into()
                            }
                            ScryptoType::NonFungibleAddress => {
                                NonFungibleAddress::try_from(bytes.as_slice())
                                    .expect(
                                        "ScryptoValue contains mismatching SBOR type ID and value",
                                    )
                                    .into()
                            }
                            _ => HardResourceOrNonFungible::SoftResourceNotFound,
                        }
                    }
                    _ => HardResourceOrNonFungible::SoftResourceNotFound,
                }
            } else {
                return HardResourceOrNonFungible::SoftResourceNotFound;
            }
        }
        SoftResourceOrNonFungible::StaticNonFungible(non_fungible_address) => {
            HardResourceOrNonFungible::NonFungible(non_fungible_address.clone())
        }
        SoftResourceOrNonFungible::StaticResource(resource_def_id) => {
            HardResourceOrNonFungible::Resource(resource_def_id.clone())
        }
    }
}

fn soft_to_hard_proof_rule(
    schema: &Type,
    proof_rule: &ProofRule,
    value: &ScryptoValue,
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
    value: &ScryptoValue,
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

pub fn convert(
    schema: &Type,
    value: &ScryptoValue,
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
