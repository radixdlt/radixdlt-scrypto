use crate::internal_prelude::*;

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, ScryptoSbor)]
pub enum LocalRef {
    Bucket(Reference),
    Vault(Reference),
}

impl LocalRef {
    pub fn as_node_id(&self) -> &NodeId {
        match self {
            LocalRef::Bucket(id) => id.as_node_id(),
            LocalRef::Vault(id) => id.as_node_id(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum ProofError {
    /// Can't generate zero-amount or empty non-fungible set proofs.
    EmptyProofNotAllowed,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, ScryptoSbor)]
pub struct ProofMoveableSubstate {
    /// Whether movement of this proof is restricted.
    pub restricted: bool,
}

impl ProofMoveableSubstate {
    pub fn change_to_restricted(&mut self) {
        self.restricted = true;
    }
}
