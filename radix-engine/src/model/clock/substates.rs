use crate::types::*;

#[derive(Debug, Clone, TypeId, Encode, Decode, PartialEq, Eq)]
pub struct CurrentTimeSubstate {
    pub current_time_ms: u64,
}

#[derive(Debug, Clone, TypeId, Encode, Decode, PartialEq, Eq)]
pub struct CurrentTimeRoundedToSecondsSubstate {
    pub current_time_rounded_to_seconds_ms: u64,
}

#[derive(Debug, Clone, TypeId, Encode, Decode, PartialEq, Eq)]
pub struct CurrentTimeRoundedToMinutesSubstate {
    pub current_time_rounded_to_minutes_ms: u64,
}
