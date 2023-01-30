use crate::types::*;

#[derive(Debug, Clone, Categorize, Encode, Decode, PartialEq, Eq)]
pub struct CurrentTimeRoundedToMinutesSubstate {
    pub current_time_rounded_to_minutes_ms: i64,
}
