use crate::types::*;

#[derive(Debug, Clone, TypeId, Encode, Decode, PartialEq, Eq)]
pub struct CurrentTimeInMillisSubstate {
    pub current_time_in_millis: u64,
}

#[derive(Debug, Clone, TypeId, Encode, Decode, PartialEq, Eq)]
pub struct CurrentTimeInSecondsSubstate {
    pub current_time_in_seconds: u64,
}

#[derive(Debug, Clone, TypeId, Encode, Decode, PartialEq, Eq)]
pub struct CurrentTimeInMinutesSubstate {
    pub current_time_in_minutes: u64,
}
