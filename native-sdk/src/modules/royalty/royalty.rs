use radix_engine_interface::api::node_modules::royalty::{
    ComponentRoyaltyCreateInput, COMPONENT_ROYALTY_BLUEPRINT, COMPONENT_ROYALTY_CREATE_IDENT,
};
use radix_engine_interface::api::ClientApi;
use radix_engine_interface::constants::ROYALTY_PACKAGE;
use radix_engine_interface::data::scrypto::model::Own;
use radix_engine_interface::data::scrypto::*;
use radix_engine_interface::types::RoyaltyConfig;
use sbor::rust::prelude::*;

pub struct ComponentRoyalty;

impl ComponentRoyalty {
    pub fn sys_create<Y, E: Debug + ScryptoDecode>(
        royalty_config: RoyaltyConfig,
        api: &mut Y,
    ) -> Result<Own, E>
    where
        Y: ClientApi<E>,
    {
        let rtn = api.call_function(
            ROYALTY_PACKAGE,
            COMPONENT_ROYALTY_BLUEPRINT,
            COMPONENT_ROYALTY_CREATE_IDENT,
            scrypto_encode(&ComponentRoyaltyCreateInput { royalty_config }).unwrap(),
        )?;
        let componentroyatly: Own = scrypto_decode(&rtn).unwrap();

        Ok(componentroyatly)
    }
}
