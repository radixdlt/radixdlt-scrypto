use super::{KernelError, RuntimeError};
use crate::model::GlobalAddressSubstate;
use crate::types::*;
use scrypto::core::{MethodIdent, ReceiverMethodIdent};

pub struct RENodeProperties;

impl RENodeProperties {
    /// Specifies whether an RENode may globalize as the root node or not
    pub fn to_global(node_id: RENodeId) -> Option<(GlobalAddress, GlobalAddressSubstate)> {
        match node_id {
            RENodeId::Global(..) => panic!("Should never get here."),
            RENodeId::Component(component_address) | RENodeId::System(component_address) => Some((
                GlobalAddress::Component(component_address),
                GlobalAddressSubstate::Component(scrypto::component::Component(component_address)),
            )),
            RENodeId::ResourceManager(resource_address) => Some((
                GlobalAddress::Resource(resource_address),
                GlobalAddressSubstate::Resource(resource_address),
            )),
            RENodeId::Package(package_address) => Some((
                GlobalAddress::Package(package_address),
                GlobalAddressSubstate::Package(package_address),
            )),
            RENodeId::AuthZone(..) => Option::None,
            RENodeId::Bucket(..) => Option::None,
            RENodeId::Proof(..) => Option::None,
            RENodeId::KeyValueStore(..) => Option::None,
            RENodeId::Worktop => Option::None,
            RENodeId::Vault(..) => Option::None,
        }
    }

    pub fn to_primary_substate_id(
        method_ident: &ReceiverMethodIdent,
    ) -> Result<SubstateId, RuntimeError> {
        let substate_id = match &method_ident.method_ident {
            MethodIdent::Native(..) => match method_ident.receiver.node_id() {
                RENodeId::AuthZone(auth_zone_id) => SubstateId(
                    RENodeId::AuthZone(auth_zone_id),
                    SubstateOffset::AuthZone(AuthZoneOffset::AuthZone),
                ),
                RENodeId::Bucket(bucket_id) => SubstateId(
                    RENodeId::Bucket(bucket_id),
                    SubstateOffset::Bucket(BucketOffset::Bucket),
                ),
                RENodeId::Proof(proof_id) => SubstateId(
                    RENodeId::Proof(proof_id),
                    SubstateOffset::Proof(ProofOffset::Proof),
                ),
                RENodeId::ResourceManager(resource_address) => SubstateId(
                    RENodeId::ResourceManager(resource_address),
                    SubstateOffset::Resource(ResourceManagerOffset::ResourceManager),
                ),
                RENodeId::System(component_address) => SubstateId(
                    RENodeId::System(component_address),
                    SubstateOffset::System(SystemOffset::System),
                ),
                RENodeId::Worktop => SubstateId(
                    RENodeId::Worktop,
                    SubstateOffset::Worktop(WorktopOffset::Worktop),
                ),
                RENodeId::Component(component_address) => SubstateId(
                    RENodeId::Component(component_address),
                    SubstateOffset::Component(ComponentOffset::Info),
                ),
                RENodeId::Vault(vault_id) => SubstateId(
                    RENodeId::Vault(vault_id),
                    SubstateOffset::Vault(VaultOffset::Vault),
                ),
                _ => {
                    return Err(RuntimeError::KernelError(KernelError::MethodNotFound(
                        method_ident.clone(),
                    )))
                }
            },
            MethodIdent::Scrypto { .. } => match method_ident.receiver.node_id() {
                RENodeId::Component(component_address) => SubstateId(
                    RENodeId::Component(component_address),
                    SubstateOffset::Component(ComponentOffset::State),
                ),
                _ => {
                    return Err(RuntimeError::KernelError(KernelError::MethodNotFound(
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
    pub fn can_own_nodes(offset: &SubstateOffset) -> bool {
        match offset {
            SubstateOffset::Global(..) => true,
            SubstateOffset::AuthZone(..) => false,
            SubstateOffset::Component(ComponentOffset::State) => true,
            SubstateOffset::Component(ComponentOffset::Info) => false,
            SubstateOffset::Resource(ResourceManagerOffset::NonFungible(..)) => false,
            SubstateOffset::Resource(ResourceManagerOffset::NonFungibleSpace) => false,
            SubstateOffset::Resource(ResourceManagerOffset::ResourceManager) => false,
            SubstateOffset::KeyValueStore(KeyValueStoreOffset::Entry(..)) => true,
            SubstateOffset::KeyValueStore(KeyValueStoreOffset::Space) => false,
            SubstateOffset::Vault(..) => false,
            SubstateOffset::Package(..) => false,
            SubstateOffset::System(..) => false,
            SubstateOffset::Bucket(..) => false,
            SubstateOffset::Proof(..) => false,
            SubstateOffset::Worktop(..) => false, // TODO: Fix
        }
    }
}
