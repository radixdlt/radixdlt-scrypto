use super::{KernelError, RuntimeError};
use crate::model::GlobalRENode;
use crate::types::*;
use scrypto::core::{MethodFnIdent, MethodIdent};

pub struct RENodeProperties;

impl RENodeProperties {
    /// Specifies whether an RENode may globalize as the root node or not
    pub fn to_global(node_id: RENodeId) -> Option<(GlobalAddress, GlobalRENode)> {
        match node_id {
            RENodeId::Global(..) => panic!("Should never get here."),
            RENodeId::Component(component_address) | RENodeId::System(component_address) => Some((
                GlobalAddress::Component(component_address),
                GlobalRENode::Component(scrypto::component::Component(component_address)),
            )),
            RENodeId::ResourceManager(resource_address) => Some((
                GlobalAddress::Resource(resource_address),
                GlobalRENode::Resource(resource_address),
            )),
            RENodeId::Package(package_address) => Some((
                GlobalAddress::Package(package_address),
                GlobalRENode::Package(package_address),
            )),
            RENodeId::AuthZone(..) => Option::None,
            RENodeId::Bucket(..) => Option::None,
            RENodeId::Proof(..) => Option::None,
            RENodeId::KeyValueStore(..) => Option::None,
            RENodeId::Worktop => Option::None,
            RENodeId::Vault(..) => Option::None,
        }
    }

    pub fn to_primary_substate_id(method_ident: &MethodIdent) -> Result<SubstateId, RuntimeError> {
        let substate_id = match &method_ident.method_fn_ident {
            MethodFnIdent::Native(..) => match method_ident.receiver.node_id() {
                RENodeId::AuthZone(auth_zone_id) => SubstateId::AuthZone(auth_zone_id, AuthZoneOffset::AuthZone),
                RENodeId::Bucket(bucket_id) => SubstateId::Bucket(bucket_id),
                RENodeId::Proof(proof_id) => SubstateId::Proof(proof_id),
                RENodeId::ResourceManager(resource_address) => {
                    SubstateId::ResourceManager(resource_address)
                }
                RENodeId::System(component_address) => SubstateId::System(component_address),
                RENodeId::Worktop => SubstateId::Worktop,
                RENodeId::Component(component_address) => {
                    SubstateId::Component(component_address, ComponentOffset::Info)
                }
                RENodeId::Vault(vault_id) => SubstateId::Vault(vault_id),
                _ => {
                    return Err(RuntimeError::KernelError(KernelError::MethodIdentNotFound(
                        method_ident.clone(),
                    )))
                }
            },
            MethodFnIdent::Scrypto { .. } => match method_ident.receiver.node_id() {
                RENodeId::Component(component_address) => {
                    SubstateId::Component(component_address, ComponentOffset::State)
                }
                _ => {
                    return Err(RuntimeError::KernelError(KernelError::MethodIdentNotFound(
                        method_ident.clone(),
                    )))
                }
            },
        };

        Ok(substate_id)
    }
}

pub struct SubstateProperties;

impl SubstateProperties {
    pub fn get_node_id(substate_id: &SubstateId) -> RENodeId {
        match substate_id {
            SubstateId::Global(global_address) => RENodeId::Global(*global_address),
            SubstateId::Component(component_address, ..) => RENodeId::Component(*component_address),
            SubstateId::NonFungibleSpace(resource_address) => {
                RENodeId::ResourceManager(*resource_address)
            }
            SubstateId::NonFungible(resource_address, ..) => {
                RENodeId::ResourceManager(*resource_address)
            }
            SubstateId::KeyValueStoreSpace(kv_store_id) => RENodeId::KeyValueStore(*kv_store_id),
            SubstateId::KeyValueStoreEntry(kv_store_id, ..) => {
                RENodeId::KeyValueStore(*kv_store_id)
            }
            SubstateId::Vault(vault_id) => RENodeId::Vault(*vault_id),
            SubstateId::Package(package_address, PackageOffset::Package) => RENodeId::Package(*package_address),
            SubstateId::ResourceManager(resource_address) => {
                RENodeId::ResourceManager(*resource_address)
            }
            SubstateId::System(component_address) => RENodeId::System(*component_address),
            SubstateId::Bucket(bucket_id) => RENodeId::Bucket(*bucket_id),
            SubstateId::Proof(proof_id) => RENodeId::Proof(*proof_id),
            SubstateId::Worktop => RENodeId::Worktop,
            SubstateId::AuthZone(auth_zone_id, AuthZoneOffset::AuthZone) => RENodeId::AuthZone(*auth_zone_id),
        }
    }

    pub fn can_own_nodes(substate_id: &SubstateId) -> bool {
        match substate_id {
            SubstateId::Global(..) => true,
            SubstateId::AuthZone(..) => false,
            SubstateId::KeyValueStoreEntry(..) => true,
            SubstateId::Component(_, ComponentOffset::State) => true,
            SubstateId::Component(_, ComponentOffset::Info) => false,
            SubstateId::NonFungible(..) => false,
            SubstateId::NonFungibleSpace(..) => false,
            SubstateId::KeyValueStoreSpace(..) => false,
            SubstateId::Vault(..) => false,
            SubstateId::Package(..) => false,
            SubstateId::ResourceManager(..) => false,
            SubstateId::System(..) => false,
            SubstateId::Bucket(..) => false,
            SubstateId::Proof(..) => false,
            SubstateId::Worktop => false, // TODO: Fix
        }
    }
}
