use radix_engine_derive::scrypto;
use radix_engine_interface::api::api::SysNativeInvokable;
use radix_engine_interface::api::types::{ComponentId, GlobalAddress, RENodeId, ToString};
use radix_engine_interface::model::{
    AccessRule, AccessRuleKey, AccessRulesSetMethodAccessRuleInvocation,
    AccessRulesSetMethodMutabilityInvocation, ComponentAddress,
};

use crate::engine::scrypto_env::ScryptoEnv;

// TODO: Should `Encode` and `Decode` be removed so that `ComponentAccessRules` can not be passed
// between components?
#[derive(Debug, Clone, PartialEq, Eq)]
#[scrypto(TypeId, Encode, Decode, Describe)]
pub struct ComponentAccessRules {
    component: ComponentIdentifier,
    index: u32,
}

impl ComponentAccessRules {
    pub(crate) fn new<T: Into<ComponentIdentifier>>(component: T, index: u32) -> Self {
        Self {
            component: component.into(),
            index,
        }
    }

    pub fn component_identifier(&self) -> &ComponentIdentifier {
        &self.component
    }

    pub fn index(&self) -> u32 {
        self.index
    }

    pub fn set_method_auth(&mut self, method_name: &str, access_rule: AccessRule) {
        let mut syscalls = ScryptoEnv;
        syscalls
            .sys_invoke(AccessRulesSetMethodAccessRuleInvocation {
                receiver: self.component.clone().into(),
                index: self.index,
                key: AccessRuleKey::ScryptoMethod(method_name.to_string()),
                rule: access_rule,
            })
            .unwrap();
    }

    pub fn lock_method_auth(&mut self, method_name: &str) {
        let mut syscalls = ScryptoEnv;
        syscalls
            .sys_invoke(AccessRulesSetMethodMutabilityInvocation {
                receiver: self.component.clone().into(),
                index: self.index,
                key: AccessRuleKey::ScryptoMethod(method_name.to_string()),
                mutability: AccessRule::DenyAll,
            })
            .unwrap();
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[scrypto(TypeId, Encode, Decode, Describe)]
pub enum ComponentIdentifier {
    RENodeId(ComponentId),
    GlobalAddress(ComponentAddress),
}

impl From<ComponentId> for ComponentIdentifier {
    fn from(value: ComponentId) -> Self {
        ComponentIdentifier::RENodeId(value)
    }
}

impl From<ComponentAddress> for ComponentIdentifier {
    fn from(value: ComponentAddress) -> Self {
        ComponentIdentifier::GlobalAddress(value)
    }
}

impl From<ComponentIdentifier> for RENodeId {
    fn from(value: ComponentIdentifier) -> Self {
        match value {
            ComponentIdentifier::RENodeId(node_id) => RENodeId::Component(node_id),
            ComponentIdentifier::GlobalAddress(component_address) => {
                RENodeId::Global(GlobalAddress::Component(component_address))
            }
        }
    }
}
