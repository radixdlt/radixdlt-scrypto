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
    index: usize,
}

impl ComponentAccessRules {
    pub(crate) fn new<T: Into<ComponentIdentifier>>(component: T, index: usize) -> Self {
        Self {
            component: component.into(),
            index,
        }
    }

    pub fn component_address(&self) -> ComponentIdentifier {
        self.component.clone()
    }

    pub fn index(&self) -> usize {
        self.index
    }

    pub fn set_method_auth(&mut self, method_name: &str, access_rule: AccessRule) {
        let mut syscalls = ScryptoEnv;
        syscalls
            .sys_invoke(AccessRulesSetMethodAccessRuleInvocation {
                receiver: self.component_re_node(),
                index: self.index as u32,
                key: AccessRuleKey::ScryptoMethod(method_name.to_string()).into(),
                rule: access_rule,
            })
            .unwrap();
    }

    pub fn lock_method_auth(&mut self, method_name: &str) {
        let mut syscalls = ScryptoEnv;
        syscalls
            .sys_invoke(AccessRulesSetMethodMutabilityInvocation {
                receiver: self.component_re_node(),
                index: self.index as u32,
                key: AccessRuleKey::ScryptoMethod(method_name.to_string()).into(),
                mutability: AccessRule::DenyAll,
            })
            .unwrap();
    }

    fn component_re_node(&self) -> RENodeId {
        match self.component {
            ComponentIdentifier::NodeId(node_id) => RENodeId::Component(node_id),
            ComponentIdentifier::ComponentAddress(component_address) => {
                RENodeId::Global(GlobalAddress::Component(component_address))
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[scrypto(TypeId, Encode, Decode, Describe)]
pub enum ComponentIdentifier {
    NodeId(ComponentId),
    ComponentAddress(ComponentAddress),
}

impl From<ComponentId> for ComponentIdentifier {
    fn from(value: ComponentId) -> Self {
        ComponentIdentifier::NodeId(value)
    }
}

impl From<ComponentAddress> for ComponentIdentifier {
    fn from(value: ComponentAddress) -> Self {
        ComponentIdentifier::ComponentAddress(value)
    }
}
