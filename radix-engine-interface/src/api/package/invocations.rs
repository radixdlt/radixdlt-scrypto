use crate::api::types::*;
use crate::blueprints::resource::*;
use crate::data::scrypto::model::*;
use crate::*;
use sbor::rust::collections::BTreeMap;
use sbor::rust::string::String;
use sbor::rust::vec::Vec;
use scrypto_abi::BlueprintAbi;

pub struct PackageAbi;

impl PackageAbi {
    pub fn blueprint_abis() -> BTreeMap<String, BlueprintAbi> {
        BTreeMap::new()
    }
}
pub const PACKAGE_LOADER_BLUEPRINT: &str = "PackageLoader";

pub const PACKAGE_LOADER_PUBLISH_WASM_IDENT: &str = "publish_wasm";

#[derive(
    Debug, Clone, Eq, PartialEq, ScryptoSbor, ManifestCategorize, ManifestEncode, ManifestDecode,
)]
pub struct PackageLoaderPublishWasmInput {
    pub package_address: Option<[u8; 26]>, // TODO: Clean this up
    pub code: Vec<u8>,
    pub abi: BTreeMap<String, BlueprintAbi>,
    pub royalty_config: BTreeMap<String, RoyaltyConfig>,
    pub metadata: BTreeMap<String, String>,
    pub access_rules: AccessRules,
}

pub const PACKAGE_LOADER_PUBLISH_PRECOMPILED_IDENT: &str = "publish_precompiled";

#[derive(
    Debug, Clone, Eq, PartialEq, ScryptoSbor, ManifestCategorize, ManifestEncode, ManifestDecode,
)]
pub struct PackageLoaderPublishPrecompiledInput {
    pub package_address: Option<[u8; 26]>, // TODO: Clean this up
    pub native_package_code_id: u8,
    pub abi: BTreeMap<String, BlueprintAbi>,
    pub dependent_resources: Vec<ResourceAddress>,
    pub dependent_components: Vec<ComponentAddress>,
    pub metadata: BTreeMap<String, String>,
    pub access_rules: AccessRules,

    pub package_access_rules: BTreeMap<FunctionKey, AccessRule>,
    pub default_package_access_rule: AccessRule,
}

pub const TRANSACTION_PROCESSOR_BLUEPRINT: &str = "TransactionProcessor";

pub const TRANSACTION_PROCESSOR_RUN_IDENT: &str = "run";
