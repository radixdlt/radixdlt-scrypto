use crate::api::Invocation;
use crate::data::ScryptoDecode;
use crate::model::CallTableInvocation;

pub trait SerializableInvocation:
    Into<CallTableInvocation> + Invocation<Output = Self::ScryptoOutput>
{
    type ScryptoOutput: ScryptoDecode;
}
