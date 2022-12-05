use crate::api::api::Invocation;
use crate::api::types::{ScryptoFunctionIdent, ScryptoMethodIdent};
use crate::scrypto;
use crate::wasm::{SerializableInvocation, SerializedInvocation};
use sbor::rust::vec::Vec;

/// Scrypto function/method invocation.
#[derive(Debug)]
#[scrypto(TypeId, Encode, Decode)]
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
