use crate::blueprints::resource::*;
use crate::*;
#[cfg(feature = "radix_engine_fuzzing")]
use arbitrary::Arbitrary;
use radix_engine_common::data::manifest::model::ManifestOwn;
use radix_engine_common::types::*;
use radix_engine_interface::api::node_modules::metadata::MetadataValue;
use sbor::rust::collections::BTreeMap;
use sbor::rust::string::String;

pub const FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT: &str = "FungibleResourceManager";

pub const FUNGIBLE_RESOURCE_MANAGER_CREATE_IDENT: &str = "create";

#[cfg_attr(feature = "radix_engine_fuzzing", derive(Arbitrary))]
#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor, ManifestSbor)]
pub struct FungibleResourceManagerCreateInput {
    pub track_total_supply: bool,
    pub divisibility: u8,
    pub metadata: BTreeMap<String, MetadataValue>,
    pub access_rules: BTreeMap<ResourceMethodAuthKey, (AccessRule, AccessRule)>,
}

pub type FungibleResourceManagerCreateOutput = ResourceAddress;

pub const FUNGIBLE_RESOURCE_MANAGER_CREATE_WITH_INITIAL_SUPPLY_IDENT: &str =
    "create_with_initial_supply";

#[cfg_attr(feature = "radix_engine_fuzzing", derive(Arbitrary))]
#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor, ManifestSbor)]
pub struct FungibleResourceManagerCreateWithInitialSupplyInput {
    pub track_total_supply: bool,
    pub divisibility: u8,
    pub metadata: BTreeMap<String, MetadataValue>,
    pub access_rules: BTreeMap<ResourceMethodAuthKey, (AccessRule, AccessRule)>,
    pub initial_supply: Decimal,
}

pub type FungibleResourceManagerCreateWithInitialSupplyOutput = (ResourceAddress, Bucket);

pub const FUNGIBLE_RESOURCE_MANAGER_CREATE_WITH_INITIAL_SUPPLY_AND_ADDRESS_IDENT: &str =
    "create_with_initial_supply_and_address";

#[derive(Debug, Eq, PartialEq, ScryptoSbor)]
pub struct FungibleResourceManagerCreateWithInitialSupplyAndAddressInput {
    pub track_total_supply: bool,
    pub divisibility: u8,
    pub metadata: BTreeMap<String, MetadataValue>,
    pub access_rules: BTreeMap<ResourceMethodAuthKey, (AccessRule, AccessRule)>,
    pub initial_supply: Decimal,
    pub resource_address: GlobalAddressReservation,
}

#[derive(Debug, Clone, Eq, PartialEq, ManifestSbor)]
pub struct FungibleResourceManagerCreateWithInitialSupplyAndAddressManifestInput {
    pub track_total_supply: bool,
    pub divisibility: u8,
    pub metadata: BTreeMap<String, MetadataValue>,
    pub access_rules: BTreeMap<ResourceMethodAuthKey, (AccessRule, AccessRule)>,
    pub initial_supply: Decimal,
    pub resource_address: ManifestOwn,
}

pub type FungibleResourceManagerCreateWithInitialSupplyAndAddressOutput = (ResourceAddress, Bucket);

pub const FUNGIBLE_RESOURCE_MANAGER_MINT_IDENT: &str = "mint";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor, ManifestSbor)]
pub struct FungibleResourceManagerMintInput {
    pub amount: Decimal,
}

pub type FungibleResourceManagerMintOutput = Bucket;
