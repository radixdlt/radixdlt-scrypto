use super::*;
use crate::types::*;

pub trait QueryableSubstateStore {
    fn list_substates(
        &self,
        node_id: &NodeId,
        module_id: ModuleId,
    ) -> HashMap<SubstateKey, OutputValue>;
}

pub trait ReadableSubstateStore {
    fn get_substate(
        &self,
        node_id: &NodeId,
        module_id: ModuleId,
        substate_key: &SubstateKey,
    ) -> Option<OutputValue>;
}

pub trait WriteableSubstateStore {
    fn put_substate(
        &mut self,
        node_id: &NodeId,
        module_id: ModuleId,
        substate_key: &SubstateKey,
        substate_value: OutputValue,
    );
}

pub trait SubstateStore: ReadableSubstateStore + WriteableSubstateStore {}

impl<T: ReadableSubstateStore + WriteableSubstateStore> SubstateStore for T {}
