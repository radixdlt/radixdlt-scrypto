use radix_engine_interface::crypto::Hash;
use sbor::rust::fmt::Debug;

use crate::api::types::TransactionRuntimeId;
use crate::api::wasm::*;
use crate::api::*;
use crate::model::*;
use crate::*;

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct TransactionRuntimeGetHashInvocation {
    pub receiver: TransactionRuntimeId,
}

impl Invocation for TransactionRuntimeGetHashInvocation {
    type Output = Hash;
}

impl SerializableInvocation for TransactionRuntimeGetHashInvocation {
    type ScryptoOutput = Hash;
}

impl Into<CallTableInvocation> for TransactionRuntimeGetHashInvocation {
    fn into(self) -> CallTableInvocation {
        NativeInvocation::TransactionRuntime(TransactionRuntimeInvocation::Get(self)).into()
    }
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct TransactionRuntimeGenerateUuidInvocation {
    pub receiver: TransactionRuntimeId,
}

impl Invocation for TransactionRuntimeGenerateUuidInvocation {
    type Output = u128;
}

impl SerializableInvocation for TransactionRuntimeGenerateUuidInvocation {
    type ScryptoOutput = u128;
}

impl Into<CallTableInvocation> for TransactionRuntimeGenerateUuidInvocation {
    fn into(self) -> CallTableInvocation {
        NativeInvocation::TransactionRuntime(TransactionRuntimeInvocation::GenerateUuid(self))
            .into()
    }
}
