use crate::engine::scrypto_env::ScryptoEnv;
use crate::runtime::*;
use crate::*;
use radix_engine_interface::api::node_modules::royalty::{
    ComponentClaimRoyaltyInput, ComponentRoyaltyCreateInput, ComponentSetRoyaltyConfigInput,
    COMPONENT_ROYALTY_BLUEPRINT, COMPONENT_ROYALTY_CLAIM_ROYALTY_IDENT,
    COMPONENT_ROYALTY_CREATE_IDENT, COMPONENT_ROYALTY_SET_ROYALTY_CONFIG_IDENT,
};
use radix_engine_interface::api::ClientObjectApi;
use radix_engine_interface::blueprints::resource::Bucket;
use radix_engine_interface::constants::ROYALTY_PACKAGE;
use radix_engine_interface::data::scrypto::{scrypto_decode, scrypto_encode};
use radix_engine_interface::types::*;
use radix_engine_interface::types::{NodeId, RoyaltyConfig, TypedModuleId};

#[derive(PartialEq, Eq, Hash, Clone)]
pub struct Royalty(pub Own);

impl Royalty {
    pub fn new(royalty_config: RoyaltyConfig) -> Self {
        let rtn = ScryptoEnv
            .call_function(
                ROYALTY_PACKAGE,
                COMPONENT_ROYALTY_BLUEPRINT,
                COMPONENT_ROYALTY_CREATE_IDENT,
                scrypto_encode(&ComponentRoyaltyCreateInput { royalty_config }).unwrap(),
            )
            .unwrap();

        let royalty: Own = scrypto_decode(&rtn).unwrap();
        Self(royalty)
    }
}

impl RoyaltyObject for Royalty {
    fn self_id(&self) -> (NodeId, TypedModuleId) {
        (self.0.as_node_id().clone(), TypedModuleId::ObjectState)
    }
}

#[derive(PartialEq, Eq, Hash)]
pub struct AttachedRoyalty(pub ComponentAddress);

impl RoyaltyObject for AttachedRoyalty {
    fn self_id(&self) -> (NodeId, TypedModuleId) {
        (self.0.into(), TypedModuleId::Royalty)
    }
}

pub trait RoyaltyObject {
    fn self_id(&self) -> (NodeId, TypedModuleId);

    fn set_config(&self, royalty_config: RoyaltyConfig) {
        let (node_id, module_id) = self.self_id();

        ScryptoEnv
            .call_module_method(
                &node_id,
                module_id,
                COMPONENT_ROYALTY_SET_ROYALTY_CONFIG_IDENT,
                scrypto_encode(&ComponentSetRoyaltyConfigInput { royalty_config }).unwrap(),
            )
            .unwrap();
    }

    fn claim_royalty(&self) -> Bucket {
        let (node_id, module_id) = self.self_id();

        let rtn = ScryptoEnv
            .call_module_method(
                &node_id,
                module_id,
                COMPONENT_ROYALTY_CLAIM_ROYALTY_IDENT,
                scrypto_encode(&ComponentClaimRoyaltyInput {}).unwrap(),
            )
            .unwrap();
        scrypto_decode(&rtn).unwrap()
    }
}
