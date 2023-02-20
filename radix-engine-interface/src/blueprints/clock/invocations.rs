use crate::api::types::*;
use crate::blueprints::clock::TimePrecision;
use crate::time::{Instant, TimeComparisonOperator};
use crate::*;
use sbor::rust::collections::BTreeMap;
use sbor::rust::fmt::Debug;
use scrypto_abi::BlueprintAbi;

pub struct ClockAbi;

impl ClockAbi {
    pub fn blueprint_abis() -> BTreeMap<String, BlueprintAbi> {
        BTreeMap::new()
    }
}

pub const CLOCK_BLUEPRINT: &str = "Clock";

pub const CLOCK_CREATE_IDENT: &str = "create";

#[derive(Debug, Clone, Eq, PartialEq, Sbor)]
pub struct ClockCreateInput {
    pub component_address: [u8; 26], // TODO: Clean this up
}

pub const CLOCK_GET_CURRENT_TIME_IDENT: &str = "get_current_time";

#[derive(Debug, Clone, Eq, PartialEq, Sbor)]
pub struct ClockGetCurrentTimeInput {
    pub precision: TimePrecision,
}

pub const CLOCK_COMPARE_CURRENT_TIME_IDENT: &str = "compare_current_time";

#[derive(Debug, Clone, Eq, PartialEq, Sbor)]
pub struct ClockCompareCurrentTimeInput {
    pub instant: Instant,
    pub precision: TimePrecision,
    pub operator: TimeComparisonOperator,
}

pub const CLOCK_SET_CURRENT_TIME_IDENT: &str = "set_current_time";

#[derive(Debug, Clone, Eq, PartialEq, Sbor)]
pub struct ClockSetCurrentTimeInput {
    pub current_time_ms: i64,
}
