use crate::api::types::*;
use crate::data::{ScryptoDecode, ScryptoEncode};
use sbor::rust::fmt::Debug;

pub trait Invocation: Debug {
    type Output: Debug;

    fn fn_identifier(&self) -> FnIdentifier;
}

/// Represents an [`Invocation`] which can be encoded and whose output type can also be decoded.
/// In addition, it must convert into a [`CallTableInvocation`]
pub trait SerializableInvocation:
    Invocation<Output = Self::ScryptoOutput> + ScryptoEncode + Into<CallTableInvocation>
{
    type ScryptoOutput: ScryptoDecode;
}
