use crate::api::types::{Hash, NodeModuleId, RENodeId};
use crate::ScryptoSbor;

/// Identifies a specific event schema emitter by some emitter RENode.
///
/// This type is an identifier uses to identify the schema of events emitted by an RENode of some
/// [`RENodeId`]. With this identifier, the schema for an event can be queried, obtained, and with
/// it, the SBOR encoded event data can be decoded and understood.
///
/// It is important to note that application events are always emitted by an RENode, meaning that
/// there is always an emitter of some [`RENodeId`].
#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub struct EventTypeIdentifier(pub RENodeId, pub NodeModuleId, pub Hash);
