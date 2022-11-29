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
            let (access_rules, _) = auth.get(method_name);
            let authorization = convert(schema, &data, &access_rules);
            authorizations.push(authorization);
        }

        authorizations
    }

    pub fn mutability_method_authorization(
        &self,
        component_state: &ComponentStateSubstate,
        schema: &Type,
        method_name: &AccessRulesMethodIdent,
    ) -> Vec<MethodAuthorization> {
        let data = IndexedScryptoValue::from_slice(&component_state.raw)
            .expect("Failed to decode component state");

        let mut authorizations = Vec::new();
        for auth in &self.access_rules {
            let (_, mutability) = match method_name {
                AccessRulesMethodIdent::Default => auth.get_default(),
                AccessRulesMethodIdent::Method(method_name) => auth.get(method_name),
            };
            let authorization = match mutability {
                LOCKED => MethodAuthorization::DenyAll,
                MUTABLE(access_rules) => convert(schema, &data, access_rules),
            };
            authorizations.push(authorization);
        }

        authorizations
    }
}
