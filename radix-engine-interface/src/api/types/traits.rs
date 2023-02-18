use crate::api::types::*;
use sbor::rust::fmt::Debug;

pub trait Invocation: Debug {
    type Output: Debug;

    fn fn_identifier(&self) -> FnIdentifier;
}
