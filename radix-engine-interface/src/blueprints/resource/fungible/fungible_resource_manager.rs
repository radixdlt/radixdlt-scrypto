use crate::blueprints::resource::*;
use crate::math::*;
use crate::*;
use radix_engine_common::types::*;
use sbor::rust::collections::BTreeMap;
use sbor::rust::string::String;

pub const FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT: &str = "FungibleResourceManager";

pub const FUNGIBLE_RESOURCE_MANAGER_CREATE_IDENT: &str = "create";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor, ManifestSbor)]
pub struct FungibleResourceManagerCreateInput {
    pub divisibility: u8,
    pub metadata: BTreeMap<String, String>,
    pub access_rules: BTreeMap<ResourceMethodAuthKey, (AccessRule, AccessRule)>,
}

pub type FungibleResourceManagerCreateOutput = ResourceAddress;

pub const FUNGIBLE_RESOURCE_MANAGER_CREATE_WITH_INITIAL_SUPPLY_IDENT: &str =
    "create_with_initial_supply";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor, ManifestSbor)]
pub struct FungibleResourceManagerCreateWithInitialSupplyInput {
    pub divisibility: u8,
    pub metadata: BTreeMap<String, String>,
    pub access_rules: BTreeMap<ResourceMethodAuthKey, (AccessRule, AccessRule)>,
    pub initial_supply: Decimal,
}

pub type FungibleResourceManagerCreateWithInitialSupplyOutput = (ResourceAddress, Bucket);

pub const FUNGIBLE_RESOURCE_MANAGER_CREATE_WITH_INITIAL_SUPPLY_AND_ADDRESS_IDENT: &str =
    "create_with_initial_supply_and_address";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor, ManifestSbor)]
pub struct FungibleResourceManagerCreateWithInitialSupplyAndAddressInput {
    pub divisibility: u8,
    pub metadata: BTreeMap<String, String>,
    pub access_rules: BTreeMap<ResourceMethodAuthKey, (AccessRule, AccessRule)>,
    pub initial_supply: Decimal,
    pub resource_address: [u8; 27], // TODO: Clean this up
}

pub type FungibleResourceManagerCreateWithInitialSupplyAndAddressOutput = (ResourceAddress, Bucket);

pub const FUNGIBLE_RESOURCE_MANAGER_MINT_IDENT: &str = "mint";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct FungibleResourceManagerMintInput {
    pub amount: Decimal,
}

pub type FungibleResourceManagerMintOutput = Bucket;
