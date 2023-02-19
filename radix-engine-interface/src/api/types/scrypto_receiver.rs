use crate::api::types::*;
use crate::*;

#[derive(Debug, Copy, Clone, Eq, PartialEq, ScryptoSbor)]
pub enum ScryptoReceiver {
    Global(ComponentAddress),
    Resource(ResourceAddress),
    Vault(VaultId),
    Component(ComponentId),
    Proof(ProofId),
    Bucket(BucketId),
    Worktop,
    Logger,
    TransactionRuntime,
    AuthZoneStack,
}
