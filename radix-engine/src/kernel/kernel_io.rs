use crate::kernel::heap::Heap;
use crate::kernel::substate_locks::SubstateLocks;
use crate::track::interface::SubstateStore;

pub struct SubstateIO<'g, S: SubstateStore> {
    /// Heap
    pub heap: Heap,
    /// Store
    pub store: &'g mut S,

    pub substate_locks: SubstateLocks,
}
