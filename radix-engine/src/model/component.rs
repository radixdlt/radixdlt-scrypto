use crate::model::MethodAuthorization;
use sbor::*;
use scrypto::engine::types::*;
use scrypto::resource::ProofRule;
use scrypto::rust::collections::*;
use scrypto::rust::string::String;
use scrypto::rust::vec::Vec;
use crate::model::method_authorization::HardProofRule;

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

    fn to_hard_rule(proof_rule: &ProofRule) -> HardProofRule {
        match proof_rule {
            ProofRule::FromComponent(_) => panic!("Not yet implemented."),
            ProofRule::This(proof_rule_resource) => HardProofRule::This(proof_rule_resource.clone()),
            ProofRule::SomeOfResource(amount, resource_def_id) => HardProofRule::SomeOfResource(*amount, *resource_def_id),
            ProofRule::AllOf(rules) => {
                let hard_rules = rules.into_iter().map(Self::to_hard_rule).collect();
                HardProofRule::AllOf(hard_rules)
            },
            ProofRule::OneOf(rules) => {
                let hard_rules = rules.into_iter().map(Self::to_hard_rule).collect();
                HardProofRule::OneOf(hard_rules)
            },
            ProofRule::CountOf { count, rules } => {
                let hard_rules = rules.into_iter().map(Self::to_hard_rule).collect();
                HardProofRule::CountOf { count: *count, rules: hard_rules }
            },
        }
    }

    pub fn get_auth(&self, method_name: &str) -> MethodAuthorization {
        match self.auth_rules.get(method_name) {
            Some(proof_rule) => {
                MethodAuthorization::Protected(Self::to_hard_rule(proof_rule))
            },
            None => MethodAuthorization::Public,
        }
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
