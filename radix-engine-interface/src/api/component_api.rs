use crate::api::types::*;
use crate::blueprints::resource::AccessRules;
use sbor::rust::collections::*;
use sbor::rust::vec::Vec;

pub trait ClientComponentApi<E> {
    // For the time being, this is to replace ClientDerefApi, by not  changing all other methods here
    // to accept both ComponentId and ComponentAddress.
    // On the long run, will update all methods to accept `handle: u32`, so this method will not be needed then.

    fn lookup_global_component(
        &mut self,
        component_address: ComponentAddress,
    ) -> Result<ComponentId, E>;

    // TODO: refine the interface
    fn new_component(
        &mut self,
        blueprint_ident: &str,
        app_states: BTreeMap<u8, Vec<u8>>,
        access_rules_chain: Vec<AccessRules>,
        royalty_config: RoyaltyConfig,
        metadata: BTreeMap<String, String>,
    ) -> Result<ComponentId, E>;

    fn new_key_value_store(&mut self) -> Result<KeyValueStoreId, E>;

    fn globalize(&mut self, node_id: RENodeId) -> Result<ComponentAddress, E>;

    fn globalize_with_address(&mut self, node_id: RENodeId, address: Address) -> Result<ComponentAddress, E>;

    fn get_component_type_info(
        &mut self,
        component_id: ComponentId,
    ) -> Result<(PackageAddress, String), E>;

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
