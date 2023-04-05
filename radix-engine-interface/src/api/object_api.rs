use crate::types::*;
use radix_engine_common::types::*;
use sbor::rust::collections::*;
use sbor::rust::prelude::*;
use sbor::rust::vec::Vec;
use scrypto_schema::KeyValueStoreSchema;

pub trait ClientObjectApi<E> {
    // TODO: refine the interface
    fn new_object(
        &mut self,
        blueprint_ident: &str,
        object_states: Vec<Vec<u8>>,
    ) -> Result<NodeId, E>;

    fn new_key_value_store(&mut self, schema: KeyValueStoreSchema) -> Result<NodeId, E>;

    fn get_object_info(&mut self, node_id: &NodeId) -> Result<ObjectInfo, E>;

    fn get_key_value_store_info(&mut self, node_id: &NodeId) -> Result<KeyValueStoreSchema, E>;

    fn globalize(
        &mut self,
        node_id: NodeId,
        modules: BTreeMap<TypedModuleId, NodeId>,
    ) -> Result<GlobalAddress, E>;

    fn globalize_with_address(
        &mut self,
        node_id: NodeId,
        modules: BTreeMap<TypedModuleId, NodeId>,
        address: GlobalAddress,
    ) -> Result<(), E>;

    fn call_method(
        &mut self,
        receiver: &NodeId,
        method_name: &str,
        args: Vec<u8>,
    ) -> Result<Vec<u8>, E>;

    fn call_module_method(
        &mut self,
        receiver: &NodeId,
        module_id: TypedModuleId,
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

    fn drop_object(&mut self, node_id: NodeId) -> Result<(), E>;
}
