use native_sdk::ScryptoSbor;

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum EventError {
    FailedToSborEncodeEventSchema,
    FailedToSborEncodeEvent,
    InvalidActor,
}
