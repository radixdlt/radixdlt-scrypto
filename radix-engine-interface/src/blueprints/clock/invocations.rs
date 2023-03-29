use crate::blueprints::clock::TimePrecision;
use crate::time::{Instant, TimeComparisonOperator};
use crate::*;
use radix_engine_common::types::ComponentAddress;
use sbor::rust::fmt::Debug;

pub const CLOCK_BLUEPRINT: &str = "Clock";

pub const CLOCK_CREATE_IDENT: &str = "create";

#[derive(Debug, Clone, Eq, PartialEq, Sbor)]
pub struct ClockCreateInput {
    pub component_address: [u8; 27], // TODO: Clean this up
}

pub type ClockCreateOutput = ComponentAddress;

pub const CLOCK_GET_CURRENT_TIME_IDENT: &str = "get_current_time";

#[derive(Debug, Clone, Eq, PartialEq, Sbor)]
pub struct ClockGetCurrentTimeInput {
    pub precision: TimePrecision,
}

pub type ClockGetCurrentTimeOutput = Instant;

pub const CLOCK_COMPARE_CURRENT_TIME_IDENT: &str = "compare_current_time";

#[derive(Debug, Clone, Eq, PartialEq, Sbor)]
pub struct ClockCompareCurrentTimeInput {
    pub instant: Instant,
    pub precision: TimePrecision,
    pub operator: TimeComparisonOperator,
}

pub type ClockCompareCurrentTimeOutput = bool;

pub const CLOCK_SET_CURRENT_TIME_IDENT: &str = "set_current_time";

#[derive(Debug, Clone, Eq, PartialEq, Sbor)]
pub struct ClockSetCurrentTimeInput {
    pub current_time_ms: i64,
}

pub type ClockSetCurrentTimeOutput = ();
