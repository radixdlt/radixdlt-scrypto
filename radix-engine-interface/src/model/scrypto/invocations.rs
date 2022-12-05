use crate::api::api::Invocation;
use crate::api::types::{ScryptoFunctionIdent, ScryptoMethodIdent};
use crate::scrypto;
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

impl ScryptoInvocation {
    pub fn args(&self) -> &[u8] {
        match self {
            ScryptoInvocation::Function(_, args) => &args,
            ScryptoInvocation::Method(_, args) => &args,
        }
    }
}
