use crate::kernel::actor::{Actor, ExecutionMode};
use crate::types::*;
use radix_engine_interface::api::node_modules::auth::ACCESS_RULES_BLUEPRINT;
use radix_engine_interface::api::node_modules::metadata::METADATA_BLUEPRINT;
use radix_engine_interface::api::node_modules::royalty::COMPONENT_ROYALTY_BLUEPRINT;
use radix_engine_interface::blueprints::resource::{PROOF_BLUEPRINT, WORKTOP_BLUEPRINT};
use radix_engine_interface::constants::*;

pub struct NodeProperties;

impl NodeProperties {
    /// Whether a node of the given blueprint can be dropped.
    pub fn can_be_dropped(
        mode: ExecutionMode,
        actor: &Actor,
        package_address: PackageAddress,
        blueprint: &str,
    ) -> bool {
        match mode {
            ExecutionMode::Kernel => true,
            ExecutionMode::KernelModule => true,
            ExecutionMode::AutoDrop => {
                if package_address.eq(&RESOURCE_MANAGER_PACKAGE) && blueprint.eq(PROOF_BLUEPRINT) {
                    actor.package_address().eq(&RESOURCE_MANAGER_PACKAGE)
                        && actor.blueprint_name().eq(PROOF_BLUEPRINT)
                } else {
                    false
                }
            }
            ExecutionMode::Client => {
                match (package_address, blueprint) {
                    (RESOURCE_MANAGER_PACKAGE, WORKTOP_BLUEPRINT) => true, // TODO: Remove
                    (METADATA_PACKAGE, METADATA_BLUEPRINT)
                    | (ROYALTY_PACKAGE, COMPONENT_ROYALTY_BLUEPRINT)
                    | (ACCESS_RULES_PACKAGE, ACCESS_RULES_BLUEPRINT) => true, // TODO: This is required for current implementation of globalize, maybe there's a better way
                    _ => package_address.eq(actor.package_address()),
                }
            }
            _ => return false,
        }
    }

    pub fn can_own(
        _offset: &SubstateKey,
        _package_address: PackageAddress,
        _blueprint_name: &str,
    ) -> bool {
        true
    }
}
