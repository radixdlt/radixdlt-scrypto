use radix_engine_interface::crypto::Hash;
use sbor::rust::fmt::Debug;

use crate::api::api::*;
use crate::api::types::TransactionRuntimeId;
use crate::model::*;
use crate::scrypto;
use crate::wasm::*;
use sbor::*;

#[derive(Debug, Clone, Eq, PartialEq)]
#[scrypto(TypeId, Encode, Decode)]
pub struct TransactionRuntimeGetHashInvocation {
    pub receiver: TransactionRuntimeId,
}

impl Invocation for TransactionRuntimeGetHashInvocation {
    type Output = Hash;
}

impl SerializableInvocation for TransactionRuntimeGetHashInvocation {
    type ScryptoOutput = Hash;
}

impl Into<SerializedInvocation> for TransactionRuntimeGetHashInvocation {
    fn into(self) -> SerializedInvocation {
        NativeInvocation::TransactionRuntime(TransactionRuntimeInvocation::Get(self)).into()
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
#[scrypto(TypeId, Encode, Decode)]
pub struct TransactionRuntimeGenerateUuidInvocation {
    pub receiver: TransactionRuntimeId,
}

impl Invocation for TransactionRuntimeGenerateUuidInvocation {
    type Output = u128;
}

impl SerializableInvocation for TransactionRuntimeGenerateUuidInvocation {
    type ScryptoOutput = u128;
}

impl Into<SerializedInvocation> for TransactionRuntimeGenerateUuidInvocation {
    fn into(self) -> SerializedInvocation {
        NativeInvocation::TransactionRuntime(TransactionRuntimeInvocation::GenerateUuid(self))
            .into()
    }
}
