use crate::model::{convert, ComponentStateSubstate, MethodAuthorization};
use crate::types::*;
use radix_engine_interface::abi::Type;
use radix_engine_interface::data::IndexedScryptoValue;
use radix_engine_interface::model::AccessRules;

/// A transient resource container.
#[derive(Debug, Clone, PartialEq, Eq)]
#[scrypto(TypeId, Encode, Decode)]
pub struct AccessRulesSubstate {
    pub access_rules: Vec<AccessRules>,
}

impl AccessRulesSubstate {
    pub fn method_authorization(
        &self,
        component_state: &ComponentStateSubstate,
        schema: &Type,
        method_name: &str,
    ) -> Vec<MethodAuthorization> {
        let data = IndexedScryptoValue::from_slice(&component_state.raw)
            .expect("Failed to decode component state");

        let mut authorizations = Vec::new();
        for auth in &self.access_rules {
            let method_auth = auth.get(method_name);
            let authorization = convert(schema, &data, &method_auth.0);
            authorizations.push(authorization);
        }

        authorizations
    }
}
