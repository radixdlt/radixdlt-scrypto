use crate::api::api::Invocation;
use crate::api::types::{ScryptoFunctionIdent, ScryptoMethodIdent};
use crate::data::IndexedScryptoValue;
use crate::model::SerializedInvocation;
use crate::wasm::SerializableInvocation;
use crate::*;
use sbor::rust::vec::Vec;

/// Scrypto function/method invocation.
#[derive(Debug, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub enum ScryptoInvocation {
    Function(ScryptoFunctionIdent, Vec<u8>),
    Method(ScryptoMethodIdent, Vec<u8>),
}

impl Invocation for ScryptoInvocation {
    type Output = Vec<u8>;
}

impl SerializableInvocation for ScryptoInvocation {
    type ScryptoOutput = Vec<u8>;
}

impl Into<SerializedInvocation> for ScryptoInvocation {
    fn into(self) -> SerializedInvocation {
        SerializedInvocation::Scrypto(self)
    }
}

impl ScryptoInvocation {
    pub fn args(&self) -> &[u8] {
        match self {
            ScryptoInvocation::Function(_, args) => &args,
            ScryptoInvocation::Method(_, args) => &args,
        }
    }
}

#[derive(Debug)]
pub enum ParsedScryptoInvocation {
    Function(ScryptoFunctionIdent, IndexedScryptoValue),
    Method(ScryptoMethodIdent, IndexedScryptoValue),
}

impl Invocation for ParsedScryptoInvocation {
    type Output = IndexedScryptoValue;
}

impl ParsedScryptoInvocation {
    pub fn args(&self) -> &IndexedScryptoValue {
        match self {
            ParsedScryptoInvocation::Function(_, args) => &args,
            ParsedScryptoInvocation::Method(_, args) => &args,
        }
    }
}
