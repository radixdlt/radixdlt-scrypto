use crate::blueprints::resource::*;
use crate::types::*;
use crate::*;
use radix_engine_common::data::manifest::model::ManifestAddressReservation;
use radix_engine_common::data::manifest::model::ManifestBlobRef;
use radix_engine_common::prelude::ScryptoSchema;
use radix_engine_interface::api::node_modules::metadata::MetadataValue;
use sbor::rust::collections::BTreeMap;
use sbor::rust::collections::BTreeSet;
use sbor::rust::string::String;
use sbor::rust::vec::Vec;
use sbor::LocalTypeIndex;
use scrypto_schema::{BlueprintEventSchemaInit, BlueprintStateSchemaInit, ReceiverInfo};

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
    pub package_address: Option<ManifestAddressReservation>,
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
    pub package_address: Option<ManifestAddressReservation>,
    pub native_package_code_id: u8,
    pub setup: PackageSetup,
    pub metadata: BTreeMap<String, MetadataValue>,
}

pub type PackagePublishNativeOutput = PackageAddress;

pub const PACKAGE_CLAIM_ROYALTIES_IDENT: &str = "PackageRoyalty_claim_royalties";

#[derive(
    Debug, Clone, Eq, PartialEq, ScryptoSbor, ManifestCategorize, ManifestEncode, ManifestDecode,
)]
pub struct PackageClaimRoyaltiesInput {}

pub type PackageClaimRoyaltiesOutput = Bucket;

#[derive(Debug, Clone, Eq, PartialEq, Default, ScryptoSbor, ManifestSbor)]
pub struct PackageSetup {
    pub blueprints: BTreeMap<String, BlueprintDefinitionInit>,
}

#[derive(Debug, Clone, PartialEq, Eq, Sbor)]
pub struct FunctionSchemaInit {
    pub receiver: Option<ReceiverInfo>,
    pub input: LocalTypeIndex,
    pub output: LocalTypeIndex,
    pub export: String,
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor, ManifestSbor)]
pub struct BlueprintDefinitionInit {
    pub outer_blueprint: Option<String>,
    pub feature_set: BTreeSet<String>,
    pub dependencies: BTreeSet<GlobalAddress>,

    pub schema: ScryptoSchema,
    pub blueprint: BlueprintStateSchemaInit,
    pub event_schema: BlueprintEventSchemaInit,
    pub functions: BTreeMap<String, FunctionSchemaInit>,
    pub virtual_lazy_load_functions: BTreeMap<u8, String>,

    pub royalty_config: RoyaltyConfig,
    pub function_auth: BTreeMap<String, AccessRule>,
    pub template: MethodAuthTemplate,
}

impl Default for BlueprintDefinitionInit {
    fn default() -> Self {
        Self {
            outer_blueprint: None,
            dependencies: BTreeSet::default(),
            feature_set: BTreeSet::default(),
            blueprint: BlueprintStateSchemaInit::default(),
            event_schema: BlueprintEventSchemaInit::default(),
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


#[cfg_attr(feature = "radix_engine_fuzzing", derive(Arbitrary))]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Ord, PartialOrd, ScryptoSbor, ManifestSbor)]
pub struct SchemaMethodKey {
    pub module_id: u8,
    pub ident: String,
}

impl SchemaMethodKey {
    pub fn main<S: ToString>(method_ident: S) -> Self {
        Self {
            module_id: 0u8,
            ident: method_ident.to_string(),
        }
    }

    pub fn metadata<S: ToString>(method_ident: S) -> Self {
        Self {
            module_id: 1u8,
            ident: method_ident.to_string(),
        }
    }

    pub fn royalty<S: ToString>(method_ident: S) -> Self {
        Self {
            module_id: 2u8,
            ident: method_ident.to_string(),
        }
    }
}

#[cfg_attr(feature = "radix_engine_fuzzing", derive(Arbitrary))]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Ord, PartialOrd, ScryptoSbor, ManifestSbor)]
pub enum SchemaMethodPermission {
    Public,
    Protected(Vec<String>),
}

impl<const N: usize> From<[&str; N]> for SchemaMethodPermission {
    fn from(value: [&str; N]) -> Self {
        SchemaMethodPermission::Protected(
            value.to_vec().into_iter().map(|s| s.to_string()).collect(),
        )
    }
}