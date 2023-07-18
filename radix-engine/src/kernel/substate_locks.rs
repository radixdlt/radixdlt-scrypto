use crate::types::*;
use radix_engine_interface::api::LockFlags;
use crate::track::SubstateLockState;

pub struct SubstateLocks {
    locks: IndexMap<u32, (NodeId, PartitionNumber, SubstateKey, LockFlags)>,
    substate_lock_states: NonIterMap<(NodeId, PartitionNumber, SubstateKey), SubstateLockState>,
    next_lock_id: u32,
}

impl SubstateLocks {
    pub fn new() -> Self {
        Self {
            locks: IndexMap::new(),
            substate_lock_states: NonIterMap::new(),
            next_lock_id: 0u32,
        }
    }

    fn new_lock_handle(
        &mut self,
        node_id: &NodeId,
        partition_num: PartitionNumber,
        substate_key: &SubstateKey,
        flags: LockFlags,
    ) -> u32 {
        let new_lock = self.next_lock_id;
        self.locks.insert(
            new_lock,
            (*node_id, partition_num, substate_key.clone(), flags),
        );
        self.next_lock_id += 1;
        new_lock
    }

    pub fn lock(
        &mut self,
        node_id: &NodeId,
        partition_num: PartitionNumber,
        substate_key: &SubstateKey,
        flags: LockFlags,
    ) -> Option<u32> {
        let lock_state = self.substate_lock_states.entry((node_id.clone(), partition_num, substate_key.clone()))
            .or_insert(SubstateLockState::Read(0));
        match lock_state.try_lock(flags) {
            Ok(()) => {},
            Err(_) => {
                return None;
            }
        }

        let handle = self.new_lock_handle(node_id, partition_num, substate_key, flags);
        Some(handle)
    }

    pub fn get(&self, handle: u32) -> &(NodeId, PartitionNumber, SubstateKey, LockFlags) {
        self.locks.get(&handle).unwrap()
    }

    pub fn unlock(&mut self, handle: u32) -> (NodeId, PartitionNumber, SubstateKey) {
        let (node_id, partition_num, substate_key, _flags) = self.locks.remove(&handle).unwrap();
        let full_key = (node_id, partition_num, substate_key);
        let lock_state = self.substate_lock_states.get_mut(&full_key).unwrap();
        lock_state.unlock();
        full_key
    }
}