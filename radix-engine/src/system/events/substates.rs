use radix_engine_interface::api::types::Vec;
use radix_engine_interface::events::EventTypeIdentifier;
use radix_engine_interface::ScryptoSbor;

/// Stores all events emitted during transaction runtime.
#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor, Default)]
pub struct EventStoreSubstate(pub Vec<(EventTypeIdentifier, Vec<u8>)>);
