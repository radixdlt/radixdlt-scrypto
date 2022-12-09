use radix_engine_interface::crypto::Hash;
use sbor::rust::fmt::Debug;

use crate::api::api::*;
use crate::api::types::TransactionHashId;
use crate::scrypto;
use crate::wasm::*;
use sbor::*;

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

#[derive(Debug)]
#[scrypto(TypeId, Encode, Decode)]
pub struct TransactionHashGenerateUuidInvocation {
    pub receiver: TransactionHashId,
}

impl Invocation for TransactionHashGenerateUuidInvocation {
    type Output = u128;
}

impl SerializableInvocation for TransactionHashGenerateUuidInvocation {
    type ScryptoOutput = u128;
}

impl Into<SerializedInvocation> for TransactionHashGenerateUuidInvocation {
    fn into(self) -> SerializedInvocation {
        NativeFnInvocation::Method(NativeMethodInvocation::TransactionHash(
            TransactionHashMethodInvocation::GenerateUuid(self),
        ))
        .into()
    }
}
