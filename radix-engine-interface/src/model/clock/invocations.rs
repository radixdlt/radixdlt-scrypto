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
pub struct ClockGetCurrentTimeInMinutesInvocation {
    pub receiver: SystemAddress,
}

impl SysInvocation for ClockGetCurrentTimeInMinutesInvocation {
    type Output = u64;
}

impl ScryptoNativeInvocation for ClockGetCurrentTimeInMinutesInvocation {}

impl Into<NativeFnInvocation> for ClockGetCurrentTimeInMinutesInvocation {
    fn into(self) -> NativeFnInvocation {
        NativeFnInvocation::Method(NativeMethodInvocation::Clock(
            ClockMethodInvocation::GetCurrentTimeInMinutes(self),
        ))
    }
}

#[derive(Debug)]
#[scrypto(TypeId, Encode, Decode)]
pub struct ClockSetCurrentTimeInvocation {
    pub receiver: SystemAddress,
    pub current_time_millis: u64,
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
