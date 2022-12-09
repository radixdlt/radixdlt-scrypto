use crate::api::types::VaultId;
use crate::scrypto;
use sbor::*;

#[scrypto(TypeId, Encode, Decode)]
pub enum Ownership {
    Vault(VaultId),
    // TODO: add more
}
