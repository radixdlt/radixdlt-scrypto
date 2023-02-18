use crate::api::types::*;
use crate::blueprints::resource::*;
use crate::*;
use sbor::rust::collections::BTreeMap;
use sbor::rust::string::String;
use sbor::rust::vec::Vec;
use scrypto_abi::BlueprintAbi;

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

    fn native_fn() -> NativeFn {
        NativeFn::Package(PackageFn::Publish)
    }
}

impl Into<CallTableInvocation> for PackagePublishInvocation {
    fn into(self) -> CallTableInvocation {
        NativeInvocation::Package(PackageInvocation::Publish(self)).into()
    }
}


pub struct NativePackageAbi;

impl NativePackageAbi {
    pub fn blueprint_abis() -> BTreeMap<String, BlueprintAbi> {
        BTreeMap::new()
    }
}

pub const NATIVE_PACKAGE_BLUEPRINT: &str = "NativePackage";

pub const NATIVE_PACKAGE_PUBLISH_IDENT: &str = "publish";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct PackagePublishNativeInput {
    pub package_address: Option<[u8; 26]>, // TODO: Clean this up
    pub native_package_code_id: u8,
    pub abi: Vec<u8>,
    pub dependent_resources: Vec<ResourceAddress>,
    pub dependent_components: Vec<ComponentAddress>,
    pub metadata: BTreeMap<String, String>,
    pub access_rules: AccessRules,
}
