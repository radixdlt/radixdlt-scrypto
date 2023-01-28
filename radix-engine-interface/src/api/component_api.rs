use crate::api::types::*;
use crate::blueprints::resource::AccessRules;
use crate::data::IndexedScryptoValue;
use sbor::rust::collections::*;
use sbor::rust::vec::Vec;

pub trait ClientComponentApi<E> {
    // TODO: refine the interface
    fn instantiate_component(
        &mut self,
        blueprint_ident: &str,
        app_states: BTreeMap<u8, Vec<u8>>,
        access_rules: AccessRules,
        royalty_config: RoyaltyConfig,
        metadata: BTreeMap<String, String>,
    ) -> Result<ComponentId, E>;

    fn globalize_component(&mut self, component_id: ComponentId) -> Result<ComponentAddress, E>;

    fn call_method(
        &mut self,
        receiver: ScryptoReceiver,
        method_name: &str,
        args: Vec<u8>,
    ) -> Result<IndexedScryptoValue, E>;
}
