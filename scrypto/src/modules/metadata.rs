use crate::engine::scrypto_env::ScryptoEnv;
use crate::runtime::*;
use crate::*;
use radix_engine_interface::api::node_modules::metadata::{MetadataSetInput, METADATA_BLUEPRINT, METADATA_SET_IDENT, MetadataCreateInput, METADATA_CREATE_IDENT};
use radix_engine_interface::api::types::{ObjectId, RENodeId};
use radix_engine_interface::api::{ClientObjectApi, ClientPackageApi};
use radix_engine_interface::constants::METADATA_PACKAGE;
use radix_engine_interface::data::{
    scrypto_decode, scrypto_encode,
};
use sbor::rust::string::String;

#[derive(PartialEq, Eq, Hash, Clone)]
pub struct Metadata(pub ObjectId);

impl Metadata {
    pub fn new() -> Self {
        let rtn = ScryptoEnv
            .call_function(
                METADATA_PACKAGE,
                METADATA_BLUEPRINT,
                METADATA_CREATE_IDENT,
                scrypto_encode(&MetadataCreateInput { }).unwrap(),
            )
            .unwrap();
        let metadata: Own = scrypto_decode(&rtn).unwrap();
        Self(metadata.id())
    }

    pub fn set(&self, key: String, value: String) {
        let _rtn = ScryptoEnv
            .call_method(
                RENodeId::Object(self.0),
                METADATA_SET_IDENT,
                scrypto_encode(&MetadataSetInput {
                    key, value
                }).unwrap(),
            )
            .unwrap();
    }
}
