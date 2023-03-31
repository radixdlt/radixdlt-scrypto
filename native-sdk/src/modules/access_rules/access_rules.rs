use radix_engine_interface::api::node_modules::auth::{
    AccessRulesCreateInput, ACCESS_RULES_BLUEPRINT, ACCESS_RULES_CREATE_IDENT,
};
use radix_engine_interface::api::ClientApi;
use radix_engine_interface::blueprints::resource::AccessRulesConfig;
use radix_engine_interface::constants::ACCESS_RULES_PACKAGE;
use radix_engine_interface::data::scrypto::model::Own;
use radix_engine_interface::data::scrypto::*;
use sbor::rust::fmt::Debug;

pub struct AccessRulesObject;

impl AccessRulesObject {
    pub fn sys_new<Y, E: Debug + ScryptoDecode>(
        access_rules: AccessRulesConfig,
        api: &mut Y,
    ) -> Result<Own, E>
    where
        Y: ClientApi<E>,
    {
        let rtn = api.call_function(
            ACCESS_RULES_PACKAGE,
            ACCESS_RULES_BLUEPRINT,
            ACCESS_RULES_CREATE_IDENT,
            scrypto_encode(&AccessRulesCreateInput { access_rules }).unwrap(),
        )?;

        let access_rules: Own = scrypto_decode(&rtn).unwrap();

        Ok(access_rules)
    }
}
