use crate::internal_prelude::*;
use radix_engine_interface::types::*;
use radix_substate_store_interface::interface::*;

#[derive(Debug)]
pub enum KernelDatabaseCheckError {
    DecodeError(DecodeError),
    MultipleOwnersOfNode(NodeId),
    NonGlobalReference(NodeId),
    NoOwnerForNonGlobalNode(NodeId),
    ZeroPartitionCount(NodeId),
}

pub enum NodeCheckerState {
    NoOwner(u8),
    OwnedBy(NodeId, u8),
}

pub struct KernelDatabaseChecker;

impl KernelDatabaseChecker {
    pub fn new() -> KernelDatabaseChecker {
        KernelDatabaseChecker
    }
}

impl KernelDatabaseChecker {
    pub fn check_db<S: SubstateDatabase + ListableSubstateDatabase>(
        &mut self,
        substate_db: &S,
    ) -> Result<(), KernelDatabaseCheckError> {
        let mut internal_nodes = BTreeMap::new();

        for (node_id, partition_number) in substate_db.read_partition_keys() {
            let state = internal_nodes
                .entry(node_id)
                .or_insert(NodeCheckerState::NoOwner(0u8));
            match state {
                NodeCheckerState::NoOwner(partition_count)
                | NodeCheckerState::OwnedBy(_, partition_count) => {
                    *partition_count = partition_count.checked_add(1).unwrap()
                }
            }

            for (_, value) in
                substate_db.list_raw_values(node_id, partition_number, None::<SubstateKey>)
            {
                let value = IndexedScryptoValue::from_vec(value)
                    .map_err(KernelDatabaseCheckError::DecodeError)?;
                for owned in value.owned_nodes() {
                    let state = internal_nodes
                        .entry(*owned)
                        .or_insert(NodeCheckerState::NoOwner(0u8));
                    match state {
                        NodeCheckerState::NoOwner(partition_count) => {
                            *state = NodeCheckerState::OwnedBy(node_id, *partition_count);
                        }
                        NodeCheckerState::OwnedBy(..) => {
                            return Err(KernelDatabaseCheckError::MultipleOwnersOfNode(*owned))
                        }
                    }
                }

                for refed in value.references() {
                    if !refed.is_global() {
                        return Err(KernelDatabaseCheckError::NonGlobalReference(*refed));
                    }
                    internal_nodes
                        .entry(*refed)
                        .or_insert(NodeCheckerState::NoOwner(0u8));
                }
            }
        }

        for (node_id, state) in internal_nodes {
            match state {
                NodeCheckerState::NoOwner(partition_count) => {
                    if !node_id.is_global() {
                        return Err(KernelDatabaseCheckError::NoOwnerForNonGlobalNode(node_id));
                    }

                    if partition_count == 0u8 {
                        return Err(KernelDatabaseCheckError::ZeroPartitionCount(node_id));
                    }
                }
                NodeCheckerState::OwnedBy(_, partition_count) => {
                    if partition_count == 0u8 {
                        return Err(KernelDatabaseCheckError::ZeroPartitionCount(node_id));
                    }
                }
            }
        }

        Ok(())
    }
}
