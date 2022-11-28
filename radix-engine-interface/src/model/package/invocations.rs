use crate::api::api::Invocation;
use crate::crypto::Blob;
use crate::model::*;
use crate::scrypto;
use crate::wasm::*;
use sbor::rust::collections::HashMap;
use sbor::rust::string::String;

#[derive(Debug)]
#[scrypto(TypeId, Encode, Decode)]
pub struct PackagePublishNoOwnerInvocation {
    pub code: Blob,
    pub abi: Blob,
    pub metadata: HashMap<String, String>,
}

impl Invocation for PackagePublishNoOwnerInvocation {
    type Output = PackageAddress;
}

impl ScryptoNativeInvocation for PackagePublishNoOwnerInvocation {
    type ScryptoOutput = PackageAddress;
}

impl Into<NativeFnInvocation> for PackagePublishNoOwnerInvocation {
    fn into(self) -> NativeFnInvocation {
        NativeFnInvocation::Function(NativeFunctionInvocation::Package(
            PackageFunctionInvocation::PublishNoOwner(self),
        ))
    }
}

#[derive(Debug)]
#[scrypto(TypeId, Encode, Decode)]
pub struct PackagePublishWithOwnerInvocation {
    pub code: Blob,
    pub abi: Blob,
    pub metadata: HashMap<String, String>,
}

impl Invocation for PackagePublishWithOwnerInvocation {
    type Output = (PackageAddress, Bucket);
}

impl ScryptoNativeInvocation for PackagePublishWithOwnerInvocation {
    type ScryptoOutput = (PackageAddress, Bucket);
}

impl Into<NativeFnInvocation> for PackagePublishWithOwnerInvocation {
    fn into(self) -> NativeFnInvocation {
        NativeFnInvocation::Function(NativeFunctionInvocation::Package(
            PackageFunctionInvocation::PublishWithOwner(self),
        ))
    }
}
