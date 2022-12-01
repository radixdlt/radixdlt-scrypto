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
        method_name: String,
    ) -> Vec<MethodAuthorization> {
        let key = AccessRuleKey::ScryptoMethod(method_name);

        let data = IndexedScryptoValue::from_slice(&component_state.raw)
            .expect("Failed to decode component state");

        let mut authorizations = Vec::new();
        for auth in &self.access_rules {
            let method_auth = auth.get(&key);
            let authorization = convert(schema, &data, method_auth);
            authorizations.push(authorization);
        }

        authorizations
    }

    pub fn native_fn_authorization(&self, native_fn: NativeFn) -> Vec<MethodAuthorization> {
        let key = AccessRuleKey::Native(native_fn);

        let mut authorizations = Vec::new();
        for auth in &self.access_rules {
            let method_auth = auth.get(&key);

            // TODO: Remove
            let authorization = convert(&Type::Any, &IndexedScryptoValue::unit(), method_auth);
            authorizations.push(authorization);
        }

        authorizations
    }

    pub fn method_mutability_authorization(&self, key: &AccessRuleKey) -> Vec<MethodAuthorization> {
        let mut authorizations = Vec::new();
        for auth in &self.access_rules {
            let method_auth = auth.get_mutability(key);

            // TODO: Remove
            let authorization = convert(&Type::Any, &IndexedScryptoValue::unit(), method_auth);
            authorizations.push(authorization);
        }

        authorizations
    }

    pub fn group_mutability_authorization(&self, name: &str) -> Vec<MethodAuthorization> {
        let mut authorizations = Vec::new();
        for auth in &self.access_rules {
            let group_auth = auth.get_group_mutability(name);

            // TODO: Remove
            let authorization = convert(&Type::Any, &IndexedScryptoValue::unit(), group_auth);
            authorizations.push(authorization);
        }

        authorizations
    }
}
