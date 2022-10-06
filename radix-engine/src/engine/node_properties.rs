use super::{KernelError, RuntimeError};
use crate::types::*;
use scrypto::core::{MethodIdent, ReceiverMethodIdent};

pub struct RENodeProperties;

impl RENodeProperties {
    pub fn to_primary_offset(
        method_ident: &ReceiverMethodIdent,
    ) -> Result<SubstateOffset, RuntimeError> {
        let offset = match &method_ident.method_ident {
            MethodIdent::Native(..) => match method_ident.receiver.node_id() {
                RENodeId::AuthZone(..) => SubstateOffset::AuthZone(AuthZoneOffset::AuthZone),
                RENodeId::Bucket(..) => SubstateOffset::Bucket(BucketOffset::Bucket),
                RENodeId::Proof(..) => SubstateOffset::Proof(ProofOffset::Proof),
                RENodeId::ResourceManager(..) => {
                    SubstateOffset::ResourceManager(ResourceManagerOffset::ResourceManager)
                }
                RENodeId::System(..) => SubstateOffset::System(SystemOffset::System),
                RENodeId::Worktop => SubstateOffset::Worktop(WorktopOffset::Worktop),
                RENodeId::Component(..) => SubstateOffset::Component(ComponentOffset::Info),
                RENodeId::Vault(..) => SubstateOffset::Vault(VaultOffset::Vault),
                _ => {
                    return Err(RuntimeError::KernelError(KernelError::MethodNotFound(
                        method_ident.clone(),
                    )))
                }
            },
            MethodIdent::Scrypto { .. } => match method_ident.receiver.node_id() {
                RENodeId::Component(..) => SubstateOffset::Component(ComponentOffset::Info),
                _ => {
                    return Err(RuntimeError::KernelError(KernelError::MethodNotFound(
                        method_ident.clone(),
                    )))
                }
            },
        };

        Ok(offset)
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
            SubstateOffset::ResourceManager(ResourceManagerOffset::NonFungible(..)) => false,
            SubstateOffset::ResourceManager(ResourceManagerOffset::NonFungibleSpace) => false,
            SubstateOffset::ResourceManager(ResourceManagerOffset::ResourceManager) => false,
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
