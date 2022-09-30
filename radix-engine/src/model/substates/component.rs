use crate::model::{convert, MethodAuthorization};
use crate::types::*;

#[derive(Debug, Clone, TypeId, Encode, Decode, PartialEq, Eq)]
pub struct ComponentStateSubstate {
    pub state: Vec<u8>,
}

impl ComponentStateSubstate {
    pub fn new(state: Vec<u8>) -> Self {
        ComponentStateSubstate { state }
    }

    pub fn state(&self) -> &[u8] {
        &self.state
    }

    pub fn set_state(&mut self, new_state: Vec<u8>) {
        self.state = new_state;
    }
}

#[derive(Debug, Clone, TypeId, Encode, Decode, PartialEq, Eq)]
pub struct ComponentInfoSubstate {
    pub package_address: PackageAddress,
    pub blueprint_name: String,
    pub access_rules: Vec<AccessRules>,
}

impl ComponentInfoSubstate {
    pub fn new(
        package_address: PackageAddress,
        blueprint_name: String,
        access_rules: Vec<AccessRules>,
    ) -> Self {
        Self {
            package_address,
            blueprint_name,
            access_rules,
        }
    }

    pub fn method_authorization(
        &self,
        component_state: &ComponentStateSubstate,
        schema: &Type,
        method_name: &str,
    ) -> Vec<MethodAuthorization> {
        let data = ScryptoValue::from_slice(&component_state.state)
            .expect("Failed to decode component state");

        let mut authorizations = Vec::new();
        for auth in &self.access_rules {
            let method_auth = auth.get(method_name);
            let authorization = convert(schema, &data, method_auth);
            authorizations.push(authorization);
        }

        authorizations
    }
}
