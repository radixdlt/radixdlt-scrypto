use radix_engine_interface::api::node_modules::royalty::{
    ComponentSetRoyaltyConfigInput, COMPONENT_ROYALTY_SET_ROYALTY_CONFIG_IDENT,
};
use radix_engine_interface::api::types::*;
use radix_engine_interface::api::ClientApi;
use radix_engine_interface::api::node_modules::metadata::{METADATA_BLUEPRINT, METADATA_CREATE_IDENT, MetadataCreateInput};
use radix_engine_interface::constants::METADATA_PACKAGE;
use radix_engine_interface::data::{scrypto_decode, scrypto_encode, ScryptoDecode};
use radix_engine_interface::data::model::Own;
use sbor::rust::fmt::Debug;

pub struct Metadata;

impl Metadata {
    pub fn sys_new<Y, E: Debug + ScryptoDecode>(
        api: &mut Y,
    ) -> Result<Own, E>
    where
        Y: ClientApi<E>,
    {
        let rtn = api.call_function(
            METADATA_PACKAGE,
            METADATA_BLUEPRINT,
            METADATA_CREATE_IDENT,
            scrypto_encode(&MetadataCreateInput {}).unwrap(),
        )?;
        let metadata: Own = scrypto_decode(&rtn).unwrap();

        Ok(metadata)
    }
}
