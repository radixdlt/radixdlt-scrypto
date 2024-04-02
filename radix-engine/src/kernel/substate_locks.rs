use crate::internal_prelude::*;

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

    fn try_lock(&mut self, read_only: bool) -> Result<(), SubstateLockError> {
        match self {
            SubstateLockState::Read(n) => {
                if read_only {
                    *n = *n + 1;
                } else {
                    if *n != 0 {
                        return Err(SubstateLockError);
                    }
                    *self = SubstateLockState::Write;
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

pub struct SubstateLocks<D> {
    locks: IndexMap<u32, (NodeId, PartitionNumber, SubstateKey, D)>,
    substate_lock_states: NonIterMap<(NodeId, PartitionNumber, SubstateKey), SubstateLockState>,
    node_num_locked: NonIterMap<NodeId, usize>,
    next_lock_id: u32,
}

impl<D> SubstateLocks<D> {
    pub fn new() -> Self {
        Self {
            locks: index_map_new(),
            substate_lock_states: NonIterMap::new(),
            node_num_locked: NonIterMap::new(),
            next_lock_id: 0u32,
        }
    }

    fn new_lock_handle(
        &mut self,
        node_id: &NodeId,
        partition_num: PartitionNumber,
        substate_key: &SubstateKey,
        data: D,
    ) -> u32 {
        let new_lock = self.next_lock_id;
        self.locks.insert(
            new_lock,
            (*node_id, partition_num, substate_key.clone(), data),
        );
        self.next_lock_id += 1;
        new_lock
    }

    pub fn node_is_locked(&self, node_id: &NodeId) -> bool {
        self.node_num_locked
            .get(node_id)
            .map(|e| *e > 0)
            .unwrap_or(false)
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
        read_only: bool,
        data: D,
    ) -> Option<u32> {
        let lock_state = self
            .substate_lock_states
            .entry((node_id.clone(), partition_num, substate_key.clone()))
            .or_insert(SubstateLockState::no_lock());
        match lock_state.try_lock(read_only) {
            Ok(()) => {}
            Err(_) => {
                return None;
            }
        }

        let count = self.node_num_locked.entry(*node_id).or_insert(0);
        *count = *count + 1;

        let handle = self.new_lock_handle(node_id, partition_num, substate_key, data);
        Some(handle)
    }

    pub fn get(&self, handle: u32) -> &(NodeId, PartitionNumber, SubstateKey, D) {
        self.locks.get(&handle).unwrap()
    }

    pub fn get_mut(&mut self, handle: u32) -> &mut (NodeId, PartitionNumber, SubstateKey, D) {
        self.locks.get_mut(&handle).unwrap()
    }

    pub fn unlock(&mut self, handle: u32) -> (NodeId, PartitionNumber, SubstateKey, D) {
        let (node_id, partition_num, substate_key, data) = self.locks.swap_remove(&handle).unwrap();
        let full_key = (node_id, partition_num, substate_key);

        let lock_state = self.substate_lock_states.get_mut(&full_key).unwrap();
        lock_state.unlock();

        let count = self.node_num_locked.entry(node_id).or_insert(0);
        *count = *count - 1;

        (full_key.0, full_key.1, full_key.2, data)
    }
}
