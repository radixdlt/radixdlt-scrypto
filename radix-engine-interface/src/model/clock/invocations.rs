use sbor::rust::fmt::Debug;

use crate::api::{api::*, wasm_input::*};
use crate::model::*;
use crate::scrypto;

#[derive(Debug)]
#[scrypto(TypeId, Encode, Decode)]
pub struct ClockCreateInvocation {}

impl SysInvocation for ClockCreateInvocation {
    type Output = SystemAddress;
}

impl ScryptoNativeInvocation for ClockCreateInvocation {}

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

impl SysInvocation for ClockGetCurrentTimeRoundedToMinutesInvocation {
    type Output = u64;
}

impl ScryptoNativeInvocation for ClockGetCurrentTimeRoundedToMinutesInvocation {}

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

impl SysInvocation for ClockSetCurrentTimeInvocation {
    type Output = ();
}

impl ScryptoNativeInvocation for ClockSetCurrentTimeInvocation {}

impl Into<NativeFnInvocation> for ClockSetCurrentTimeInvocation {
    fn into(self) -> NativeFnInvocation {
        NativeFnInvocation::Method(NativeMethodInvocation::Clock(
            ClockMethodInvocation::SetCurrentTime(self),
        ))
    }
}
