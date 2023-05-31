use crate::types::*;
use sbor::rust::fmt::Debug;

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum PackageCodeTypeSubstate {
    Wasm,
    Native,
}
