use radix_engine_interface::events::EventTypeIdentifier;
use radix_engine_interface::ScryptoSbor;

/// A substate that stores all events emitted during transaction runtime.
#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub struct EventStoreSubstate(pub Vec<(EventTypeIdentifier, Vec<u8>)>);
