use crate::api::api::Invocation;
use crate::api::types::{ScryptoFunctionIdent, ScryptoMethodIdent};

/// Scrypto function/method invocation.
#[derive(Debug)]
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