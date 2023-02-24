use crate::api::types::*;
use sbor::rust::fmt::Debug;

pub trait Invocation: Debug {
    type Output: Debug;

    fn debug_identifier(&self) -> InvocationDebugIdentifier;
}
