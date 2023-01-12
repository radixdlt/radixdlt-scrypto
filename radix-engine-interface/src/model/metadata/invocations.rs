use sbor::rust::fmt::Debug;

use crate::api::{api::*, types::*};
use crate::scrypto;
use crate::wasm::*;

#[derive(Debug, Clone, Eq, PartialEq)]
#[scrypto(Categorize, Encode, Decode)]
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

#[derive(Debug, Clone, Eq, PartialEq)]
#[scrypto(Categorize, Encode, Decode)]
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
