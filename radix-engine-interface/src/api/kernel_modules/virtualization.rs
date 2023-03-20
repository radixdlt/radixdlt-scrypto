use sbor::rust::collections::BTreeMap;
use radix_engine_common::data::scrypto::model::Own;
use crate::ManifestSbor;
use crate::ScryptoSbor;
use crate::api::types::*;

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor, ManifestSbor)]
pub struct VirtualLazyLoadInput {
    pub id: [u8; 26],
}

pub type VirtualLazyLoadOutput = (Own, BTreeMap<NodeModuleId, Own>);
