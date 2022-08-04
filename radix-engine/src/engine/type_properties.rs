use scrypto::engine::types::*;

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
            RENodeId::System => true,
        }
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
            SubstateId::System => RENodeId::System,
            SubstateId::Bucket(bucket_id) => RENodeId::Bucket(*bucket_id),
            SubstateId::Proof(proof_id) => RENodeId::Proof(*proof_id),
            SubstateId::Worktop => RENodeId::Worktop,
        }
    }

    pub fn is_native(substate_id: &SubstateId) -> bool {
        match substate_id {
            SubstateId::KeyValueStoreEntry(..) => false,
            SubstateId::ComponentState(..) => false,
            SubstateId::NonFungible(..) => false,
            SubstateId::ComponentInfo(..) => true,
            SubstateId::NonFungibleSpace(..) => true,
            SubstateId::KeyValueStoreSpace(..) => true,
            SubstateId::Vault(..) => true,
            SubstateId::Package(..) => true,
            SubstateId::ResourceManager(..) => true,
            SubstateId::System => true,
            SubstateId::Bucket(..) => true,
            SubstateId::Proof(..) => true,
            SubstateId::Worktop => true,
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
            SubstateId::System => false,
            SubstateId::Bucket(..) => false,
            SubstateId::Proof(..) => false,
            SubstateId::Worktop => false, // TODO: Fix
        }
    }
}
