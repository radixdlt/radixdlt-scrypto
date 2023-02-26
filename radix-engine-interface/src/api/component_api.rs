use crate::api::types::*;
use crate::blueprints::resource::AccessRules;
use sbor::rust::collections::*;
use sbor::rust::vec::Vec;

pub trait ClientComponentApi<E> {
    // TODO: refine the interface
    fn new_component(
        &mut self,
        blueprint_ident: &str,
        app_states: BTreeMap<u8, Vec<u8>>,
        metadata: BTreeMap<String, String>,
    ) -> Result<ComponentId, E>;

    fn new_key_value_store(&mut self) -> Result<KeyValueStoreId, E>;

    fn globalize(
        &mut self,
        node_id: RENodeId,
        modules: (AccessRules, Option<RoyaltyConfig>),
    ) -> Result<ComponentAddress, E>;

    fn globalize_with_address(
        &mut self,
        node_id: RENodeId,
        modules: (AccessRules, Option<RoyaltyConfig>),
        address: Address,
    ) -> Result<ComponentAddress, E>;

    fn get_component_type_info(&mut self, node_id: RENodeId)
        -> Result<(PackageAddress, String), E>;

    fn call_method(
        &mut self,
        receiver: RENodeId,
        method_name: &str,
        args: Vec<u8>,
    ) -> Result<Vec<u8>, E>;

    fn call_module_method(
        &mut self,
        receiver: RENodeId,
        node_module_id: NodeModuleId,
        method_name: &str,
        args: Vec<u8>,
    ) -> Result<Vec<u8>, E>;
}
