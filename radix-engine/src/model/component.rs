use crate::model::method_authorization::{
    HardProofRule, HardProofRuleResourceList, HardResourceOrNonFungible,
};
use crate::model::{MethodAuthorization, ValidatedData};
use sbor::any::Value;
use sbor::*;
use scrypto::engine::types::*;
use scrypto::prelude::SoftResource;
use scrypto::resource::{
    NonFungibleAddress, ProofRule, SoftResourceOrNonFungible, SoftResourceOrNonFungibleList,
};
use scrypto::rust::collections::*;
use scrypto::rust::string::String;
use scrypto::rust::vec::Vec;
use scrypto::types::CustomType;

/// A component is an instance of blueprint.
#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct Component {
    package_id: PackageId,
    blueprint_name: String,
    auth_rules: HashMap<String, ProofRule>,
    state: Vec<u8>,
}

impl Component {
    pub fn new(
        package_id: PackageId,
        blueprint_name: String,
        auth_rules: HashMap<String, ProofRule>,
        state: Vec<u8>,
    ) -> Self {
        Self {
            package_id,
            blueprint_name,
            auth_rules,
            state,
        }
    }

    fn soft_to_hard_resource_list(
        list: &SoftResourceOrNonFungibleList,
        dom: &Value,
    ) -> HardProofRuleResourceList {
        match list {
            SoftResourceOrNonFungibleList::Static(resources) => {
                let mut hard_resources = Vec::new();
                for soft_resource in resources {
                    let resource = Self::soft_to_hard_resource_or_non_fungible(soft_resource, dom);
                    hard_resources.push(resource);
                }
                HardProofRuleResourceList::List(hard_resources)
            }
            SoftResourceOrNonFungibleList::Dynamic(path) => match path.rel_path().get_from(dom) {
                Some(Value::Vec(type_id, values)) => match CustomType::from_id(*type_id).unwrap() {
                    CustomType::ResourceDefId => HardProofRuleResourceList::List(
                        values
                            .iter()
                            .map(|v| {
                                if let Value::Custom(_, bytes) = v {
                                    return ResourceDefId::try_from(bytes.as_slice())
                                        .unwrap()
                                        .into();
                                }
                                panic!("Unexpected type");
                            })
                            .collect(),
                    ),
                    CustomType::NonFungibleAddress => HardProofRuleResourceList::List(
                        values
                            .iter()
                            .map(|v| {
                                if let Value::Custom(_, bytes) = v {
                                    return NonFungibleAddress::try_from(bytes.as_slice())
                                        .unwrap()
                                        .into();
                                }
                                panic!("Unexpected type");
                            })
                            .collect(),
                    ),
                    _ => HardProofRuleResourceList::SoftResourceListNotFound,
                },
                _ => HardProofRuleResourceList::SoftResourceListNotFound,
            },
        }
    }

    fn soft_to_hard_resource(
        soft_resource: &SoftResource,
        dom: &Value,
    ) -> HardResourceOrNonFungible {
        match soft_resource {
            SoftResource::Dynamic(path) => match path.rel_path().get_from(dom) {
                Some(Value::Custom(type_id, bytes)) => {
                    match CustomType::from_id(*type_id).unwrap() {
                        CustomType::ResourceDefId => {
                            ResourceDefId::try_from(bytes.as_slice()).unwrap().into()
                        }
                        _ => HardResourceOrNonFungible::SoftResourceNotFound,
                    }
                }
                _ => HardResourceOrNonFungible::SoftResourceNotFound,
            },
            SoftResource::Static(resource_def_id) => {
                HardResourceOrNonFungible::Resource(resource_def_id.clone())
            }
        }
    }

    fn soft_to_hard_resource_or_non_fungible(
        proof_rule_resource: &SoftResourceOrNonFungible,
        dom: &Value,
    ) -> HardResourceOrNonFungible {
        match proof_rule_resource {
            SoftResourceOrNonFungible::Dynamic(path) => match path.rel_path().get_from(dom) {
                Some(Value::Custom(type_id, bytes)) => {
                    match CustomType::from_id(*type_id).unwrap() {
                        CustomType::ResourceDefId => {
                            ResourceDefId::try_from(bytes.as_slice()).unwrap().into()
                        }
                        CustomType::NonFungibleAddress => {
                            NonFungibleAddress::try_from(bytes.as_slice())
                                .unwrap()
                                .into()
                        }
                        _ => HardResourceOrNonFungible::SoftResourceNotFound,
                    }
                }
                _ => HardResourceOrNonFungible::SoftResourceNotFound,
            },
            SoftResourceOrNonFungible::StaticNonFungible(non_fungible_address) => {
                HardResourceOrNonFungible::NonFungible(non_fungible_address.clone())
            }
            SoftResourceOrNonFungible::StaticResource(resource_def_id) => {
                HardResourceOrNonFungible::Resource(resource_def_id.clone())
            }
        }
    }

    fn soft_to_hard_rule(proof_rule: &ProofRule, dom: &Value) -> HardProofRule {
        match proof_rule {
            ProofRule::This(soft_resource_or_non_fungible) => {
                let resource =
                    Self::soft_to_hard_resource_or_non_fungible(soft_resource_or_non_fungible, dom);
                HardProofRule::This(resource)
            }
            ProofRule::AmountOf(amount, soft_resource) => {
                let resource = Self::soft_to_hard_resource(soft_resource, dom);
                HardProofRule::SomeOfResource(*amount, resource)
            }
            ProofRule::AllOf(resources) => {
                let hard_resources = Self::soft_to_hard_resource_list(resources, dom);
                HardProofRule::AllOf(hard_resources)
            }
            ProofRule::AnyOf(resources) => {
                let hard_resources = Self::soft_to_hard_resource_list(resources, dom);
                HardProofRule::AnyOf(hard_resources)
            }
            ProofRule::CountOf(count, resources) => {
                let hard_resources = Self::soft_to_hard_resource_list(resources, dom);
                HardProofRule::CountOf(*count, hard_resources)
            }
        }
    }

    pub fn initialize_method(&self, method_name: &str) -> (ValidatedData, MethodAuthorization) {
        let data = ValidatedData::from_slice(&self.state).unwrap();
        let authorization = match self.auth_rules.get(method_name) {
            Some(proof_rule) => {
                MethodAuthorization::Protected(Self::soft_to_hard_rule(proof_rule, &data.dom))
            }
            None => MethodAuthorization::Public,
        };

        (data, authorization)
    }

    pub fn auth_rules(&self) -> &HashMap<String, ProofRule> {
        &self.auth_rules
    }

    pub fn package_id(&self) -> PackageId {
        self.package_id
    }

    pub fn blueprint_name(&self) -> &str {
        &self.blueprint_name
    }

    pub fn state(&self) -> &[u8] {
        &self.state
    }

    pub fn set_state(&mut self, new_state: Vec<u8>) {
        self.state = new_state;
    }
}
