use crate::errors::{InvalidOwnership, KernelError, RuntimeError};
use crate::kernel::actor::{Actor, ActorIdentifier, ExecutionMode};
use crate::types::*;
use radix_engine_interface::api::node_modules::auth::ACCESS_RULES_BLUEPRINT;
use radix_engine_interface::api::node_modules::metadata::METADATA_BLUEPRINT;
use radix_engine_interface::api::node_modules::royalty::COMPONENT_ROYALTY_BLUEPRINT;
use radix_engine_interface::api::substate_api::LockFlags;
use radix_engine_interface::blueprints::resource::{
    BUCKET_BLUEPRINT, PROOF_BLUEPRINT, WORKTOP_BLUEPRINT,
};
use radix_engine_interface::constants::*;

pub struct VisibilityProperties;

impl VisibilityProperties {
    pub fn check_drop_node_visibility(
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

    pub fn check_substate_access(
        mode: ExecutionMode,
        actor: &Actor,
        node_id: &NodeId,
        substate_key: SubstateKey,
        flags: LockFlags,
    ) -> bool {
        let read_only = flags == LockFlags::read_only();

        // TODO: Cleanup and reduce to least privilege
        match (mode, offset) {
            (ExecutionMode::Kernel, offset) => match offset {
                SubstateKey::TypeInfo(TypeInfoOffset::TypeInfo) => true,
                _ => false, // Protect ourselves!
            },
            (ExecutionMode::Resolver, offset) => match offset {
                SubstateKey::TypeInfo(TypeInfoOffset::TypeInfo) => read_only,
                SubstateKey::Package(PackageOffset::CodeType) => read_only,
                SubstateKey::Package(PackageOffset::Info) => read_only,
                SubstateKey::Bucket(BucketOffset::Info) => read_only,
                _ => false,
            },
            (ExecutionMode::AutoDrop, offset) => match offset {
                SubstateKey::TypeInfo(TypeInfoOffset::TypeInfo) => true,
                _ => false,
            },
            (ExecutionMode::DropNode, offset) => match offset {
                SubstateKey::TypeInfo(TypeInfoOffset::TypeInfo) => true,
                SubstateKey::Bucket(BucketOffset::Info) => true,
                SubstateKey::Proof(ProofOffset::Info) => true,
                SubstateKey::Proof(..) => true,
                SubstateKey::Worktop(WorktopOffset::Worktop) => true,
                _ => false,
            },
            (ExecutionMode::System, offset) => match offset {
                SubstateKey::AuthZone(_) => read_only,
                _ => false,
            },
            (ExecutionMode::KernelModule, offset) => match offset {
                // TODO: refine based on specific module
                SubstateKey::ResourceManager(ResourceManagerOffset::ResourceManager) => read_only,
                SubstateKey::Vault(..) => true,
                SubstateKey::Bucket(..) => read_only,
                SubstateKey::Proof(..) => true,
                SubstateKey::Package(PackageOffset::Info) => read_only,
                SubstateKey::Package(PackageOffset::CodeType) => read_only,
                SubstateKey::Package(PackageOffset::Code) => read_only,
                SubstateKey::Package(PackageOffset::Royalty) => true,
                SubstateKey::Package(PackageOffset::FunctionAccessRules) => true,
                SubstateKey::Component(ComponentOffset::State0) => read_only,
                SubstateKey::TypeInfo(_) => read_only,
                SubstateKey::AccessRules(_) => read_only,
                SubstateKey::AuthZone(_) => read_only,
                SubstateKey::Royalty(_) => true,
                _ => false,
            },
            (ExecutionMode::Client, offset) => {
                if !flags.contains(LockFlags::MUTABLE) {
                    if matches!(offset, SubstateKey::TypeInfo(TypeInfoOffset::TypeInfo)) {
                        return true;
                    }

                    match &actor.fn_identifier {
                        // Native
                        FnIdentifier {
                            package_address, ..
                        } if is_native_package(*package_address) => true,
                        // Scrypto
                        _ => match &actor.identifier {
                            ActorIdentifier::VirtualLazyLoad | ActorIdentifier::Function(..) => {
                                match (node_id, offset) {
                                    // READ package code & abi
                                    (
                                        NodeId::GlobalObject(_),
                                        SubstateKey::Package(PackageOffset::Info), // TODO: Remove
                                    )
                                    | (
                                        NodeId::GlobalObject(_),
                                        SubstateKey::Package(PackageOffset::CodeType), // TODO: Remove
                                    )
                                    | (
                                        NodeId::GlobalObject(_),
                                        SubstateKey::Package(PackageOffset::Code), // TODO: Remove
                                    ) => read_only,
                                    // READ global substates
                                    (
                                        NodeId::Object(_),
                                        SubstateKey::TypeInfo(TypeInfoOffset::TypeInfo),
                                    ) => read_only,
                                    // READ/WRITE KVStore entry
                                    (
                                        NodeId::KeyValueStore(_),
                                        SubstateKey::KeyValueStore(KeyValueStoreOffset::Entry(..)),
                                    ) => true,
                                    // Otherwise, false
                                    _ => false,
                                }
                            }
                            ActorIdentifier::Method(method_identifier) => match method_identifier {
                                MethodIdentifier(NodeId::Object(component_address), ..) => {
                                    match (node_id, offset) {
                                        // READ package code & abi
                                        (
                                            NodeId::GlobalObject(_),
                                            SubstateKey::Package(PackageOffset::Info), // TODO: Remove
                                        )
                                        | (
                                            NodeId::GlobalObject(_),
                                            SubstateKey::Package(PackageOffset::CodeType), // TODO: Remove
                                        )
                                        | (
                                            NodeId::GlobalObject(_),
                                            SubstateKey::Package(PackageOffset::Code), // TODO: Remove
                                        ) => read_only,
                                        // READ/WRITE KVStore entry
                                        (
                                            NodeId::KeyValueStore(_),
                                            SubstateKey::KeyValueStore(KeyValueStoreOffset::Entry(
                                                ..,
                                            )),
                                        ) => true,
                                        // READ/WRITE component application state
                                        (
                                            NodeId::Object(addr),
                                            SubstateKey::Component(ComponentOffset::State0),
                                        ) => addr.eq(component_address),
                                        // Otherwise, false
                                        _ => false,
                                    }
                                }
                                MethodIdentifier(
                                    NodeId::GlobalObject(GlobalAddress::Component(
                                        component_address,
                                    )),
                                    ..,
                                ) => match (node_id, offset) {
                                    // READ package code & abi
                                    (
                                        NodeId::GlobalObject(_),
                                        SubstateKey::Package(PackageOffset::Info), // TODO: Remove
                                    )
                                    | (
                                        NodeId::GlobalObject(_),
                                        SubstateKey::Package(PackageOffset::CodeType), // TODO: Remove
                                    )
                                    | (
                                        NodeId::GlobalObject(_),
                                        SubstateKey::Package(PackageOffset::Code), // TODO: Remove
                                    ) => read_only,
                                    // READ/WRITE KVStore entry
                                    (
                                        NodeId::KeyValueStore(_),
                                        SubstateKey::KeyValueStore(KeyValueStoreOffset::Entry(..)),
                                    ) => true,
                                    // READ/WRITE component application state
                                    (
                                        NodeId::GlobalObject(GlobalAddress::Component(addr)),
                                        SubstateKey::Component(ComponentOffset::State0),
                                    ) => addr.eq(component_address),
                                    // Otherwise, false
                                    _ => false,
                                },
                                _ => false,
                            },
                        },
                    }
                } else {
                    match &actor.fn_identifier {
                        // Native
                        FnIdentifier {
                            package_address, ..
                        } if is_native_package(*package_address) => true,

                        // Scrypto
                        _ => match &actor.identifier {
                            ActorIdentifier::VirtualLazyLoad | ActorIdentifier::Function(..) => {
                                match (node_id, offset) {
                                    (
                                        NodeId::KeyValueStore(_),
                                        SubstateKey::KeyValueStore(KeyValueStoreOffset::Entry(..)),
                                    ) => true,
                                    _ => false,
                                }
                            }

                            ActorIdentifier::Method(method_identifier) => match method_identifier {
                                MethodIdentifier(NodeId::Object(component_address), ..) => {
                                    match (node_id, offset) {
                                        (
                                            NodeId::KeyValueStore(_),
                                            SubstateKey::KeyValueStore(KeyValueStoreOffset::Entry(
                                                ..,
                                            )),
                                        ) => true,
                                        (
                                            NodeId::Object(addr),
                                            SubstateKey::Component(ComponentOffset::State0),
                                        ) => addr.eq(component_address),
                                        _ => false,
                                    }
                                }
                                MethodIdentifier(
                                    NodeId::GlobalObject(GlobalAddress::Component(
                                        component_address,
                                    )),
                                    ..,
                                ) => match (node_id, offset) {
                                    (
                                        NodeId::KeyValueStore(_),
                                        SubstateKey::KeyValueStore(KeyValueStoreOffset::Entry(..)),
                                    ) => true,
                                    (
                                        NodeId::GlobalObject(GlobalAddress::Component(addr)),
                                        SubstateKey::Component(ComponentOffset::State0),
                                    ) => addr.eq(component_address),
                                    _ => false,
                                },
                                _ => false,
                            },
                        },
                    }
                }
            }
        }
    }
}

pub struct SubstateProperties;

impl SubstateProperties {
    pub fn is_persisted(offset: &SubstateKey) -> bool {
        match offset {
            SubstateKey::Component(..) => true,
            SubstateKey::Royalty(..) => true,
            SubstateKey::AccessRules(..) => true,
            SubstateKey::Package(..) => true,
            SubstateKey::ResourceManager(..) => true,
            SubstateKey::KeyValueStore(..) => true,
            SubstateKey::Vault(..) => true,
            SubstateKey::EpochManager(..) => true,
            SubstateKey::Validator(..) => true,
            SubstateKey::Bucket(..) => false,
            SubstateKey::Proof(..) => false,
            SubstateKey::Worktop(..) => false,
            SubstateKey::AuthZone(..) => false,
            SubstateKey::Clock(..) => true,
            SubstateKey::Account(..) => true,
            SubstateKey::AccessController(..) => true,
            SubstateKey::TypeInfo(..) => true,
        }
    }

    pub fn verify_can_own(
        offset: &SubstateKey,
        package_address: PackageAddress,
        blueprint_name: &str,
    ) -> Result<(), RuntimeError> {
        match (package_address, blueprint_name) {
            (RESOURCE_MANAGER_PACKAGE, BUCKET_BLUEPRINT) => match offset {
                SubstateKey::Worktop(WorktopOffset::Worktop) => Ok(()),
                _ => Err(RuntimeError::KernelError(KernelError::InvalidOwnership(
                    Box::new(InvalidOwnership(
                        offset.clone(),
                        package_address,
                        blueprint_name.to_string(),
                    )),
                ))),
            },
            (RESOURCE_MANAGER_PACKAGE, PROOF_BLUEPRINT) => match offset {
                SubstateKey::AuthZone(AuthZoneOffset::AuthZone) => Ok(()),
                _ => Err(RuntimeError::KernelError(KernelError::InvalidOwnership(
                    Box::new(InvalidOwnership(
                        offset.clone(),
                        package_address,
                        blueprint_name.to_string(),
                    )),
                ))),
            },
            _ => Ok(()),
        }
    }
}
