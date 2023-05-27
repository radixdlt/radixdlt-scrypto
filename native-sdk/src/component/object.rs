use radix_engine_interface::api::node_modules::metadata::{
    MetadataSetInput, MetadataVal, METADATA_SET_IDENT,
};
use radix_engine_interface::api::object_api::ObjectModuleId;
use radix_engine_interface::api::ClientApi;
use radix_engine_interface::data::scrypto::{scrypto_encode, ScryptoDecode};
use radix_engine_interface::types::NodeId;
use sbor::rust::prelude::{Debug, ToOwned};

#[derive(Debug)]
pub struct BorrowedObject(pub NodeId);

impl BorrowedObject {
    pub fn new<T>(node_id: T) -> Self
    where
        T: Into<[u8; NodeId::LENGTH]>,
    {
        Self(NodeId(node_id.into()))
    }

    pub fn set_metadata<Y, E, S, V>(&mut self, key: S, value: V, api: &mut Y) -> Result<(), E>
    where
        Y: ClientApi<E>,
        S: AsRef<str>,
        V: MetadataVal,
        E: Debug + ScryptoDecode,
    {
        api.call_method_advanced(
            &self.0,
            false,
            ObjectModuleId::Metadata,
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
