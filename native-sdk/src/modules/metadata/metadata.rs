use radix_engine_interface::api::node_modules::metadata::{
    MetadataCreateInput, MetadataCreateWithDataInput, METADATA_BLUEPRINT, METADATA_CREATE_IDENT,
    METADATA_CREATE_WITH_DATA_IDENT,
};
use radix_engine_interface::api::ClientApi;
use radix_engine_interface::constants::METADATA_PACKAGE;
use radix_engine_interface::data::scrypto::model::Own;
use radix_engine_interface::data::scrypto::*;
use sbor::rust::prelude::*;

pub struct Metadata;

impl Metadata {
    pub fn sys_create<Y, E: Debug + ScryptoDecode>(api: &mut Y) -> Result<Own, E>
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

    pub fn sys_create_with_data<Y, E: Debug + ScryptoDecode>(
        data: BTreeMap<String, String>,
        api: &mut Y,
    ) -> Result<Own, E>
    where
        Y: ClientApi<E>,
    {
        let rtn = api.call_function(
            METADATA_PACKAGE,
            METADATA_BLUEPRINT,
            METADATA_CREATE_WITH_DATA_IDENT,
            scrypto_encode(&MetadataCreateWithDataInput { data }).unwrap(),
        )?;
        let metadata: Own = scrypto_decode(&rtn).unwrap();

        Ok(metadata)
    }
}
