use radix_common::constants::METADATA_MODULE_PACKAGE;
use radix_common::data::scrypto::model::Own;
use radix_common::data::scrypto::*;
use radix_engine_interface::api::*;
use radix_engine_interface::object_modules::metadata::{
    MetadataCreateInput, MetadataCreateWithDataInput, MetadataInit, MetadataSetInput, MetadataVal,
    METADATA_BLUEPRINT, METADATA_CREATE_IDENT, METADATA_CREATE_WITH_DATA_IDENT, METADATA_SET_IDENT,
};
use sbor::rust::prelude::*;

pub struct Metadata(pub Own);

impl Metadata {
    pub fn create<Y: SystemApi<E>, E: SystemApiError>(api: &mut Y) -> Result<Own, E> {
        let rtn = api.call_function(
            METADATA_MODULE_PACKAGE,
            METADATA_BLUEPRINT,
            METADATA_CREATE_IDENT,
            scrypto_encode(&MetadataCreateInput {}).unwrap(),
        )?;
        let metadata: Own = scrypto_decode(&rtn).unwrap();

        Ok(metadata)
    }

    pub fn create_with_data<Y: SystemApi<E>, E: SystemApiError>(
        data: MetadataInit,
        api: &mut Y,
    ) -> Result<Own, E> {
        let rtn = api.call_function(
            METADATA_MODULE_PACKAGE,
            METADATA_BLUEPRINT,
            METADATA_CREATE_WITH_DATA_IDENT,
            scrypto_encode(&MetadataCreateWithDataInput { data }).unwrap(),
        )?;
        let metadata: Own = scrypto_decode(&rtn).unwrap();

        Ok(metadata)
    }

    pub fn new<Y: SystemApi<E>, E: SystemApiError>(api: &mut Y) -> Result<Self, E> {
        Self::create(api).map(Self)
    }

    pub fn set<Y: SystemApi<E>, E: SystemApiError, S: AsRef<str>, V: MetadataVal>(
        &mut self,
        api: &mut Y,
        key: S,
        value: V,
    ) -> Result<(), E> {
        api.call_method(
            self.0.as_node_id(),
            METADATA_SET_IDENT,
            scrypto_encode(&MetadataSetInput {
                key: key.as_ref().to_owned(),
                value: value.to_metadata_value(),
            })
            .unwrap(),
        )?;
        Ok(())
    }
}
