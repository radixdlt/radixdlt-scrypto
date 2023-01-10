use sbor::rust::fmt::Debug;
use sbor::*;

use crate::api::api::*;
use crate::model::*;
use crate::scrypto;
use crate::time::{Instant, TimeComparisonOperator};
use crate::wasm::*;

#[derive(Debug, Clone, Eq, PartialEq)]
#[scrypto(Categorize, Encode, Decode)]
pub struct ClockCreateInvocation {}

impl Invocation for ClockCreateInvocation {
    type Output = ComponentAddress;
}

impl SerializableInvocation for ClockCreateInvocation {
    type ScryptoOutput = ComponentAddress;
}

impl Into<SerializedInvocation> for ClockCreateInvocation {
    fn into(self) -> SerializedInvocation {
        NativeInvocation::Clock(ClockInvocation::Create(self)).into()
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
#[scrypto(Categorize, Encode, Decode)]
pub struct ClockGetCurrentTimeInvocation {
    pub receiver: ComponentAddress,
    pub precision: TimePrecision,
}

impl Invocation for ClockGetCurrentTimeInvocation {
    type Output = Instant;
}

impl SerializableInvocation for ClockGetCurrentTimeInvocation {
    type ScryptoOutput = Instant;
}

impl Into<SerializedInvocation> for ClockGetCurrentTimeInvocation {
    fn into(self) -> SerializedInvocation {
        NativeInvocation::Clock(ClockInvocation::GetCurrentTime(self)).into()
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
#[scrypto(Categorize, Encode, Decode)]
pub struct ClockCompareCurrentTimeInvocation {
    pub receiver: ComponentAddress,
    pub instant: Instant,
    pub precision: TimePrecision,
    pub operator: TimeComparisonOperator,
}

impl Invocation for ClockCompareCurrentTimeInvocation {
    type Output = bool;
}

impl SerializableInvocation for ClockCompareCurrentTimeInvocation {
    type ScryptoOutput = bool;
}

impl Into<SerializedInvocation> for ClockCompareCurrentTimeInvocation {
    fn into(self) -> SerializedInvocation {
        NativeInvocation::Clock(ClockInvocation::CompareCurrentTime(self)).into()
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
#[scrypto(Categorize, Encode, Decode)]
pub struct ClockSetCurrentTimeInvocation {
    pub receiver: ComponentAddress,
    pub current_time_ms: i64,
}

impl Invocation for ClockSetCurrentTimeInvocation {
    type Output = ();
}

impl SerializableInvocation for ClockSetCurrentTimeInvocation {
    type ScryptoOutput = ();
}

impl Into<SerializedInvocation> for ClockSetCurrentTimeInvocation {
    fn into(self) -> SerializedInvocation {
        NativeInvocation::Clock(ClockInvocation::SetCurrentTime(self)).into()
    }
}
