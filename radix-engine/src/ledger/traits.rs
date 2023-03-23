use super::*;
use crate::types::*;

pub trait QueryableSubstateStore {
    fn list_substates(
        &self,
        node_id: &NodeId,
        module_id: ModuleId,
    ) -> BTreeMap<SubstateKey, IndexedScryptoValue>;
}

pub trait ReadableSubstateStore {
    fn get_substate(
        &self,
        node_id: &NodeId,
        module_id: ModuleId,
        substate_key: &SubstateKey,
    ) -> Option<IndexedScryptoValue>;
}

pub trait WriteableSubstateStore {
    fn put_substate(
        &mut self,
        node_id: &NodeId,
        module_id: ModuleId,
        substate_key: &SubstateKey,
        substate_value: IndexedScryptoValue,
    );
}

pub trait SubstateStore: ReadableSubstateStore + WriteableSubstateStore {}

impl<T: ReadableSubstateStore + WriteableSubstateStore> SubstateStore for T {}
