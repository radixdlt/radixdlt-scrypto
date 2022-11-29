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
pub struct ClockGetCurrentTimeRoundedToMinutesInvocation {
    pub receiver: SystemAddress,
}

impl Invocation for ClockGetCurrentTimeRoundedToMinutesInvocation {
    type Output = u64;
}

impl ScryptoNativeInvocation for ClockGetCurrentTimeRoundedToMinutesInvocation {
    type ScryptoOutput = u64;
}

impl Into<NativeFnInvocation> for ClockGetCurrentTimeRoundedToMinutesInvocation {
    fn into(self) -> NativeFnInvocation {
        NativeFnInvocation::Method(NativeMethodInvocation::Clock(
            ClockMethodInvocation::GetCurrentTimeRoundedToMinutes(self),
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
