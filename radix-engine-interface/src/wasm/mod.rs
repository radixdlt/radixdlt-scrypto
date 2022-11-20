use radix_engine_interface::api::api::SysInvocation;

pub mod input;
pub use input::NativeFnInvocation;
pub use input::*;

pub trait ScryptoNativeInvocation: Into<NativeFnInvocation> + SysInvocation {}
