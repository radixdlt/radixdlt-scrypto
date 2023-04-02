use crate::{system::node_substates::PersistedSubstate, types::*};
use crate::system::node_substates::RuntimeSubstate;

pub trait QueryableSubstateStore {
    /// Returns the entire key-value store, as a map from key's entry to value's substate.
    /// NOTE: This function is supposed to be used only for test assert purposes - the
    /// implementations are sub-optimal, and the returned hashmap is inherently non-deterministic.
    fn get_kv_store_entries(
        &self,
        kv_store_id: &KeyValueStoreId,
    ) -> HashMap<Vec<u8>, PersistedSubstate>;
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord, ScryptoSbor)]
pub struct OutputId {
    pub substate_id: SubstateId,
    pub substate_hash: Hash,
    pub version: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub struct OutputValue {
    pub substate: PersistedSubstate,
    pub version: u32,
}

pub trait ReadableSubstateStore {
    fn get_substate(&self, substate_id: &SubstateId) -> Option<OutputValue>;

    fn first_in_iterable(&self, node_id: &RENodeId, module_id: NodeModuleId, count: u32) -> Vec<(SubstateId, RuntimeSubstate)>;
}

pub trait WriteableSubstateStore {
    fn put_substate(&mut self, substate_id: SubstateId, substate: OutputValue);
}

pub trait SubstateStore: ReadableSubstateStore + WriteableSubstateStore {}

impl<T: ReadableSubstateStore + WriteableSubstateStore> SubstateStore for T {}
