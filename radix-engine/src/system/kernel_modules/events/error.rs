use radix_engine_interface::*;

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum EventError {
    FailedToSborEncodeEventSchema,
    FailedToSborEncodeEvent,
    InvalidActor,
}
