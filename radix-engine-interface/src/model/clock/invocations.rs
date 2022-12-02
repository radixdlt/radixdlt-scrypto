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

impl ScryptoNativeInvocation for ClockCreateInvocation {
    type ScryptoOutput = SystemAddress;
}

impl Into<NativeFnInvocation> for ClockCreateInvocation {
    fn into(self) -> NativeFnInvocation {
        NativeFnInvocation::Function(NativeFunctionInvocation::Clock(
            ClockFunctionInvocation::Create(self),
        ))
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

impl ScryptoNativeInvocation for ClockGetCurrentTimeInvocation {
    type ScryptoOutput = Instant;
}

impl Into<NativeFnInvocation> for ClockGetCurrentTimeInvocation {
    fn into(self) -> NativeFnInvocation {
        NativeFnInvocation::Method(NativeMethodInvocation::Clock(
            ClockMethodInvocation::GetCurrentTime(self),
        ))
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

impl ScryptoNativeInvocation for ClockCompareCurrentTimeInvocation {
    type ScryptoOutput = bool;
}

impl Into<NativeFnInvocation> for ClockCompareCurrentTimeInvocation {
    fn into(self) -> NativeFnInvocation {
        NativeFnInvocation::Method(NativeMethodInvocation::Clock(
            ClockMethodInvocation::CompareCurrentTime(self),
        ))
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

impl ScryptoNativeInvocation for ClockSetCurrentTimeInvocation {
    type ScryptoOutput = ();
}

impl Into<NativeFnInvocation> for ClockSetCurrentTimeInvocation {
    fn into(self) -> NativeFnInvocation {
        NativeFnInvocation::Method(NativeMethodInvocation::Clock(
            ClockMethodInvocation::SetCurrentTime(self),
        ))
    }
}
