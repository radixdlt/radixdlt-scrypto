use crate::api::object_api::ObjectModuleId;
use crate::ManifestSbor;
use crate::ScryptoSbor;
use radix_engine_common::data::scrypto::model::Own;
use radix_engine_common::types::NodeId;
use sbor::rust::collections::BTreeMap;

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor, ManifestSbor)]
pub struct VirtualLazyLoadInput {
    pub id: [u8; NodeId::RID_LENGTH],
}

pub type VirtualLazyLoadOutput = BTreeMap<ObjectModuleId, Own>;
