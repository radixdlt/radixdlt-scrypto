use crate::api::types::*;
use crate::*;
use radix_engine_interface::crypto::Hash;
use sbor::rust::fmt::Debug;

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct TransactionRuntimeGetHashInvocation {
    pub receiver: TransactionRuntimeId,
}

impl Invocation for TransactionRuntimeGetHashInvocation {
    type Output = Hash;

    fn fn_identifier(&self) -> FnIdentifier {
        FnIdentifier::Native(NativeFn::TransactionRuntime(TransactionRuntimeFn::GetHash))
    }
}

impl SerializableInvocation for TransactionRuntimeGetHashInvocation {
    type ScryptoOutput = Hash;
}

impl Into<CallTableInvocation> for TransactionRuntimeGetHashInvocation {
    fn into(self) -> CallTableInvocation {
        NativeInvocation::TransactionRuntime(TransactionRuntimeInvocation::GetHash(self)).into()
    }
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct TransactionRuntimeGenerateUuidInvocation {
    pub receiver: TransactionRuntimeId,
}

impl Invocation for TransactionRuntimeGenerateUuidInvocation {
    type Output = u128;

    fn fn_identifier(&self) -> FnIdentifier {
        FnIdentifier::Native(NativeFn::TransactionRuntime(
            TransactionRuntimeFn::GenerateUuid,
        ))
    }
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
