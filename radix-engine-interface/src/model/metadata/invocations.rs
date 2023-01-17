use sbor::rust::fmt::Debug;

use crate::api::types::*;
use crate::api::wasm::*;
use crate::api::*;
use crate::*;

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct MetadataSetInvocation {
    pub receiver: RENodeId,
    pub key: String,
    pub value: String,
}

impl Invocation for MetadataSetInvocation {
    type Output = ();
}

impl SerializableInvocation for MetadataSetInvocation {
    type ScryptoOutput = ();
}

impl Into<CallTableInvocation> for MetadataSetInvocation {
    fn into(self) -> CallTableInvocation {
        NativeInvocation::Metadata(MetadataInvocation::Set(self)).into()
    }
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct MetadataGetInvocation {
    pub receiver: RENodeId,
    pub key: String,
}

impl Invocation for MetadataGetInvocation {
    type Output = Option<String>;
}

impl SerializableInvocation for MetadataGetInvocation {
    type ScryptoOutput = Option<String>;
}

impl Into<CallTableInvocation> for MetadataGetInvocation {
    fn into(self) -> CallTableInvocation {
        NativeInvocation::Metadata(MetadataInvocation::Get(self)).into()
    }
}
