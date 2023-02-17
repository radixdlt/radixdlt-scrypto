use radix_engine_interface::api::node_modules::auth::AccessRulesAddAccessCheckInvocation;
use radix_engine_interface::api::node_modules::royalty::{
    ComponentSetRoyaltyConfigInput, COMPONENT_ROYALTY_SET_ROYALTY_CONFIG_IDENT,
};
use radix_engine_interface::api::types::*;
use radix_engine_interface::api::{
    ClientApi, ClientNativeInvokeApi, ClientNodeApi, ClientSubstateApi,
};
use radix_engine_interface::blueprints::resource::AccessRules;
use radix_engine_interface::data::{scrypto_encode, ScryptoDecode};
use sbor::rust::fmt::Debug;

#[derive(PartialEq, Eq, Hash, Clone)]
pub struct Component(pub ComponentId);

impl Component {
    pub fn sys_add_access_check<Y, E: Debug + ScryptoDecode>(
        &mut self,
        access_rules: AccessRules,
        api: &mut Y,
    ) -> Result<&mut Self, E>
    where
        Y: ClientNodeApi<E> + ClientSubstateApi<E> + ClientNativeInvokeApi<E>,
    {
        api.call_native(AccessRulesAddAccessCheckInvocation {
            receiver: RENodeId::Component(self.0),
            access_rules,
        })?;

        Ok(self)
    }

    pub fn sys_set_royalty_config<Y, E: Debug + ScryptoDecode>(
        &mut self,
        royalty_config: RoyaltyConfig,
        api: &mut Y,
    ) -> Result<&mut Self, E>
    where
        Y: ClientApi<E>,
    {
        api.call_module_method(
            ScryptoReceiver::Component(self.0),
            NodeModuleId::ComponentRoyalty,
            COMPONENT_ROYALTY_SET_ROYALTY_CONFIG_IDENT,
            scrypto_encode(&ComponentSetRoyaltyConfigInput { royalty_config }).unwrap(),
        )?;

        Ok(self)
    }
}
