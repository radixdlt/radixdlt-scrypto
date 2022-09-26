use super::{KernelError, RuntimeError};
use crate::types::*;

pub struct RENodeProperties;

impl RENodeProperties {
    /// Specifies whether an RENode may globalize as the root node or not
    pub fn can_globalize(node_id: RENodeId) -> bool {
        match node_id {
            RENodeId::Bucket(..) => false,
            RENodeId::Proof(..) => false,
            RENodeId::KeyValueStore(..) => false,
            RENodeId::Worktop => false,
            RENodeId::Component(..) => true,
            RENodeId::Vault(..) => false,
            RENodeId::ResourceManager(..) => true,
            RENodeId::Package(..) => true,
            RENodeId::System(..) => true,
        }
    }

    pub fn to_primary_substate_id(
        function: &FnIdentifier,
        node_id: RENodeId,
    ) -> Result<SubstateId, RuntimeError> {
        let substate_id = match function {
            FnIdentifier::Native(..) => match node_id {
                RENodeId::Bucket(bucket_id) => SubstateId::Bucket(bucket_id),
                RENodeId::Proof(proof_id) => SubstateId::Proof(proof_id),
                RENodeId::ResourceManager(resource_address) => {
                    SubstateId::ResourceManager(resource_address)
                }
                RENodeId::System(component_address) => SubstateId::System(component_address),
                RENodeId::Worktop => SubstateId::Worktop,
                RENodeId::Component(component_address) => {
                    SubstateId::ComponentInfo(component_address)
                }
                RENodeId::Vault(vault_id) => SubstateId::Vault(vault_id),
                _ => {
                    return Err(RuntimeError::KernelError(KernelError::MethodNotFound(
                        function.clone(),
                    )))
                }
            },
            FnIdentifier::Scrypto { .. } => match node_id {
                RENodeId::Component(component_address) => {
                    SubstateId::ComponentState(component_address)
                }
                _ => {
                    return Err(RuntimeError::KernelError(KernelError::MethodNotFound(
                        function.clone(),
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
            SubstateId::ComponentInfo(component_address, ..) => {
                RENodeId::Component(*component_address)
            }
            SubstateId::ComponentState(component_address) => {
                RENodeId::Component(*component_address)
            }
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
            SubstateId::Package(package_address) => RENodeId::Package(*package_address),
            SubstateId::ResourceManager(resource_address) => {
                RENodeId::ResourceManager(*resource_address)
            }
            SubstateId::System(component_address) => RENodeId::System(*component_address),
            SubstateId::Bucket(bucket_id) => RENodeId::Bucket(*bucket_id),
            SubstateId::Proof(proof_id) => RENodeId::Proof(*proof_id),
            SubstateId::Worktop => RENodeId::Worktop,
        }
    }

    pub fn can_own_nodes(substate_id: &SubstateId) -> bool {
        match substate_id {
            SubstateId::KeyValueStoreEntry(..) => true,
            SubstateId::ComponentState(..) => true,
            SubstateId::ComponentInfo(..) => false,
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
