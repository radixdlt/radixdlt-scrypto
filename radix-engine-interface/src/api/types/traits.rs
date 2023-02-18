use crate::api::types::*;
use sbor::rust::fmt::Debug;

pub trait Invocation: Debug {
    type Output: Debug;

    fn identifier(&self) -> InvocationIdentifier;
}
