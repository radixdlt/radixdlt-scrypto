use crate::api::api::Invocation;
use crate::api::types::RENodeId;
use crate::crypto::Blob;
use crate::model::*;
use crate::scrypto;
use crate::wasm::*;
use sbor::rust::collections::HashMap;
use sbor::rust::string::String;
use sbor::rust::vec::Vec;

#[derive(Debug)]
#[scrypto(TypeId, Encode, Decode)]
pub struct PackagePublishNoOwnerInvocation {
    pub code: Blob,
    pub abi: Blob,
    pub royalty_config: HashMap<String, RoyaltyConfig>,
    pub access_rules_chain: Vec<AccessRules>,
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
    pub royalty_config: HashMap<String, RoyaltyConfig>,
    pub access_rules_chain: Vec<AccessRules>,
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

#[derive(Debug)]
#[scrypto(TypeId, Encode, Decode)]
pub struct PackageSetRoyaltyConfigInvocation {
    pub receiver: PackageAddress,
    pub royalty_config: HashMap<String, RoyaltyConfig>, // TODO: optimize to allow per blueprint configuration.
}

impl Invocation for PackageSetRoyaltyConfigInvocation {
    type Output = ();
}

impl ScryptoNativeInvocation for PackageSetRoyaltyConfigInvocation {
    type ScryptoOutput = ();
}

impl Into<NativeFnInvocation> for PackageSetRoyaltyConfigInvocation {
    fn into(self) -> NativeFnInvocation {
        NativeFnInvocation::Method(NativeMethodInvocation::Package(
            PackageMethodInvocation::SetRoyaltyConfig(self),
        ))
    }
}

#[derive(Debug)]
#[scrypto(TypeId, Encode, Decode)]
pub struct PackageSetRoyaltyConfigExecutable {
    pub receiver: RENodeId,
    pub royalty_config: HashMap<String, RoyaltyConfig>,
}

#[derive(Debug)]
#[scrypto(TypeId, Encode, Decode)]
pub struct PackageClaimRoyaltyInvocation {
    pub receiver: PackageAddress,
}

impl Invocation for PackageClaimRoyaltyInvocation {
    type Output = Bucket;
}

impl ScryptoNativeInvocation for PackageClaimRoyaltyInvocation {
    type ScryptoOutput = Bucket;
}

impl Into<NativeFnInvocation> for PackageClaimRoyaltyInvocation {
    fn into(self) -> NativeFnInvocation {
        NativeFnInvocation::Method(NativeMethodInvocation::Package(
            PackageMethodInvocation::ClaimRoyalty(self),
        ))
    }
}

#[derive(Debug)]
#[scrypto(TypeId, Encode, Decode)]
pub struct PackageClaimRoyaltyExecutable {
    pub receiver: RENodeId,
}
