use sbor::rust::fmt::Debug;

use crate::api::api::*;
use crate::model::*;
use crate::scrypto;
use crate::wasm::*;

#[derive(Debug)]
#[scrypto(TypeId, Encode, Decode)]
pub struct ClockCreateInvocation {}

impl Invocation for ClockCreateInvocation {
    type Output = SystemAddress;
}

impl SerializableInvocation for ClockCreateInvocation {
    type ScryptoOutput = SystemAddress;
}

impl Into<SerializedInvocation> for ClockCreateInvocation {
    fn into(self) -> SerializedInvocation {
        NativeFnInvocation::Function(NativeFunctionInvocation::Clock(
            ClockFunctionInvocation::Create(self),
        ))
        .into()
    }
}

#[derive(Debug)]
#[scrypto(TypeId, Encode, Decode)]
pub struct ClockGetCurrentTimeInvocation {
    pub receiver: SystemAddress,
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
        NativeFnInvocation::Method(NativeMethodInvocation::Clock(
            ClockMethodInvocation::GetCurrentTime(self),
        ))
        .into()
    }
}

#[derive(Debug)]
#[scrypto(TypeId, Encode, Decode)]
pub struct ClockCompareCurrentTimeInvocation {
    pub receiver: SystemAddress,
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
        NativeFnInvocation::Method(NativeMethodInvocation::Clock(
            ClockMethodInvocation::CompareCurrentTime(self),
        ))
        .into()
    }
}

#[derive(Debug)]
#[scrypto(TypeId, Encode, Decode)]
pub struct ClockSetCurrentTimeInvocation {
    pub receiver: SystemAddress,
    pub current_time_ms: u64,
}

impl Invocation for ClockSetCurrentTimeInvocation {
    type Output = ();
}

impl SerializableInvocation for ClockSetCurrentTimeInvocation {
    type ScryptoOutput = ();
}

impl Into<SerializedInvocation> for ClockSetCurrentTimeInvocation {
    fn into(self) -> SerializedInvocation {
        NativeFnInvocation::Method(NativeMethodInvocation::Clock(
            ClockMethodInvocation::SetCurrentTime(self),
        ))
        .into()
    }
}
