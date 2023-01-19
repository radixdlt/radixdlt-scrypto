use sbor::rust::format;
use sbor::rust::string::String;

use crate::address::entity::EntityType;
use crate::node::NetworkDefinition;

/// Represents an HRP set (typically corresponds to a network).
#[derive(Debug, Clone)]
pub struct HrpSet {
    resource: String,

    package: String,

    normal_component: String,
    account_component: String,
    identity_component: String,

    system_component: String,
}

impl HrpSet {
    pub fn get_entity_hrp(&self, entity: &EntityType) -> &str {
        match entity {
            EntityType::Resource => &self.resource,
            EntityType::Package => &self.package,

            EntityType::NormalComponent => &self.normal_component,
            EntityType::AccountComponent => &self.account_component,

            EntityType::EpochManager => &self.system_component,
            EntityType::Validator => &self.system_component,
            EntityType::Clock => &self.system_component,
            EntityType::EcdsaSecp256k1VirtualAccountComponent => &self.account_component,
            EntityType::EddsaEd25519VirtualAccountComponent => &self.account_component,
            EntityType::EcdsaSecp256k1VirtualIdentityComponent => &self.identity_component,
        }
    }
}

impl From<&NetworkDefinition> for HrpSet {
    fn from(network_definition: &NetworkDefinition) -> Self {
        let suffix = &network_definition.hrp_suffix;
        HrpSet {
            normal_component: format!("component_{}", suffix),
            account_component: format!("account_{}", suffix),
            system_component: format!("system_{}", suffix),
            identity_component: format!("identity_{}", suffix),
            package: format!("package_{}", suffix),
            resource: format!("resource_{}", suffix),
        }
    }
}
