use crate::engine::scrypto_env::ScryptoEnv;
use crate::runtime::*;
use crate::*;
use radix_engine_interface::api::node_modules::metadata::{MetadataSetInput, METADATA_BLUEPRINT, METADATA_SET_IDENT, MetadataCreateInput, METADATA_CREATE_IDENT};
use radix_engine_interface::api::types::{NodeModuleId, ObjectId, RENodeId};
use radix_engine_interface::api::{ClientObjectApi, ClientPackageApi};
use radix_engine_interface::constants::METADATA_PACKAGE;
use radix_engine_interface::data::{
    scrypto_decode, scrypto_encode,
};

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

impl MetadataMethods for Metadata {
    fn self_id(&self) -> (RENodeId, NodeModuleId) {
        (RENodeId::Object(self.0), NodeModuleId::SELF)
    }
}

#[derive(PartialEq, Eq, Hash)]
pub struct AttachedMetadata(pub Address);

impl MetadataMethods for AttachedMetadata {
    fn self_id(&self) -> (RENodeId, NodeModuleId) {
        (self.0.into(), NodeModuleId::Metadata)
    }
}

pub trait MetadataMethods {
    fn self_id(&self) -> (RENodeId, NodeModuleId);

    fn set<K: AsRef<str>, V: AsRef<str>>(&self, name: K, value: V) {
        let (node_id, module_id) = self.self_id();

        let _rtn = ScryptoEnv
            .call_module_method(
                node_id,
                module_id,
                METADATA_SET_IDENT,
                scrypto_encode(&MetadataSetInput {
                    key: name.as_ref().to_owned(),
                    value: value.as_ref().to_owned(),
                }).unwrap(),
            )
            .unwrap();
    }
}
