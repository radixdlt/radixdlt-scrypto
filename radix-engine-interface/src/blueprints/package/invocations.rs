use crate::blueprints::package::VirtualLazyLoadExport;
use crate::blueprints::resource::*;
use crate::types::*;
use crate::*;
use radix_engine_common::data::manifest::model::ManifestBlobRef;
use radix_engine_common::data::manifest::model::ManifestOwn;
use radix_engine_common::prelude::ScryptoSchema;
use radix_engine_interface::api::node_modules::metadata::MetadataValue;
use sbor::rust::collections::BTreeMap;
use sbor::rust::collections::BTreeSet;
use sbor::rust::string::String;
use sbor::rust::vec::Vec;
use sbor::LocalTypeIndex;
use scrypto_schema::{
    BlueprintSchema, ExportSchema, ReceiverInfo, SchemaMethodKey, SchemaMethodPermission,
};

pub const PACKAGE_BLUEPRINT: &str = "Package";

pub const PACKAGE_PUBLISH_WASM_IDENT: &str = "publish_wasm";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor, ManifestSbor)]
pub struct PackagePublishWasmInput {
    pub code: Vec<u8>,
    pub setup: PackageSetup,
    pub metadata: BTreeMap<String, MetadataValue>,
}

#[derive(Debug, Clone, Eq, PartialEq, ManifestSbor)]
pub struct PackagePublishWasmManifestInput {
    pub code: ManifestBlobRef,
    pub setup: PackageSetup,
    pub metadata: BTreeMap<String, MetadataValue>,
}

pub type PackagePublishWasmOutput = (PackageAddress, Bucket);

pub const PACKAGE_PUBLISH_WASM_ADVANCED_IDENT: &str = "publish_wasm_advanced";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct PackagePublishWasmAdvancedInput {
    pub package_address: Option<GlobalAddressReservation>,
    pub code: Vec<u8>,
    pub setup: PackageSetup,
    pub metadata: BTreeMap<String, MetadataValue>,
    pub owner_rule: OwnerRole,
}

#[derive(Debug, Clone, Eq, PartialEq, ManifestSbor)]
pub struct PackagePublishWasmAdvancedManifestInput {
    pub package_address: Option<ManifestOwn>,
    pub code: ManifestBlobRef,
    pub setup: PackageSetup,
    pub metadata: BTreeMap<String, MetadataValue>,
    pub owner_rule: OwnerRole,
}

pub type PackagePublishWasmAdvancedOutput = PackageAddress;

pub const PACKAGE_PUBLISH_NATIVE_IDENT: &str = "publish_native";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct PackagePublishNativeInput {
    pub package_address: Option<GlobalAddressReservation>,
    pub native_package_code_id: u8,
    pub setup: PackageSetup,
    pub metadata: BTreeMap<String, MetadataValue>,
}

#[derive(Debug, Clone, Eq, PartialEq, ManifestSbor)]
pub struct PackagePublishNativeManifestInput {
    pub package_address: Option<ManifestOwn>,
    pub native_package_code_id: u8,
    pub setup: PackageSetup,
    pub metadata: BTreeMap<String, MetadataValue>,
}

pub type PackagePublishNativeOutput = PackageAddress;

pub const PACKAGE_SET_ROYALTY_IDENT: &str = "PackageRoyalty_set_royalty";

#[derive(
    Debug, Clone, Eq, PartialEq, ScryptoSbor, ManifestCategorize, ManifestEncode, ManifestDecode,
)]
pub struct PackageSetRoyaltyInput {
    pub blueprint: String,
    pub fn_name: String,
    pub royalty: RoyaltyAmount,
}

pub type PackageSetRoyaltyOutput = ();

pub const PACKAGE_CLAIM_ROYALTIES_IDENT: &str = "PackageRoyalty_claim_royalties";

#[derive(
    Debug, Clone, Eq, PartialEq, ScryptoSbor, ManifestCategorize, ManifestEncode, ManifestDecode,
)]
pub struct PackageClaimRoyaltiesInput {}

pub type PackageClaimRoyaltiesOutput = Bucket;

#[derive(Debug, Clone, Eq, PartialEq, Default, ScryptoSbor, ManifestSbor)]
pub struct PackageSetup {
    pub blueprints: BTreeMap<String, BlueprintSetup>,
}

#[derive(Debug, Clone, PartialEq, Eq, Sbor)]
pub struct FunctionSetup {
    pub receiver: Option<ReceiverInfo>,
    pub input: LocalTypeIndex,
    pub output: LocalTypeIndex,
    pub export: ExportSchema,
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor, ManifestSbor)]
pub struct BlueprintSetup {
    pub outer_blueprint: Option<String>,
    pub dependencies: BTreeSet<GlobalAddress>,
    pub features: BTreeSet<String>,
    pub blueprint: BlueprintSchema,
    pub event_schema: BTreeMap<String, LocalTypeIndex>,
    pub function_auth: BTreeMap<String, AccessRule>,
    pub functions: BTreeMap<String, FunctionSetup>,
    pub virtual_lazy_load_functions: BTreeMap<u8, VirtualLazyLoadExport>,
    pub royalty_config: RoyaltyConfig,
    pub schema: ScryptoSchema,
    pub template: MethodAuthTemplate,
}

impl Default for BlueprintSetup {
    fn default() -> Self {
        Self {
            outer_blueprint: None,
            dependencies: BTreeSet::default(),
            features: BTreeSet::default(),
            blueprint: BlueprintSchema::default(),
            event_schema: BTreeMap::default(),
            function_auth: BTreeMap::default(),
            functions: BTreeMap::default(),
            virtual_lazy_load_functions: BTreeMap::default(),
            royalty_config: RoyaltyConfig::default(),
            schema: ScryptoSchema {
                type_kinds: Vec::new(),
                type_metadata: Vec::new(),
                type_validations: Vec::new(),
            },
            template: MethodAuthTemplate::default(),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Default, ScryptoSbor, ManifestSbor)]
pub struct MethodAuthTemplate {
    pub method_auth_template: BTreeMap<SchemaMethodKey, SchemaMethodPermission>,
    pub outer_method_auth_template: BTreeMap<SchemaMethodKey, SchemaMethodPermission>,
}
