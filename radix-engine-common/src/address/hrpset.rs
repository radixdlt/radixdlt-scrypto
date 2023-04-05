use crate::network::NetworkDefinition;
use crate::types::EntityType;
use sbor::rust::prelude::*;

/// Represents an HRP set (typically corresponds to a network).
#[derive(Debug, Clone)]
pub struct HrpSet {
    package: String,
    resource: String,
    component: String,
    account: String,
    identity: String,
    epoch_manager: String,
    clock: String,
    validator: String,
    access_controller: String,
    internal_vault: String,
    internal_access_controller: String,
    internal_account: String,
    internal_component: String,
    internal_key_value_store: String,
}

impl HrpSet {
    pub fn get_entity_hrp(&self, entity: &EntityType) -> &str {
        match entity {
            EntityType::GlobalPackage => &self.package,
            EntityType::GlobalFungibleResource => &self.resource,
            EntityType::GlobalNonFungibleResource => &self.resource,
            EntityType::GlobalEpochManager => &self.epoch_manager,
            EntityType::GlobalValidator => &self.validator,
            EntityType::GlobalClock => &self.clock,
            EntityType::GlobalAccessController => &self.access_controller,
            EntityType::GlobalAccount => &self.account,
            EntityType::GlobalIdentity => &self.identity,
            EntityType::GlobalGenericComponent => &self.component,
            EntityType::GlobalVirtualEcdsaAccount => &self.account,
            EntityType::GlobalVirtualEddsaAccount => &self.account,
            EntityType::GlobalVirtualEcdsaIdentity => &self.identity,
            EntityType::GlobalVirtualEddsaIdentity => &self.identity,
            EntityType::InternalFungibleVault => &self.internal_vault,
            EntityType::InternalNonFungibleVault => &self.internal_vault,
            EntityType::InternalAccessController => &self.internal_access_controller,
            EntityType::InternalAccount => &self.internal_account,
            EntityType::InternalGenericComponent => &self.internal_component,
            EntityType::InternalKeyValueStore => &self.internal_key_value_store,
        }
    }
}

impl From<&NetworkDefinition> for HrpSet {
    fn from(network_definition: &NetworkDefinition) -> Self {
        let suffix = &network_definition.hrp_suffix;
        HrpSet {
            package: format!("package_{}", suffix),
            resource: format!("resource_{}", suffix),
            component: format!("component_{}", suffix),
            account: format!("account_{}", suffix),
            identity: format!("identity_{}", suffix),
            epoch_manager: format!("epochmanager_{}", suffix),
            clock: format!("clock_{}", suffix),
            validator: format!("validator_{}", suffix),
            access_controller: format!("accesscontroller_{}", suffix),
            internal_vault: format!("internal_vault_{}", suffix),
            internal_access_controller: format!("internal_accesscontroller_{}", suffix),
            internal_account: format!("internal_account_{}", suffix),
            internal_component: format!("internal_component_{}", suffix),
            internal_key_value_store: format!("internal_keyvaluestore_{}", suffix),
        }
    }
}
