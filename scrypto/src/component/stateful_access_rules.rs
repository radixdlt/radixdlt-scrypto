use radix_engine_derive::scrypto;
use radix_engine_interface::api::api::SysNativeInvokable;
use radix_engine_interface::api::types::{
    AccessRulesOffset, ComponentId, GlobalAddress, RENodeId, SubstateOffset, ToString,
};
use radix_engine_interface::model::{
    AccessRule, AccessRuleKey, AccessRuleSelector, AccessRules, AccessRulesSetAccessRuleInvocation,
    AccessRulesSetMutabilityInvocation, ComponentAddress,
};

use crate::engine::scrypto_env::ScryptoEnv;
use crate::runtime::{DataPointer, DataRef};

use super::AccessRulesSubstate;

// TODO: Should `Encode` and `Decode` be removed so that `StatefulAccessRules` can not be passed
// between components?
#[derive(Debug, Clone, PartialEq, Eq)]
#[scrypto(TypeId, Encode, Decode, Describe)]
pub struct StatefulAccessRules {
    component: ComponentIdentifier,
    index: usize,
}

impl StatefulAccessRules {
    pub(crate) fn new<T: Into<ComponentIdentifier>>(component: T, index: usize) -> Self {
        Self {
            component: component.into(),
            index,
        }
    }

    pub fn component_address(&self) -> ComponentIdentifier {
        self.component.clone()
    }

    pub fn access_rules(&self) -> AccessRules {
        let pointer = DataPointer::new(
            self.component_re_node(),
            SubstateOffset::AccessRules(AccessRulesOffset::AccessRules),
        );
        let state: DataRef<AccessRulesSubstate> = pointer.get();
        state
            .access_rules
            .get(self.index)
            .expect("No access rule found at trusted index")
            .clone()
    }

    pub fn index(&self) -> usize {
        self.index
    }

    pub fn set_method_auth(&mut self, method_name: &str, access_rule: AccessRule) {
        let mut syscalls = ScryptoEnv;
        syscalls
            .sys_invoke(AccessRulesSetAccessRuleInvocation {
                receiver: self.component_re_node(),
                index: self.index as u32,
                selector: AccessRuleKey::ScryptoMethod(method_name.to_string()).into(),
                rule: access_rule,
            })
            .unwrap();
    }

    pub fn set_default(&mut self, access_rule: AccessRule) {
        let mut syscalls = ScryptoEnv;
        syscalls
            .sys_invoke(AccessRulesSetAccessRuleInvocation {
                receiver: self.component_re_node(),
                index: self.index as u32,
                selector: AccessRuleSelector::Default,
                rule: access_rule,
            })
            .unwrap();
    }

    pub fn lock_method_auth(&mut self, method_name: &str) {
        let mut syscalls = ScryptoEnv;
        syscalls
            .sys_invoke(AccessRulesSetMutabilityInvocation {
                receiver: self.component_re_node(),
                index: self.index as u32,
                selector: AccessRuleKey::ScryptoMethod(method_name.to_string()).into(),
                mutability: AccessRule::DenyAll,
            })
            .unwrap();
    }

    pub fn lock_default(&mut self) {
        let mut syscalls = ScryptoEnv;
        syscalls
            .sys_invoke(AccessRulesSetMutabilityInvocation {
                receiver: self.component_re_node(),
                index: self.index as u32,
                selector: AccessRuleSelector::Default,
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
