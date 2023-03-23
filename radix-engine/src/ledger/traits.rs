use crate::{system::node_substates::PersistedSubstate, types::*};

pub trait QueryableSubstateStore {
    fn get_kv_store_entries(
        &self,
        kv_store_id: &KeyValueStoreId,
    ) -> HashMap<Vec<u8>, PersistedSubstate>;
}

pub trait ReadableSubstateStore {
    fn get_substate(&self, substate_id: &SubstateId) -> Option<OutputValue>;
}

pub trait WriteableSubstateStore {
    fn put_substate(&mut self, substate_id: SubstateId, substate: OutputValue);
}

pub trait SubstateStore: ReadableSubstateStore + WriteableSubstateStore {}

impl<T: ReadableSubstateStore + WriteableSubstateStore> SubstateStore for T {}
