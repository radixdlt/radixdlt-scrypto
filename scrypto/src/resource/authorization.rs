use crate::resource::*;
use crate::rust::collections::HashMap;
use crate::rust::str;
use crate::rust::string::String;
use crate::rust::string::ToString;
use sbor::*;

/// Authorization Rule
#[derive(Debug, Clone, PartialEq, Describe, TypeId, Encode, Decode)]
pub struct ComponentAuthorization(HashMap<String, ProofRule>);

impl ComponentAuthorization {
    pub fn new() -> Self {
        ComponentAuthorization(HashMap::new())
    }

    pub fn single_auth(method_name: &str, proof_rule: ProofRule) -> Self {
        let mut map = HashMap::new();
        map.insert(method_name.to_string(), proof_rule);
        ComponentAuthorization(map)
    }

    pub fn to_map(self) -> HashMap<String, ProofRule> {
        self.0
    }
}