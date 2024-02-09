use crate::resource::*;
use radix_engine_common::data::manifest::model::ManifestAddressReservation;
use radix_engine_common::data::manifest::model::ManifestBlobRef;
use radix_engine_common::prelude::*;

use super::PackageDefinition;

pub const PACKAGE_BLUEPRINT: &str = "Package";

pub const PACKAGE_PUBLISH_WASM_IDENT: &str = "publish_wasm";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor, ManifestSbor)]
pub struct PackagePublishWasmInput {
    pub definition: PackageDefinition,
    pub code: Vec<u8>,
    pub metadata: MetadataInit,
}

#[derive(Debug, Clone, Eq, PartialEq, ManifestSbor)]
pub struct PackagePublishWasmManifestInput {
    pub definition: PackageDefinition,
    pub code: ManifestBlobRef,
    pub metadata: MetadataInit,
}

pub type PackagePublishWasmOutput = (PackageAddress, Bucket);

pub const PACKAGE_PUBLISH_WASM_ADVANCED_IDENT: &str = "publish_wasm_advanced";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct PackagePublishWasmAdvancedInput {
    pub owner_role: OwnerRole,
    pub definition: PackageDefinition,
    pub code: Vec<u8>,
    pub metadata: MetadataInit,
    pub package_address: Option<GlobalAddressReservation>,
}

#[derive(Debug, Clone, Eq, PartialEq, ManifestSbor)]
pub struct PackagePublishWasmAdvancedManifestInput {
    pub owner_role: OwnerRole,
    pub definition: PackageDefinition,
    pub code: ManifestBlobRef,
    pub metadata: MetadataInit,
    pub package_address: Option<ManifestAddressReservation>,
}

pub type PackagePublishWasmAdvancedOutput = PackageAddress;

pub const PACKAGE_PUBLISH_NATIVE_IDENT: &str = "publish_native";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct PackagePublishNativeInput {
    pub definition: PackageDefinition,
    pub native_package_code_id: u64,
    pub metadata: MetadataInit,
    pub package_address: Option<GlobalAddressReservation>,
}

#[derive(Debug, Clone, Eq, PartialEq, ManifestSbor)]
pub struct PackagePublishNativeManifestInput {
    pub definition: PackageDefinition,
    pub native_package_code_id: u64,
    pub metadata: MetadataInit,
    pub package_address: Option<ManifestAddressReservation>,
}

pub type PackagePublishNativeOutput = PackageAddress;

pub const PACKAGE_CLAIM_ROYALTIES_IDENT: &str = "PackageRoyalty_claim_royalties";

#[cfg_attr(
    feature = "radix_engine_fuzzing",
    derive(arbitrary::Arbitrary, serde::Serialize, serde::Deserialize)
)]
#[derive(
    Debug, Clone, Eq, PartialEq, ScryptoSbor, ManifestCategorize, ManifestEncode, ManifestDecode,
)]
pub struct PackageClaimRoyaltiesInput {}

pub type PackageClaimRoyaltiesOutput = Bucket;
