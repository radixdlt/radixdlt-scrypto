use crate::ManifestSbor;
use crate::ScryptoSbor;
use radix_engine_common::data::scrypto::model::Own;
use sbor::rust::collections::BTreeMap;
use crate::api::object_api::ObjectModuleId;

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor, ManifestSbor)]
pub struct VirtualLazyLoadInput {
    pub id: [u8; 26],
}

pub type VirtualLazyLoadOutput = BTreeMap<ObjectModuleId, Own>;
