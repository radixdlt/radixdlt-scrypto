use crate::errors::{IdAllocationError, KernelError, RuntimeError};
use crate::internal_prelude::*;

/// An ID allocator defines how identities are generated.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IdAllocator {
    transaction_hash: Hash,
    next_id: u32,
}

impl IdAllocator {
    pub fn new(transaction_hash: Hash) -> Self {
        Self {
            transaction_hash,
            next_id: 0u32,
        }
    }

    pub fn allocate_node_id(&mut self, entity_type: EntityType) -> Result<NodeId, RuntimeError> {
        let node_id = self
            .next_node_id(entity_type)
            .map_err(|e| RuntimeError::KernelError(KernelError::IdAllocationError(e)))?;

        Ok(node_id)
    }

    fn next(&mut self) -> Result<u32, IdAllocationError> {
        if self.next_id == u32::MAX {
            Err(IdAllocationError::OutOfID)
        } else {
            let rtn = self.next_id;
            self.next_id += 1;
            Ok(rtn)
        }
    }

    fn next_node_id(&mut self, entity_type: EntityType) -> Result<NodeId, IdAllocationError> {
        // Compute `hash(transaction_hash, index)`
        let mut buf = [0u8; Hash::LENGTH + 4];
        buf[..Hash::LENGTH].copy_from_slice(self.transaction_hash.as_ref());
        buf[Hash::LENGTH..].copy_from_slice(&self.next()?.to_le_bytes());
        let hash = hash(buf);

        // Install the entity type
        let mut node_id: [u8; NodeId::LENGTH] = hash.lower_bytes();
        node_id[0] = entity_type as u8;

        Ok(NodeId(node_id))
    }
}
