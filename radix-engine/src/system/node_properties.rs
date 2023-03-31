use crate::kernel::actor::{Actor, ActorIdentifier, ExecutionMode};
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
                    actor
                        .fn_identifier
                        .package_address
                        .eq(&RESOURCE_MANAGER_PACKAGE)
                        && actor.fn_identifier.blueprint_name.eq(PROOF_BLUEPRINT)
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
                    _ => package_address.eq(&actor.fn_identifier.package_address),
                }
            }
            _ => return false,
        }
    }

    /// Whether the substate can be read
    pub fn can_be_read(
        mode: ExecutionMode,
        actor: &Actor,
        node_id: &NodeId,
        module_id: TypedModuleId,
        substate_key: &SubstateKey,
    ) -> bool {
        match mode {
            ExecutionMode::Kernel => match module_id {
                TypedModuleId::TypeInfo => true,
                _ => false,
            },
            ExecutionMode::Resolver => match module_id {
                TypedModuleId::TypeInfo => true,
                TypedModuleId::ObjectState => true,
                _ => false,
            },
            ExecutionMode::DropNode => match module_id {
                TypedModuleId::TypeInfo => true,
                _ => false,
            },
            ExecutionMode::AutoDrop => match module_id {
                TypedModuleId::TypeInfo => true,
                _ => false,
            },
            ExecutionMode::System => match module_id {
                TypedModuleId::TypeInfo => true,
                TypedModuleId::ObjectState => true,
                _ => false,
            },
            ExecutionMode::KernelModule => true,
            ExecutionMode::Client => match &actor.fn_identifier {
                // Native
                FnIdentifier {
                    package_address, ..
                } if is_native_package(*package_address) => true,
                // Scrypto
                _ => match &actor.identifier {
                    ActorIdentifier::VirtualLazyLoad | ActorIdentifier::Function(..) => {
                        match module_id {
                            TypedModuleId::TypeInfo => true,
                            TypedModuleId::KeyValueStore => true,
                            _ => false,
                        }
                    }
                    ActorIdentifier::Method(MethodIdentifier(actor_node_id, ..)) => match module_id
                    {
                        TypedModuleId::TypeInfo => true,
                        TypedModuleId::KeyValueStore => true,
                        TypedModuleId::ObjectState if actor_node_id == node_id => true,
                        _ => false,
                    },
                },
            },
        }
    }

    /// Whether the substate can be written
    pub fn can_be_written(
        mode: ExecutionMode,
        actor: &Actor,
        node_id: &NodeId,
        module_id: TypedModuleId,
        substate_key: &SubstateKey,
    ) -> bool {
        match mode {
            ExecutionMode::Kernel => match module_id {
                _ => false,
            },
            ExecutionMode::Resolver => match module_id {
                _ => false,
            },
            ExecutionMode::DropNode => match module_id {
                _ => false,
            },
            ExecutionMode::AutoDrop => match module_id {
                _ => false,
            },
            ExecutionMode::System => match module_id {
                _ => false,
            },
            ExecutionMode::KernelModule => true,
            ExecutionMode::Client => match &actor.fn_identifier {
                // Native
                FnIdentifier {
                    package_address, ..
                } if is_native_package(*package_address) => true,
                // Scrypto
                _ => match &actor.identifier {
                    ActorIdentifier::VirtualLazyLoad | ActorIdentifier::Function(..) => {
                        match module_id {
                            _ => false,
                        }
                    }
                    ActorIdentifier::Method(MethodIdentifier(actor_node_id, ..)) => match module_id
                    {
                        TypedModuleId::ObjectState if actor_node_id == node_id => true,
                        _ => false,
                    },
                },
            },
        }
    }

    pub fn can_own(
        offset: &SubstateKey,
        package_address: PackageAddress,
        blueprint_name: &str,
    ) -> bool {
        true
    }
}
