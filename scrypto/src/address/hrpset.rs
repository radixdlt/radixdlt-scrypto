use crate::address::entity::EntityType;
use crate::core::NetworkDefinition;
use crate::sbor::rust::format;
use crate::sbor::rust::string::String;

/// Represents an HRP set (typically corresponds to a network).
#[derive(Debug, Clone)]
pub struct HrpSet {
    resource: String,

    package: String,

    component: String,
    account_component: String,
    system_component: String,
}

impl HrpSet {
    pub fn get_entity_hrp(&self, entity: &EntityType) -> &str {
        match entity {
            EntityType::Resource => &self.resource,
            EntityType::Package => &self.package,

            EntityType::Component => &self.component,
            EntityType::AccountComponent => &self.account_component,
            EntityType::SystemComponent => &self.system_component,
        }
    }
}

impl From<&NetworkDefinition> for HrpSet {
    fn from(network_definition: &NetworkDefinition) -> Self {
        let suffix = &network_definition.hrp_suffix;
        HrpSet {
            component: format!("component_{}", suffix),
            account_component: format!("account_{}", suffix),
            system_component: format!("system_{}", suffix),
            package: format!("package_{}", suffix),
            resource: format!("resource_{}", suffix),
        }
    }
}
