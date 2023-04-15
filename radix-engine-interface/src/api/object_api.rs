use crate::types::*;
use radix_engine_common::data::scrypto::{
    scrypto_decode, scrypto_encode, ScryptoDecode, ScryptoEncode,
};
use radix_engine_common::types::*;
use radix_engine_derive::{ManifestSbor, ScryptoEncode, ScryptoSbor};
use sbor::rust::collections::*;
use sbor::rust::prelude::*;
use sbor::rust::vec::Vec;
use scrypto_schema::KeyValueStoreSchema;

#[repr(u8)]
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    PartialOrd,
    Ord,
    ScryptoSbor,
    ManifestSbor,
    FromRepr,
    EnumIter,
)]
pub enum ObjectModuleId {
    SELF,
    Metadata,
    Royalty,
    AccessRules,
}

/// A high level interface to manipulate objects in the actor's call frame
pub trait ClientObjectApi<E> {
    // TODO: refine the interface
    /// Creates a new object of a given blueprint type
    fn new_object(
        &mut self,
        blueprint_ident: &str,
        object_states: Vec<Vec<u8>>,
    ) -> Result<NodeId, E>;

    /// Get info regarding a visible object
    fn get_object_info(&mut self, node_id: &NodeId) -> Result<ObjectInfo, E>;

    /// Creates a new key value store with a given schema
    fn new_key_value_store(&mut self, schema: KeyValueStoreSchema) -> Result<NodeId, E>;

    /// Get info regarding a visible key value store
    fn get_key_value_store_info(&mut self, node_id: &NodeId) -> Result<KeyValueStoreSchema, E>;

    /// Moves an object currently in the heap into the global space making
    /// it accessible to all. A global address is automatically created and returned.
    fn globalize(&mut self, modules: BTreeMap<ObjectModuleId, NodeId>) -> Result<GlobalAddress, E>;

    /// Moves an object currently in the heap into the global space making
    /// it accessible to all with the provided global address.
    fn globalize_with_address(
        &mut self,
        modules: BTreeMap<ObjectModuleId, NodeId>,
        address: GlobalAddress,
    ) -> Result<(), E>;

    /// Calls a method on an object
    fn call_method(
        &mut self,
        receiver: &NodeId,
        method_name: &str,
        args: Vec<u8>,
    ) -> Result<Vec<u8>, E>;

    // TODO: Add Object Module logic
    /// Calls a method on an object module
    fn call_module_method(
        &mut self,
        receiver: &NodeId,
        module_id: ObjectModuleId,
        method_name: &str,
        args: Vec<u8>,
    ) -> Result<Vec<u8>, E>;

    /// Drops an object
    fn drop_object(&mut self, node_id: NodeId) -> Result<(), E>;
}

#[derive(Clone, Debug, Eq, PartialEq, ScryptoSbor, ManifestSbor)]
pub struct SortedKey(u16, Vec<u8>);

impl SortedKey {
    pub fn new(sorted: u16, key: Vec<u8>) -> Self {
        Self(sorted, key)
    }
}

impl Into<Vec<u8>> for SortedKey {
    fn into(self) -> Vec<u8> {
        let mut bytes = self.0.to_be_bytes().to_vec();
        bytes.extend(self.1);
        bytes
    }
}

pub trait ClientSortedApi<E> {
    /// Creates a new key value store with a given schema
    fn new_sorted(&mut self) -> Result<NodeId, E>;

    fn insert_into_sorted(
        &mut self,
        node_id: &NodeId,
        sorted_key: SortedKey,
        buffer: Vec<u8>,
    ) -> Result<(), E>;

    fn insert_typed_into_sorted<V: ScryptoEncode>(
        &mut self,
        node_id: &NodeId,
        sorted_key: SortedKey,
        value: V,
    ) -> Result<(), E> {
        self.insert_into_sorted(node_id, sorted_key, scrypto_encode(&value).unwrap())
    }

    fn remove_from_sorted(
        &mut self,
        node_id: &NodeId,
        sorted_key: &SortedKey,
    ) -> Result<Option<Vec<u8>>, E>;

    fn remove_typed_from_sorted<V: ScryptoDecode>(
        &mut self,
        node_id: &NodeId,
        sorted_key: &SortedKey,
    ) -> Result<Option<V>, E> {
        let rtn = self
            .remove_from_sorted(node_id, sorted_key)?
            .map(|e| scrypto_decode(&e).unwrap());
        Ok(rtn)
    }

    fn read_from_sorted(&mut self, node_id: &NodeId, count: u32) -> Result<Vec<Vec<u8>>, E>;

    fn read_typed_from_sorted<S: ScryptoDecode>(
        &mut self,
        node_id: &NodeId,
        count: u32,
    ) -> Result<Vec<S>, E> {
        let entries = self
            .read_from_sorted(node_id, count)?
            .into_iter()
            .map(|buf| {
                let typed: S = scrypto_decode(&buf).unwrap();
                typed
            })
            .collect();

        Ok(entries)
    }
}
