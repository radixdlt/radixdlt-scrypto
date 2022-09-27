use crate::engine::Substate;
use crate::types::*;

pub trait QueryableSubstateStore {
    fn get_kv_store_entries(&self, kv_store_id: &KeyValueStoreId) -> HashMap<Vec<u8>, Substate>;
}

#[derive(Debug, Clone, Hash, TypeId, Encode, Decode, PartialEq, Eq)]
pub struct OutputId {
    pub substate_id: SubstateId,
    pub substate_hash: Hash,
    pub version: u32,
}

#[derive(Debug, Clone, Encode, Decode, TypeId, PartialEq, Eq)]
pub struct OutputValue {
    pub substate: Substate,
    pub version: u32,
}

pub trait ReadableSubstateStore {
    fn get_substate(&self, substate_id: &SubstateId) -> Option<OutputValue>;
}

pub trait WriteableSubstateStore {
    fn put_substate(&mut self, substate_id: SubstateId, substate: OutputValue);
}

pub trait SubstateStore: ReadableSubstateStore + WriteableSubstateStore {}

impl<T: ReadableSubstateStore + WriteableSubstateStore> SubstateStore for T {}
