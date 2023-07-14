use crate::api::object_api::ObjectModuleId;
use crate::ManifestSbor;
use crate::ScryptoSbor;
use radix_engine_common::data::scrypto::model::Own;
use radix_engine_common::types::NodeId;
use sbor::rust::collections::BTreeMap;

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor, ManifestSbor)]
pub struct OnVirtualizeInput {
    pub variant_id: u8,
    pub rid: [u8; NodeId::RID_LENGTH],
}

pub type OnVirtualizeOutput = BTreeMap<ObjectModuleId, Own>;

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor, ManifestSbor)]
pub struct OnDropInput {}

pub type OnDropOutput = ();

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor, ManifestSbor)]
pub struct OnMoveInput {}

pub type OnMoveOutput = ();

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor, ManifestSbor)]
pub struct OnPersistInput {}

pub type OnPersistOutput = ();
