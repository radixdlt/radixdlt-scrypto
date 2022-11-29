use radix_engine_interface::api::api::Invocation;

pub mod input;
use crate::data::ScryptoDecode;
pub use input::NativeFnInvocation;
pub use input::*;

pub trait ScryptoNativeInvocation:
    Into<NativeFnInvocation> + Invocation<Output = Self::ScryptoOutput>
{
    type ScryptoOutput: ScryptoDecode;
}
