use crate::api::api::SysInvocation;
use crate::crypto::Blob;
use crate::model::*;
use crate::scrypto;
use crate::wasm::*;
use sbor::rust::collections::HashMap;

#[derive(Debug)]
#[scrypto(TypeId, Encode, Decode)]
pub struct PackagePublishInvocation {
    pub code: Blob,
    pub abi: Blob,
    pub metadata: HashMap<String, String>,
}

impl SysInvocation for PackagePublishInvocation {
    type Output = PackageAddress;
}

impl ScryptoNativeInvocation for PackagePublishInvocation {}

impl Into<NativeFnInvocation> for PackagePublishInvocation {
    fn into(self) -> NativeFnInvocation {
        NativeFnInvocation::Function(NativeFunctionInvocation::Package(
            PackageFunctionInvocation::Publish(self),
        ))
    }
}
