use radix_engine_interface::crypto::Hash;
use sbor::rust::fmt::Debug;

use crate::api::api::*;
use crate::api::types::TransactionHashId;
use crate::scrypto;
use crate::wasm::*;

#[derive(Debug)]
#[scrypto(TypeId, Encode, Decode)]
pub struct TransactionHashGetInvocation {
    pub receiver: TransactionHashId,
}

impl Invocation for TransactionHashGetInvocation {
    type Output = Hash;
}

impl SerializableInvocation for TransactionHashGetInvocation {
    type ScryptoOutput = Hash;
}

impl Into<SerializedInvocation> for TransactionHashGetInvocation {
    fn into(self) -> SerializedInvocation {
        NativeFnInvocation::Method(NativeMethodInvocation::TransactionHash(
            TransactionHashMethodInvocation::Get(self),
        ))
        .into()
    }
}
