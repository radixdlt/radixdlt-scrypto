use radix_engine_interface::api::node_modules::metadata::{
    MetadataCreateInput, METADATA_BLUEPRINT, METADATA_CREATE_IDENT,
};
use radix_engine_interface::api::ClientApi;
use radix_engine_interface::constants::METADATA_PACKAGE;
use radix_engine_interface::data::scrypto::model::Own;
use radix_engine_interface::data::scrypto::*;
use sbor::rust::fmt::Debug;

pub struct Metadata;

impl Metadata {
    pub fn sys_new<Y, E: Debug + ScryptoDecode>(api: &mut Y) -> Result<Own, E>
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
