use crate::api::component::ComponentAddress;
use crate::api::types::*;
use crate::blueprints::clock::TimePrecision;
use crate::time::{Instant, TimeComparisonOperator};
use crate::*;
use sbor::rust::fmt::Debug;

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct ClockCreateInvocation {
    pub component_address: [u8; 26], // TODO: Clean this up
}

impl Invocation for ClockCreateInvocation {
    type Output = ComponentAddress;

    fn fn_identifier(&self) -> FnIdentifier {
        FnIdentifier::Native(NativeFn::Clock(ClockFn::Create))
    }
}

impl SerializableInvocation for ClockCreateInvocation {
    type ScryptoOutput = ComponentAddress;

    fn native_fn() -> NativeFn {
        NativeFn::Clock(ClockFn::Create)
    }
}

impl Into<CallTableInvocation> for ClockCreateInvocation {
    fn into(self) -> CallTableInvocation {
        NativeInvocation::Clock(ClockInvocation::Create(self)).into()
    }
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct ClockGetCurrentTimeMethodArgs {
    pub precision: TimePrecision,
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct ClockGetCurrentTimeInvocation {
    pub receiver: ComponentAddress,
    pub precision: TimePrecision,
}

impl Invocation for ClockGetCurrentTimeInvocation {
    type Output = Instant;

    fn fn_identifier(&self) -> FnIdentifier {
        FnIdentifier::Native(NativeFn::Clock(ClockFn::GetCurrentTime))
    }
}

impl SerializableInvocation for ClockGetCurrentTimeInvocation {
    type ScryptoOutput = Instant;

    fn native_fn() -> NativeFn {
        NativeFn::Clock(ClockFn::GetCurrentTime)
    }
}

impl Into<CallTableInvocation> for ClockGetCurrentTimeInvocation {
    fn into(self) -> CallTableInvocation {
        NativeInvocation::Clock(ClockInvocation::GetCurrentTime(self)).into()
    }
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct ClockCompareCurrentTimeMethodArgs {
    pub instant: Instant,
    pub precision: TimePrecision,
    pub operator: TimeComparisonOperator,
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct ClockCompareCurrentTimeInvocation {
    pub receiver: ComponentAddress,
    pub instant: Instant,
    pub precision: TimePrecision,
    pub operator: TimeComparisonOperator,
}

impl Invocation for ClockCompareCurrentTimeInvocation {
    type Output = bool;

    fn fn_identifier(&self) -> FnIdentifier {
        FnIdentifier::Native(NativeFn::Clock(ClockFn::CompareCurrentTime))
    }
}

impl SerializableInvocation for ClockCompareCurrentTimeInvocation {
    type ScryptoOutput = bool;

    fn native_fn() -> NativeFn {
        NativeFn::Clock(ClockFn::CompareCurrentTime)
    }
}

impl Into<CallTableInvocation> for ClockCompareCurrentTimeInvocation {
    fn into(self) -> CallTableInvocation {
        NativeInvocation::Clock(ClockInvocation::CompareCurrentTime(self)).into()
    }
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct ClockSetCurrentTimeMethodArgs {
    pub current_time_ms: i64,
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct ClockSetCurrentTimeInvocation {
    pub receiver: ComponentAddress,
    pub current_time_ms: i64,
}

impl Invocation for ClockSetCurrentTimeInvocation {
    type Output = ();

    fn fn_identifier(&self) -> FnIdentifier {
        FnIdentifier::Native(NativeFn::Clock(ClockFn::SetCurrentTime))
    }
}

impl SerializableInvocation for ClockSetCurrentTimeInvocation {
    type ScryptoOutput = ();

    fn native_fn() -> NativeFn {
        NativeFn::Clock(ClockFn::SetCurrentTime)
    }
}

impl Into<CallTableInvocation> for ClockSetCurrentTimeInvocation {
    fn into(self) -> CallTableInvocation {
        NativeInvocation::Clock(ClockInvocation::SetCurrentTime(self)).into()
    }
}
