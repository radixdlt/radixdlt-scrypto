use crate::engine::scrypto_env::ScryptoEnv;
use crate::runtime::*;
use crate::*;
use radix_engine_interface::api::node_modules::metadata::{MetadataCreateInput, MetadataGetStringInput, MetadataRemoveInput, MetadataSetStringInput, METADATA_BLUEPRINT, METADATA_CREATE_IDENT, METADATA_GET_STRING_IDENT, METADATA_REMOVE_IDENT, METADATA_SET_STRING_IDENT, MetadataError};
use radix_engine_interface::api::types::{NodeModuleId, ObjectId, RENodeId};
use radix_engine_interface::api::{ClientObjectApi, ClientPackageApi};
use radix_engine_interface::constants::METADATA_PACKAGE;
use radix_engine_interface::data::scrypto::{scrypto_decode, scrypto_encode};
use sbor::rust::prelude::ToOwned;
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
                scrypto_encode(&MetadataCreateInput {}).unwrap(),
            )
            .unwrap();
        let metadata: Own = scrypto_decode(&rtn).unwrap();
        Self(metadata.id())
    }
}

impl MetadataObject for Metadata {
    fn self_id(&self) -> (RENodeId, NodeModuleId) {
        (RENodeId::Object(self.0), NodeModuleId::SELF)
    }
}

#[derive(PartialEq, Eq, Hash)]
pub struct AttachedMetadata(pub Address);

impl MetadataObject for AttachedMetadata {
    fn self_id(&self) -> (RENodeId, NodeModuleId) {
        (self.0.into(), NodeModuleId::Metadata)
    }
}

pub trait MetadataObject {
    fn self_id(&self) -> (RENodeId, NodeModuleId);

    fn set_string<K: AsRef<str>, V: AsRef<str>>(&self, name: K, value: V) {
        let (node_id, module_id) = self.self_id();

        let _rtn = ScryptoEnv
            .call_module_method(
                node_id,
                module_id,
                METADATA_SET_STRING_IDENT,
                scrypto_encode(&MetadataSetStringInput {
                    key: name.as_ref().to_owned(),
                    value: value.as_ref().to_owned(),
                })
                .unwrap(),
            )
            .unwrap();
    }

    fn get_string<K: AsRef<str>>(&self, name: K) -> Result<String, MetadataError> {
        let (node_id, module_id) = self.self_id();

        let rtn = ScryptoEnv
            .call_module_method(
                node_id,
                module_id,
                METADATA_GET_STRING_IDENT,
                scrypto_encode(&MetadataGetStringInput {
                    key: name.as_ref().to_owned(),
                })
                .unwrap(),
            )
            .unwrap();

        scrypto_decode(&rtn).unwrap()
    }

    fn remove<K: AsRef<str>>(&self, name: K) -> bool {
        let (node_id, module_id) = self.self_id();

        let rtn = ScryptoEnv
            .call_module_method(
                node_id,
                module_id,
                METADATA_REMOVE_IDENT,
                scrypto_encode(&MetadataRemoveInput {
                    key: name.as_ref().to_owned(),
                })
                .unwrap(),
            )
            .unwrap();

        scrypto_decode(&rtn).unwrap()
    }
}
