use crate::address::entity::EntityType;
use crate::core::Network;
/// Represents an HRP set (typically corresponds to a network).
#[derive(Debug, Clone, Copy)]
pub struct HrpSet {
    pub resource: &'static str,

    pub package: &'static str,

    pub component: &'static str,
    pub account_component: &'static str,
    pub system_component: &'static str,
}

impl HrpSet {
    pub fn get_entity_hrp(&self, entity: &EntityType) -> &'static str {
        match entity {
            EntityType::Resource => self.resource,
            EntityType::Package => self.package,

            EntityType::Component => self.component,
            EntityType::AccountComponent => self.account_component,
            EntityType::SystemComponent => self.system_component,
        }
    }
}

/// The Human Readable Parts used for the Local Simulator.
pub const LOCAL_SIMULATOR_NETWORK_HRP_SET: HrpSet = HrpSet {
    component: "comp_sim",
    account_component: "acct_sim",
    system_component: "sys_sim",
    package: "pkg_sim",
    resource: "asset_sim",
};

/// The Human Readable Parts used for the Internal Test Network.
pub const INTERNAL_TEST_NETWORK_HRP_SET: HrpSet = HrpSet {
    component: "comp_itn",
    account_component: "acct_itn",
    system_component: "sys_itn",
    package: "pkg_itn",
    resource: "asset_itn",
};

/// Returns the HrpSet associated with the network.
pub fn get_network_hrp_set(network: &Network) -> HrpSet {
    match network {
        Network::LocalSimulator => LOCAL_SIMULATOR_NETWORK_HRP_SET,
        Network::InternalTestnet => INTERNAL_TEST_NETWORK_HRP_SET,
    }
}
