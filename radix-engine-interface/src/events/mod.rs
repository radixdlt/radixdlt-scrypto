use crate::api::types::{Hash, NodeModuleId, RENodeId};
use crate::ScryptoSbor;

/// An identifier to identify the schema of the event. In addition to the hash of the schema of the
/// event, this identifier also stores information relevant to the emitter of the event, namely
/// their node id and node module id.
#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub struct EventTypeIdentifier(pub RENodeId, pub NodeModuleId, pub Hash);
