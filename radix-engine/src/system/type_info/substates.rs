use crate::types::*;
use sbor::rust::fmt::Debug;

#[derive(Debug, Clone, PartialEq, Eq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub enum TypeInfoSubstate {
    WasmPackage,
    NativePackage,
}
