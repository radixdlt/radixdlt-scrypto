use crate::api::types::*;
use crate::blueprints::resource::*;
use crate::*;
use sbor::rust::collections::BTreeMap;
use sbor::rust::string::String;
use sbor::rust::vec::Vec;

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
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

    fn native_fn() -> NativeFn {
        NativeFn::Package(PackageFn::Publish)
    }
}

impl Into<CallTableInvocation> for PackagePublishInvocation {
    fn into(self) -> CallTableInvocation {
        NativeInvocation::Package(PackageInvocation::Publish(self)).into()
    }
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct PackagePublishNativeInvocation {
    pub package_address: Option<[u8; 26]>, // TODO: Clean this up
    pub native_package_code_id: u8,
    pub abi: Vec<u8>,
    pub dependent_resources: Vec<ResourceAddress>,
    pub dependent_components: Vec<ComponentAddress>,
    pub metadata: BTreeMap<String, String>,
    pub access_rules: AccessRules,
}

impl Invocation for PackagePublishNativeInvocation {
    type Output = PackageAddress;

    fn fn_identifier(&self) -> FnIdentifier {
        FnIdentifier::Native(NativeFn::Package(PackageFn::PublishNative))
    }
}

impl SerializableInvocation for PackagePublishNativeInvocation {
    type ScryptoOutput = PackageAddress;

    fn native_fn() -> NativeFn {
        NativeFn::Package(PackageFn::PublishNative)
    }
}

impl Into<CallTableInvocation> for PackagePublishNativeInvocation {
    fn into(self) -> CallTableInvocation {
        NativeInvocation::Package(PackageInvocation::PublishNative(self)).into()
    }
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
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

    fn native_fn() -> NativeFn {
        NativeFn::Package(PackageFn::SetRoyaltyConfig)
    }
}

impl Into<CallTableInvocation> for PackageSetRoyaltyConfigInvocation {
    fn into(self) -> CallTableInvocation {
        NativeInvocation::Package(PackageInvocation::SetRoyaltyConfig(self)).into()
    }
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct PackageSetRoyaltyConfigExecutable {
    pub receiver: RENodeId,
    pub royalty_config: BTreeMap<String, RoyaltyConfig>,
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
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

    fn native_fn() -> NativeFn {
        NativeFn::Package(PackageFn::ClaimRoyalty)
    }
}

impl Into<CallTableInvocation> for PackageClaimRoyaltyInvocation {
    fn into(self) -> CallTableInvocation {
        NativeInvocation::Package(PackageInvocation::ClaimRoyalty(self)).into()
    }
}

#[derive(Debug, ScryptoSbor)]
pub struct PackageClaimRoyaltyExecutable {
    pub receiver: RENodeId,
}
