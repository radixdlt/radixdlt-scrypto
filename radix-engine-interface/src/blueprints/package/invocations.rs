use crate::api::types::*;
use crate::blueprints::resource::*;
use crate::data::scrypto::model::*;
use crate::*;
use sbor::rust::collections::BTreeMap;
use sbor::rust::string::String;
use sbor::rust::vec::Vec;
use scrypto_schema::PackageSchema;

pub const PACKAGE_BLUEPRINT: &str = "Package";

pub const PACKAGE_PUBLISH_WASM_IDENT: &str = "publish_wasm";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor, ManifestSbor)]
pub struct PackageLoaderPublishWasmInput {
    pub package_address: Option<[u8; 26]>, // TODO: Clean this up
    pub code: Vec<u8>,
    pub schema: PackageSchema,
    pub royalty_config: BTreeMap<String, RoyaltyConfig>,
    pub metadata: BTreeMap<String, String>,
    pub access_rules: AccessRulesConfig,
}

pub const PACKAGE_PUBLISH_NATIVE_IDENT: &str = "publish_native";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor, ManifestSbor)]
pub struct PackageLoaderPublishNativeInput {
    pub package_address: Option<[u8; 26]>, // TODO: Clean this up
    pub native_package_code_id: u8,
    pub schema: PackageSchema,
    pub dependent_resources: Vec<ResourceAddress>,
    pub dependent_components: Vec<ComponentAddress>,
    pub metadata: BTreeMap<String, String>,
    pub access_rules: AccessRulesConfig,

    pub package_access_rules: BTreeMap<FnKey, AccessRule>,
    pub default_package_access_rule: AccessRule,
}

pub const PACKAGE_SET_ROYALTY_CONFIG_IDENT: &str = "PackageRoyalty_set_royalty_config";

#[derive(
    Debug, Clone, Eq, PartialEq, ScryptoSbor, ManifestCategorize, ManifestEncode, ManifestDecode,
)]
pub struct PackageSetRoyaltyConfigInput {
    pub royalty_config: BTreeMap<String, RoyaltyConfig>, // TODO: optimize to allow per blueprint configuration.
}

pub type PackageSetRoyaltyConfigOutput = ();

pub const PACKAGE_CLAIM_ROYALTY_IDENT: &str = "PackageRoyalty_claim_royalty";

#[derive(
    Debug, Clone, Eq, PartialEq, ScryptoSbor, ManifestCategorize, ManifestEncode, ManifestDecode,
)]
pub struct PackageClaimRoyaltyInput {}

pub type PackageClaimRoyaltyOutput = Bucket;
