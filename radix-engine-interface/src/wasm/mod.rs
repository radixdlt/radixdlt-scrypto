use radix_engine_interface::api::api::Invocation;

pub mod input;
pub use input::NativeFnInvocation;
pub use input::*;
use sbor::Decode;
use crate::data::ScryptoCustomTypeId;

pub trait ScryptoNativeInvocation: Into<NativeFnInvocation> + Invocation<Output = Self::ScryptoOutput> {
    type ScryptoOutput: Decode<ScryptoCustomTypeId>;
}
