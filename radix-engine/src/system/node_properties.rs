use crate::errors::{KernelError, RuntimeError};
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
        node_id: RENodeId,
        offset: SubstateOffset,
        flags: LockFlags,
    ) -> bool {
        let read_only = flags == LockFlags::read_only();

        // TODO: Cleanup and reduce to least privilege
        match (mode, offset) {
            (ExecutionMode::Kernel, offset) => match offset {
                SubstateOffset::TypeInfo(TypeInfoOffset::TypeInfo) => true,
                _ => false, // Protect ourselves!
            },
            (ExecutionMode::Resolver, offset) => match offset {
                SubstateOffset::TypeInfo(TypeInfoOffset::TypeInfo) => read_only,
                SubstateOffset::Package(PackageOffset::CodeType) => read_only,
                SubstateOffset::Package(PackageOffset::Info) => read_only,
                SubstateOffset::Bucket(BucketOffset::Info) => read_only,
                _ => false,
            },
            (ExecutionMode::AutoDrop, offset) => match offset {
                SubstateOffset::TypeInfo(TypeInfoOffset::TypeInfo) => true,
                _ => false,
            },
            (ExecutionMode::DropNode, offset) => match offset {
                SubstateOffset::TypeInfo(TypeInfoOffset::TypeInfo) => true,
                SubstateOffset::Bucket(BucketOffset::Info) => true,
                SubstateOffset::Proof(ProofOffset::Info) => true,
                SubstateOffset::Proof(..) => true,
                SubstateOffset::Worktop(WorktopOffset::Worktop) => true,
                _ => false,
            },
            (ExecutionMode::KernelModule, offset) => match offset {
                // TODO: refine based on specific module
                SubstateOffset::ResourceManager(ResourceManagerOffset::ResourceManager) => {
                    read_only
                }
                SubstateOffset::Vault(..) => true,
                SubstateOffset::Bucket(..) => read_only,
                SubstateOffset::Proof(..) => true,
                SubstateOffset::Package(PackageOffset::Info) => read_only,
                SubstateOffset::Package(PackageOffset::CodeType) => read_only,
                SubstateOffset::Package(PackageOffset::Code) => read_only,
                SubstateOffset::Package(PackageOffset::Royalty) => true,
                SubstateOffset::Package(PackageOffset::FunctionAccessRules) => true,
                SubstateOffset::Component(ComponentOffset::State0) => read_only,
                SubstateOffset::TypeInfo(_) => read_only,
                SubstateOffset::AccessRules(_) => read_only,
                SubstateOffset::AuthZone(_) => read_only,
                SubstateOffset::Royalty(_) => true,
                _ => false,
            },
            (ExecutionMode::Client, offset) => {
                if !flags.contains(LockFlags::MUTABLE) {
                    if matches!(offset, SubstateOffset::TypeInfo(TypeInfoOffset::TypeInfo)) {
                        return true;
                    }

                    match &actor.fn_identifier {
                        // Native
                        FnIdentifier {
                            package_address, ..
                        } if is_native_package(*package_address) => true,
                        // Scrypto
                        _ => match &actor.identifier {
                            ActorIdentifier::Function(..) => match (node_id, offset) {
                                // READ package code & abi
                                (
                                    RENodeId::GlobalObject(_),
                                    SubstateOffset::Package(PackageOffset::Info), // TODO: Remove
                                )
                                | (
                                    RENodeId::GlobalObject(_),
                                    SubstateOffset::Package(PackageOffset::CodeType), // TODO: Remove
                                )
                                | (
                                    RENodeId::GlobalObject(_),
                                    SubstateOffset::Package(PackageOffset::Code), // TODO: Remove
                                )
                                | (
                                    RENodeId::GlobalObject(_),
                                    SubstateOffset::Package(PackageOffset::EventSchema), // TODO: Remove
                                ) => read_only,
                                // READ global substates
                                (
                                    RENodeId::Object(_),
                                    SubstateOffset::TypeInfo(TypeInfoOffset::TypeInfo),
                                ) => read_only,
                                // READ/WRITE KVStore entry
                                (
                                    RENodeId::KeyValueStore(_),
                                    SubstateOffset::KeyValueStore(KeyValueStoreOffset::Entry(..)),
                                ) => true,
                                // Otherwise, false
                                _ => false,
                            },
                            ActorIdentifier::Method(method_identifier) => match method_identifier {
                                MethodIdentifier(RENodeId::Object(component_address), ..) => {
                                    match (node_id, offset) {
                                        // READ package code & abi
                                        (
                                            RENodeId::GlobalObject(_),
                                            SubstateOffset::Package(PackageOffset::Info), // TODO: Remove
                                        )
                                        | (
                                            RENodeId::GlobalObject(_),
                                            SubstateOffset::Package(PackageOffset::CodeType), // TODO: Remove
                                        )
                                        | (
                                            RENodeId::GlobalObject(_),
                                            SubstateOffset::Package(PackageOffset::Code), // TODO: Remove
                                        )
                                        | (
                                            RENodeId::GlobalObject(_),
                                            SubstateOffset::Package(PackageOffset::EventSchema), // TODO: Remove
                                        ) => read_only,
                                        // READ/WRITE KVStore entry
                                        (
                                            RENodeId::KeyValueStore(_),
                                            SubstateOffset::KeyValueStore(
                                                KeyValueStoreOffset::Entry(..),
                                            ),
                                        ) => true,
                                        // READ/WRITE component application state
                                        (
                                            RENodeId::Object(addr),
                                            SubstateOffset::Component(ComponentOffset::State0),
                                        ) => addr.eq(component_address),
                                        // Otherwise, false
                                        _ => false,
                                    }
                                }
                                MethodIdentifier(
                                    RENodeId::GlobalObject(Address::Component(component_address)),
                                    ..,
                                ) => match (node_id, offset) {
                                    // READ package code & abi
                                    (
                                        RENodeId::GlobalObject(_),
                                        SubstateOffset::Package(PackageOffset::Info), // TODO: Remove
                                    )
                                    | (
                                        RENodeId::GlobalObject(_),
                                        SubstateOffset::Package(PackageOffset::CodeType), // TODO: Remove
                                    )
                                    | (
                                        RENodeId::GlobalObject(_),
                                        SubstateOffset::Package(PackageOffset::Code), // TODO: Remove
                                    ) => read_only,
                                    // READ/WRITE KVStore entry
                                    (
                                        RENodeId::KeyValueStore(_),
                                        SubstateOffset::KeyValueStore(KeyValueStoreOffset::Entry(
                                            ..,
                                        )),
                                    ) => true,
                                    // READ/WRITE component application state
                                    (
                                        RENodeId::GlobalObject(Address::Component(addr)),
                                        SubstateOffset::Component(ComponentOffset::State0),
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
                            ActorIdentifier::Function(..) => match (node_id, offset) {
                                (
                                    RENodeId::KeyValueStore(_),
                                    SubstateOffset::KeyValueStore(KeyValueStoreOffset::Entry(..)),
                                ) => true,
                                _ => false,
                            },

                            ActorIdentifier::Method(method_identifier) => match method_identifier {
                                MethodIdentifier(RENodeId::Object(component_address), ..) => {
                                    match (node_id, offset) {
                                        (
                                            RENodeId::KeyValueStore(_),
                                            SubstateOffset::KeyValueStore(
                                                KeyValueStoreOffset::Entry(..),
                                            ),
                                        ) => true,
                                        (
                                            RENodeId::Object(addr),
                                            SubstateOffset::Component(ComponentOffset::State0),
                                        ) => addr.eq(component_address),
                                        _ => false,
                                    }
                                }
                                MethodIdentifier(
                                    RENodeId::GlobalObject(Address::Component(component_address)),
                                    ..,
                                ) => match (node_id, offset) {
                                    (
                                        RENodeId::KeyValueStore(_),
                                        SubstateOffset::KeyValueStore(KeyValueStoreOffset::Entry(
                                            ..,
                                        )),
                                    ) => true,
                                    (
                                        RENodeId::GlobalObject(Address::Component(addr)),
                                        SubstateOffset::Component(ComponentOffset::State0),
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
    pub fn is_persisted(offset: &SubstateOffset) -> bool {
        match offset {
            SubstateOffset::Component(..) => true,
            SubstateOffset::Royalty(..) => true,
            SubstateOffset::AccessRules(..) => true,
            SubstateOffset::Package(..) => true,
            SubstateOffset::ResourceManager(..) => true,
            SubstateOffset::KeyValueStore(..) => true,
            SubstateOffset::Vault(..) => true,
            SubstateOffset::EpochManager(..) => true,
            SubstateOffset::Validator(..) => true,
            SubstateOffset::Bucket(..) => false,
            SubstateOffset::Proof(..) => false,
            SubstateOffset::Worktop(..) => false,
            SubstateOffset::AuthZone(..) => false,
            SubstateOffset::Clock(..) => true,
            SubstateOffset::Account(..) => true,
            SubstateOffset::AccessController(..) => true,
            SubstateOffset::TypeInfo(..) => true,
        }
    }

    pub fn verify_can_own(
        offset: &SubstateOffset,
        package_address: PackageAddress,
        blueprint_name: &str,
    ) -> Result<(), RuntimeError> {
        match (package_address, blueprint_name) {
            (RESOURCE_MANAGER_PACKAGE, BUCKET_BLUEPRINT) => match offset {
                SubstateOffset::Worktop(WorktopOffset::Worktop) => Ok(()),
                _ => Err(RuntimeError::KernelError(KernelError::InvalidOwnership(
                    offset.clone(),
                    package_address,
                    blueprint_name.to_string(),
                ))),
            },
            (RESOURCE_MANAGER_PACKAGE, PROOF_BLUEPRINT) => match offset {
                SubstateOffset::AuthZone(AuthZoneOffset::AuthZone) => Ok(()),
                _ => Err(RuntimeError::KernelError(KernelError::InvalidOwnership(
                    offset.clone(),
                    package_address,
                    blueprint_name.to_string(),
                ))),
            },
            _ => Ok(()),
        }
    }
}
