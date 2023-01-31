use sbor::rust::format;
use sbor::rust::string::String;

use crate::address::entity::EntityType;
use crate::node::NetworkDefinition;

/// Represents an HRP set (typically corresponds to a network).
#[derive(Debug, Clone)]
pub struct HrpSet {
    package: String,
    resource: String,
    normal_component: String,
    account_component: String,
    identity_component: String,
    epoch_manager_component: String,
    clock_component: String,
    validator_component: String,
    access_controller_component: String,
}

impl HrpSet {
    pub fn get_entity_hrp(&self, entity: &EntityType) -> &str {
        match entity {
            EntityType::Resource => &self.resource,
            EntityType::Package => &self.package,

            EntityType::NormalComponent => &self.normal_component,
            EntityType::AccountComponent => &self.account_component,
            EntityType::IdentityComponent => &self.identity_component,
            EntityType::EpochManager => &self.epoch_manager_component,
            EntityType::Validator => &self.validator_component,
            EntityType::Clock => &self.clock_component,
            EntityType::EcdsaSecp256k1VirtualAccountComponent => &self.account_component,
            EntityType::EddsaEd25519VirtualAccountComponent => &self.account_component,
            EntityType::EcdsaSecp256k1VirtualIdentityComponent => &self.identity_component,
            EntityType::EddsaEd25519VirtualIdentityComponent => &self.identity_component,
            EntityType::AccessControllerComponent => &self.access_controller_component,
        }
    }
}

impl From<&NetworkDefinition> for HrpSet {
    fn from(network_definition: &NetworkDefinition) -> Self {
        let suffix = &network_definition.hrp_suffix;
        HrpSet {
            package: format!("package_{}", suffix),
            resource: format!("resource_{}", suffix),
            normal_component: format!("component_{}", suffix),
            account_component: format!("account_{}", suffix),
            identity_component: format!("identity_{}", suffix),
            epoch_manager_component: format!("epochmanager_{}", suffix),
            clock_component: format!("clock_{}", suffix),
            validator_component: format!("validator_{}", suffix),
            access_controller_component: format!("accesscontroller_{}", suffix),
        }
    }
}
