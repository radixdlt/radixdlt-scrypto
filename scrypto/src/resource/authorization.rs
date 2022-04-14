use crate::resource::*;
use crate::rust::collections::hash_map::Iter;
use crate::rust::collections::HashMap;
use crate::rust::str;
use crate::rust::string::String;
use crate::rust::string::ToString;
use sbor::*;

/// Method authorization rules for a component
#[derive(Debug, Clone, PartialEq, Describe, TypeId, Encode, Decode)]
pub struct Authorization {
    method_auth: HashMap<String, MethodAuth>,
    default_auth: Option<MethodAuth>
}

impl Authorization {
    pub fn new() -> Self {
        Self {
            method_auth: HashMap::new(),
            default_auth: Option::None
        }
    }

    pub fn get(&self, method_name: &str) -> Option<&MethodAuth> {
        self.method_auth.get(method_name).or_else(|| self.default_auth.as_ref())
    }

    pub fn method(mut self, method_name: &str, method_auth: MethodAuth) -> Self {
        self.method_auth.insert(method_name.to_string(), method_auth);
        self
    }

    pub fn default(mut self, method_auth: MethodAuth) -> Self {
        self.default_auth = Some(method_auth);
        self
    }

    pub fn iter(&self) -> Iter<'_, String, MethodAuth> {
        let l = self.method_auth.iter();
        l
    }
}
