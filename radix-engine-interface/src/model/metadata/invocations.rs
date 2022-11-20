use sbor::rust::fmt::Debug;

use crate::api::{api::*, types::*, wasm_input::*};
use crate::scrypto;

#[derive(Debug)]
#[scrypto(TypeId, Encode, Decode)]
pub struct MetadataSetInvocation {
    pub receiver: RENodeId,
    pub key: String,
    pub value: String,
}

impl SysInvocation for MetadataSetInvocation {
    type Output = ();
}

impl ScryptoNativeInvocation for MetadataSetInvocation {}

impl Into<NativeFnInvocation> for MetadataSetInvocation {
    fn into(self) -> NativeFnInvocation {
        NativeFnInvocation::Method(NativeMethodInvocation::Metadata(
            MetadataMethodInvocation::Set(self),
        ))
    }
}
