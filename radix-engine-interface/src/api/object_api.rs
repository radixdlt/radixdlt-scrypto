use crate::api::types::*;
use crate::data::scrypto::model::*;
use radix_engine_common::data::scrypto::{scrypto_decode, ScryptoDecode};
use sbor::rust::collections::*;
use sbor::rust::prelude::*;
use sbor::rust::vec::Vec;
use scrypto_schema::{IterableMapSchema, KeyValueStoreSchema};

pub trait ClientIterableMapApi<E> {
    fn new_iterable_map(&mut self, schema: IterableMapSchema) -> Result<ObjectId, E>;

    fn insert_into_iterable_map(
        &mut self,
        node_id: RENodeId,
        key: Vec<u8>,
        value: Vec<u8>,
    ) -> Result<(), E>;

    fn remove_from_iterable_map(&mut self, node_id: RENodeId, key: Vec<u8>) -> Result<(), E>;

    fn remove_first_in_iterable_map(
        &mut self,
        node_id: RENodeId,
        count: u32,
    ) -> Result<Vec<(Vec<u8>, Vec<u8>)>, E>;

    fn first_in_iterable_map(
        &mut self,
        node_id: RENodeId,
        count: u32,
    ) -> Result<Vec<(Vec<u8>, Vec<u8>)>, E>;

    fn first_keys_in_iterable_map<S: ScryptoDecode>(
        &mut self,
        node_id: RENodeId,
        count: u32,
    ) -> Result<Vec<Vec<u8>>, E> {
        let keys = self
            .first_in_iterable_map(node_id, count)?
            .into_iter()
            .map(|(key, _buf)| key)
            .collect();

        Ok(keys)
    }

    fn first_typed_values_in_iterable_map<S: ScryptoDecode>(
        &mut self,
        node_id: RENodeId,
        count: u32,
    ) -> Result<Vec<S>, E> {
        let entries = self
            .first_in_iterable_map(node_id, count)?
            .into_iter()
            .map(|(_key, buf)| {
                let typed_substate: S = scrypto_decode(&buf).unwrap();
                typed_substate
            })
            .collect();

        Ok(entries)
    }
}

pub trait ClientObjectApi<E> {
    // TODO: refine the interface
    fn new_object(
        &mut self,
        blueprint_ident: &str,
        app_states: Vec<Vec<u8>>,
    ) -> Result<ObjectId, E>;

    fn get_object_info(&mut self, node_id: RENodeId) -> Result<ObjectInfo, E>;

    fn new_key_value_store(&mut self, schema: KeyValueStoreSchema) -> Result<KeyValueStoreId, E>;

    fn get_key_value_store_info(&mut self, node_id: RENodeId) -> Result<KeyValueStoreSchema, E>;

    fn globalize(
        &mut self,
        node_id: RENodeId,
        modules: BTreeMap<NodeModuleId, ObjectId>,
    ) -> Result<Address, E>;

    fn globalize_with_address(
        &mut self,
        node_id: RENodeId,
        modules: BTreeMap<NodeModuleId, ObjectId>,
        address: Address,
    ) -> Result<Address, E>;

    fn call_method(
        &mut self,
        receiver: &RENodeId,
        method_name: &str,
        args: Vec<u8>,
    ) -> Result<Vec<u8>, E>;

    fn call_module_method(
        &mut self,
        receiver: &RENodeId,
        node_module_id: NodeModuleId,
        method_name: &str,
        args: Vec<u8>,
    ) -> Result<Vec<u8>, E>;

    fn call_function(
        &mut self,
        package_address: PackageAddress,
        blueprint_name: &str,
        function_name: &str,
        args: Vec<u8>,
    ) -> Result<Vec<u8>, E>;

    fn drop_object(&mut self, node_id: RENodeId) -> Result<(), E>;
}
