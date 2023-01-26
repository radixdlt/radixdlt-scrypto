use crate::api::types::*;
use crate::data::ScryptoDecode;
use sbor::rust::fmt::Debug;
use sbor::rust::format;
use sbor::rust::string::String;

pub trait Invocation: Debug {
    type Output: Debug;

    // TODO: temp to unblock large payload display; fix as part of the universal invocation refactor.
    fn fn_identifier(&self) -> String {
        format!("{:?}", self)
    }
}

pub trait SerializableInvocation:
    Into<CallTableInvocation> + Invocation<Output = Self::ScryptoOutput>
{
    type ScryptoOutput: ScryptoDecode;
}
