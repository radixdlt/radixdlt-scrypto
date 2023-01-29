use crate::api::types::*;
use crate::data::types::Own;
use crate::*;
use sbor::rust::fmt::Debug;

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct KeyValueStoreCreateInvocation {}

impl Invocation for KeyValueStoreCreateInvocation {
    type Output = Own;

    fn fn_identifier(&self) -> FnIdentifier {
        FnIdentifier::Native(NativeFn::KeyValueStore(KeyValueStoreFn::Create))
    }
}

impl SerializableInvocation for KeyValueStoreCreateInvocation {
    type ScryptoOutput = Own;
}

impl Into<CallTableInvocation> for KeyValueStoreCreateInvocation {
    fn into(self) -> CallTableInvocation {
        NativeInvocation::KeyValueStore(KeyValueStoreInvocation::Create(self)).into()
    }
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct KeyValueStoreGetMethodArgs {
    pub key: Vec<u8>,
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct KeyValueStoreGetInvocation {
    pub receiver: KeyValueStoreId,
    pub key: Vec<u8>,
}

impl Invocation for KeyValueStoreGetInvocation {
    type Output = Option<Vec<u8>>;

    fn fn_identifier(&self) -> FnIdentifier {
        FnIdentifier::Native(NativeFn::KeyValueStore(KeyValueStoreFn::Get))
    }
}

impl SerializableInvocation for KeyValueStoreGetInvocation {
    type ScryptoOutput = Option<Vec<u8>>;
}

impl Into<CallTableInvocation> for KeyValueStoreGetInvocation {
    fn into(self) -> CallTableInvocation {
        NativeInvocation::KeyValueStore(KeyValueStoreInvocation::Get(self)).into()
    }
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct KeyValueStoreInsertMethodArgs {
    pub current_time_ms: i64,
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct KeyValueStoreInsertInvocation {
    pub receiver: KeyValueStoreId,
    pub key: Vec<u8>,
    pub value: Vec<u8>,
}

impl Invocation for KeyValueStoreInsertInvocation {
    type Output = ();

    fn fn_identifier(&self) -> FnIdentifier {
        FnIdentifier::Native(NativeFn::KeyValueStore(KeyValueStoreFn::Insert))
    }
}

impl SerializableInvocation for KeyValueStoreInsertInvocation {
    type ScryptoOutput = ();
}

impl Into<CallTableInvocation> for KeyValueStoreInsertInvocation {
    fn into(self) -> CallTableInvocation {
        NativeInvocation::KeyValueStore(KeyValueStoreInvocation::Insert(self)).into()
    }
}
