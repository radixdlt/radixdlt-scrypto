use crate::types::*;
use radix_engine_interface::api::LockFlags;
use crate::kernel::call_frame::SubstateLocation;

pub struct SubstateLockError;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Sbor)]
pub enum SubstateLockState {
    Read(usize),
    Write,
}

impl SubstateLockState {
    fn no_lock() -> Self {
        Self::Read(0)
    }

    fn is_locked(&self) -> bool {
        !matches!(self, SubstateLockState::Read(0usize))
    }

    fn try_lock(&mut self, flags: LockFlags) -> Result<(), SubstateLockError> {
        match self {
            SubstateLockState::Read(n) => {
                if flags.contains(LockFlags::MUTABLE) {
                    if *n != 0 {
                        return Err(SubstateLockError);
                    }
                    *self = SubstateLockState::Write;
                } else {
                    *n = *n + 1;
                }
            }
            SubstateLockState::Write => {
                return Err(SubstateLockError);
            }
        }

        Ok(())
    }

    fn unlock(&mut self) {
        match self {
            SubstateLockState::Read(n) => {
                *n = *n - 1;
            }
            SubstateLockState::Write => {
                *self = SubstateLockState::no_lock();
            }
        }
    }
}

pub struct SubstateLocks {
    locks: IndexMap<u32, (NodeId, PartitionNumber, SubstateKey, LockFlags, SubstateLocation)>,
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
        location: SubstateLocation,
    ) -> u32 {
        let new_lock = self.next_lock_id;
        self.locks.insert(
            new_lock,
            (*node_id, partition_num, substate_key.clone(), flags, location),
        );
        self.next_lock_id += 1;
        new_lock
    }

    pub fn is_locked(
        &self,
        node_id: &NodeId,
        partition_num: PartitionNumber,
        substate_key: &SubstateKey,
    ) -> bool {
        if let Some(state) =
            self.substate_lock_states
                .get(&(node_id.clone(), partition_num, substate_key.clone()))
        {
            state.is_locked()
        } else {
            false
        }
    }

    pub fn lock(
        &mut self,
        node_id: &NodeId,
        partition_num: PartitionNumber,
        substate_key: &SubstateKey,
        flags: LockFlags,
        location: SubstateLocation,
    ) -> Option<u32> {
        let lock_state = self
            .substate_lock_states
            .entry((node_id.clone(), partition_num, substate_key.clone()))
            .or_insert(SubstateLockState::no_lock());
        match lock_state.try_lock(flags) {
            Ok(()) => {}
            Err(_) => {
                return None;
            }
        }

        let handle = self.new_lock_handle(node_id, partition_num, substate_key, flags, location);
        Some(handle)
    }

    pub fn get(&self, handle: u32) -> &(NodeId, PartitionNumber, SubstateKey, LockFlags, SubstateLocation) {
        self.locks.get(&handle).unwrap()
    }

    pub fn unlock(&mut self, handle: u32) -> (NodeId, PartitionNumber, SubstateKey, LockFlags, SubstateLocation) {
        let (node_id, partition_num, substate_key, flags, location) = self.locks.remove(&handle).unwrap();
        let full_key = (node_id, partition_num, substate_key);
        let lock_state = self.substate_lock_states.get_mut(&full_key).unwrap();
        lock_state.unlock();
        (full_key.0, full_key.1, full_key.2, flags, location)
    }
}
