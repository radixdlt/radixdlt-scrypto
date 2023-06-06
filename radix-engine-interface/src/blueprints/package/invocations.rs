use crate::blueprints::resource::*;
use crate::types::*;
use crate::*;
use radix_engine_common::data::manifest::model::ManifestBlobRef;
use radix_engine_common::data::manifest::model::ManifestOwned;
use radix_engine_common::prelude::Own;
use radix_engine_interface::api::node_modules::metadata::MetadataValue;
use sbor::rust::collections::BTreeMap;
use sbor::rust::string::String;
use sbor::rust::vec::Vec;
use scrypto_schema::PackageSchema;

pub const PACKAGE_BLUEPRINT: &str = "Package";

pub const PACKAGE_PUBLISH_WASM_IDENT: &str = "publish_wasm";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor, ManifestSbor)]
pub struct PackagePublishWasmInput {
    pub code: Vec<u8>,
    pub schema: PackageSchema,
    pub royalty_config: BTreeMap<String, RoyaltyConfig>,
    pub metadata: BTreeMap<String, MetadataValue>,
}

#[derive(Debug, Clone, Eq, PartialEq, ManifestSbor)]
pub struct PackagePublishWasmManifestInput {
    pub code: ManifestBlobRef,
    pub schema: PackageSchema,
    pub royalty_config: BTreeMap<String, RoyaltyConfig>,
    pub metadata: BTreeMap<String, MetadataValue>,
}

pub type PackagePublishWasmOutput = (PackageAddress, Bucket);

pub const PACKAGE_PUBLISH_WASM_ADVANCED_IDENT: &str = "publish_wasm_advanced";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct PackagePublishWasmAdvancedInput {
    pub package_address: Option<Own>,
    pub code: Vec<u8>,
    pub schema: PackageSchema,
    pub royalty_config: BTreeMap<String, RoyaltyConfig>,
    pub metadata: BTreeMap<String, MetadataValue>,
    pub owner_rule: OwnerRole,
}

#[derive(Debug, Clone, Eq, PartialEq, ManifestSbor)]
pub struct PackagePublishWasmAdvancedManifestInput {
    pub package_address: Option<ManifestOwned>,
    pub code: ManifestBlobRef,
    pub schema: PackageSchema,
    pub royalty_config: BTreeMap<String, RoyaltyConfig>,
    pub metadata: BTreeMap<String, MetadataValue>,
    pub owner_rule: OwnerRole,
}

pub type PackagePublishWasmAdvancedOutput = PackageAddress;

pub const PACKAGE_PUBLISH_NATIVE_IDENT: &str = "publish_native";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct PackagePublishNativeInput {
    pub package_address: Option<Own>,
    pub native_package_code_id: u8,
    pub schema: PackageSchema,
    pub metadata: BTreeMap<String, MetadataValue>,
    pub package_access_rules: BTreeMap<FnKey, AccessRule>,
    pub default_package_access_rule: AccessRule,
}

#[derive(Debug, Clone, Eq, PartialEq, ManifestSbor)]
pub struct PackagePublishNativeManifestInput {
    pub package_address: Option<ManifestOwned>,
    pub native_package_code_id: u8,
    pub schema: PackageSchema,
    pub metadata: BTreeMap<String, MetadataValue>,
    pub package_access_rules: BTreeMap<FnKey, AccessRule>,
    pub default_package_access_rule: AccessRule,
}

pub type PackagePublishNativeOutput = PackageAddress;

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
