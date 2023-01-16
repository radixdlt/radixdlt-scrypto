use radix_engine_interface::api::Invocation;

pub mod input;
use crate::data::ScryptoDecode;
pub use input::*;
use radix_engine_interface::model::CallTableInvocation;

pub trait SerializableInvocation:
    Into<CallTableInvocation> + Invocation<Output = Self::ScryptoOutput>
{
    type ScryptoOutput: ScryptoDecode;
}
