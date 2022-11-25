use crate::api::api::{ScryptoNativeInvocation, SysInvocation};
use crate::api::wasm_input::{
    NativeFnInvocation, NativeFunctionInvocation, NativeMethodInvocation,
    PackageFunctionInvocation, PackageMethodInvocation,
};
use crate::crypto::Blob;
use crate::model::*;
use crate::scrypto;
use sbor::rust::collections::HashMap;
use sbor::rust::string::String;

#[derive(Debug)]
#[scrypto(TypeId, Encode, Decode)]
pub struct PackagePublishInvocation {
    pub code: Blob,
    pub abi: Blob,
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

#[derive(Debug)]
#[scrypto(TypeId, Encode, Decode)]
pub struct PackageSetRoyaltyConfigInvocation {
    pub receiver: PackageAddress,
    pub royalty_config: HashMap<String, RoyaltyConfig>, // TODO: optimize to allow per blueprint configuration.
}

impl SysInvocation for PackageSetRoyaltyConfigInvocation {
    type Output = ();
}

impl ScryptoNativeInvocation for PackageSetRoyaltyConfigInvocation {}

impl Into<NativeFnInvocation> for PackageSetRoyaltyConfigInvocation {
    fn into(self) -> NativeFnInvocation {
        NativeFnInvocation::Method(NativeMethodInvocation::Package(
            PackageMethodInvocation::SetRoyaltyConfig(self),
        ))
    }
}
