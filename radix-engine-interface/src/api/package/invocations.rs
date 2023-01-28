use crate::api::types::*;
use crate::blueprints::resource::*;
use crate::*;
use sbor::rust::collections::BTreeMap;
use sbor::rust::string::String;
use sbor::rust::vec::Vec;

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct PackagePublishInvocation {
    pub package_address: Option<[u8; 26]>, // TODO: Clean this up
    pub code: Vec<u8>,
    pub abi: Vec<u8>,
    pub royalty_config: BTreeMap<String, RoyaltyConfig>,
    pub metadata: BTreeMap<String, String>,
    pub access_rules: AccessRules,
}

impl Invocation for PackagePublishInvocation {
    type Output = PackageAddress;

    fn fn_identifier(&self) -> FnIdentifier {
        FnIdentifier::Native(NativeFn::Package(PackageFn::Publish))
    }
}

impl SerializableInvocation for PackagePublishInvocation {
    type ScryptoOutput = PackageAddress;
}

impl Into<CallTableInvocation> for PackagePublishInvocation {
    fn into(self) -> CallTableInvocation {
        NativeInvocation::Package(PackageInvocation::Publish(self)).into()
    }
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct PackageSetRoyaltyConfigInvocation {
    pub receiver: PackageAddress,
    pub royalty_config: BTreeMap<String, RoyaltyConfig>, // TODO: optimize to allow per blueprint configuration.
}

impl Invocation for PackageSetRoyaltyConfigInvocation {
    type Output = ();

    fn fn_identifier(&self) -> FnIdentifier {
        FnIdentifier::Native(NativeFn::Package(PackageFn::SetRoyaltyConfig))
    }
}

impl SerializableInvocation for PackageSetRoyaltyConfigInvocation {
    type ScryptoOutput = ();
}

impl Into<CallTableInvocation> for PackageSetRoyaltyConfigInvocation {
    fn into(self) -> CallTableInvocation {
        NativeInvocation::Package(PackageInvocation::SetRoyaltyConfig(self)).into()
    }
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct PackageSetRoyaltyConfigExecutable {
    pub receiver: RENodeId,
    pub royalty_config: BTreeMap<String, RoyaltyConfig>,
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct PackageClaimRoyaltyInvocation {
    pub receiver: PackageAddress,
}

impl Invocation for PackageClaimRoyaltyInvocation {
    type Output = Bucket;

    fn fn_identifier(&self) -> FnIdentifier {
        FnIdentifier::Native(NativeFn::Package(PackageFn::ClaimRoyalty))
    }
}

impl SerializableInvocation for PackageClaimRoyaltyInvocation {
    type ScryptoOutput = Bucket;
}

impl Into<CallTableInvocation> for PackageClaimRoyaltyInvocation {
    fn into(self) -> CallTableInvocation {
        NativeInvocation::Package(PackageInvocation::ClaimRoyalty(self)).into()
    }
}

#[derive(Debug, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct PackageClaimRoyaltyExecutable {
    pub receiver: RENodeId,
}
