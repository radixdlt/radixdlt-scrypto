use radix_engine_derive::scrypto;
use radix_engine_interface::api::types::{ComponentId, GlobalAddress, RENodeId};
use radix_engine_interface::model::{AccessRule, AccessRules, ComponentAddress};

// TODO: Should `Decode` be removed so that `StatefulAccessRules` can not be passed between 
// components?
#[derive(Debug, Clone, PartialEq, Eq)]
#[scrypto(TypeId, Encode, Decode, Describe)]
pub struct StatefulAccessRules {
    component: ComponentIdentifier,
    access_rules: AccessRules,
    index: usize,
}

impl StatefulAccessRules {
    pub fn new<T: Into<ComponentIdentifier>>(
        component: T,
        access_rules: AccessRules,
        index: usize,
    ) -> Self {
        Self {
            component: component.into(),
            access_rules,
            index,
        }
    }

    pub fn component_address(&self) -> ComponentIdentifier {
        self.component.clone()
    }

    pub fn access_rules(&self) -> AccessRules {
        self.access_rules.clone()
    }

    pub fn index(&self) -> usize {
        self.index
    }

    pub fn set_method_auth(&mut self, _method_name: &str, _access_rule: AccessRule) {
        todo!();
    }

    pub fn set_default(&mut self, _access_rule: AccessRule) {
        todo!();
    }

    pub fn lock_method_auth(&mut self, _method_name: &str) {
        todo!();
    }

    pub fn lock_default(&mut self) {
        todo!();
    }

    fn _component_re_node(&self) -> RENodeId {
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
