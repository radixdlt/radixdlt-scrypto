use crate::api::api::Invocation;
use crate::api::types::RENodeId;
use crate::model::*;
use crate::scrypto;
use crate::wasm::*;
use sbor::rust::collections::BTreeMap;
use sbor::rust::string::String;
use sbor::*;

#[derive(Debug, Clone, Eq, PartialEq)]
#[scrypto(Categorize, Encode, Decode)]
pub struct PackagePublishInvocation {
    pub code: Vec<u8>,
    pub abi: Vec<u8>,
    pub royalty_config: BTreeMap<String, RoyaltyConfig>,
    pub metadata: BTreeMap<String, String>,
    pub access_rules: AccessRules,
}

impl Invocation for PackagePublishInvocation {
    type Output = PackageAddress;

    fn fn_identifier(&self) -> String {
        "Package(Publish)".to_owned()
    }
}

impl SerializableInvocation for PackagePublishInvocation {
    type ScryptoOutput = PackageAddress;
}

impl Into<SerializedInvocation> for PackagePublishInvocation {
    fn into(self) -> SerializedInvocation {
        NativeInvocation::Package(PackageInvocation::Publish(self)).into()
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
#[scrypto(Categorize, Encode, Decode)]
pub struct PackageSetRoyaltyConfigInvocation {
    pub receiver: PackageAddress,
    pub royalty_config: BTreeMap<String, RoyaltyConfig>, // TODO: optimize to allow per blueprint configuration.
}

impl Invocation for PackageSetRoyaltyConfigInvocation {
    type Output = ();
}

impl SerializableInvocation for PackageSetRoyaltyConfigInvocation {
    type ScryptoOutput = ();
}

impl Into<SerializedInvocation> for PackageSetRoyaltyConfigInvocation {
    fn into(self) -> SerializedInvocation {
        NativeInvocation::Package(PackageInvocation::SetRoyaltyConfig(self)).into()
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
#[scrypto(Categorize, Encode, Decode)]
pub struct PackageSetRoyaltyConfigExecutable {
    pub receiver: RENodeId,
    pub royalty_config: BTreeMap<String, RoyaltyConfig>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
#[scrypto(Categorize, Encode, Decode)]
pub struct PackageClaimRoyaltyInvocation {
    pub receiver: PackageAddress,
}

impl Invocation for PackageClaimRoyaltyInvocation {
    type Output = Bucket;
}

impl SerializableInvocation for PackageClaimRoyaltyInvocation {
    type ScryptoOutput = Bucket;
}

impl Into<SerializedInvocation> for PackageClaimRoyaltyInvocation {
    fn into(self) -> SerializedInvocation {
        NativeInvocation::Package(PackageInvocation::ClaimRoyalty(self)).into()
    }
}

#[derive(Debug)]
#[scrypto(Categorize, Encode, Decode)]
pub struct PackageClaimRoyaltyExecutable {
    pub receiver: RENodeId,
}
