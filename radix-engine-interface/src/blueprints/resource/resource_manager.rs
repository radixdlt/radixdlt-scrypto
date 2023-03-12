use crate::blueprints::resource::*;
use crate::data::scrypto::model::*;
use crate::math::*;
use crate::*;
use sbor::rust::collections::BTreeMap;
use sbor::rust::string::String;

pub const FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT: &str = "FungibleResourceManager";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, ScryptoSbor, ManifestSbor)]
pub enum ResourceMethodAuthKey {
    Mint,
    Burn,
    UpdateNonFungibleData,
    UpdateMetadata,
    Withdraw,
    Deposit,
    Recall,
}

pub const FUNGIBLE_RESOURCE_MANAGER_CREATE_IDENT: &str = "create_fungible";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor, ManifestSbor)]
pub struct FungibleResourceManagerCreateInput {
    pub divisibility: u8,
    pub metadata: BTreeMap<String, String>,
    pub access_rules: BTreeMap<ResourceMethodAuthKey, (AccessRule, AccessRule)>,
}

pub type FungibleResourceManagerCreateOutput = ResourceAddress;

pub const FUNGIBLE_RESOURCE_MANAGER_CREATE_WITH_INITIAL_SUPPLY_IDENT: &str =
    "create_fungible_with_initial_supply";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor, ManifestSbor)]
pub struct FungibleResourceManagerCreateWithInitialSupplyInput {
    pub divisibility: u8,
    pub metadata: BTreeMap<String, String>,
    pub access_rules: BTreeMap<ResourceMethodAuthKey, (AccessRule, AccessRule)>,
    pub initial_supply: Decimal,
}

pub type FungibleResourceManagerCreateWithInitialSupplyOutput = (ResourceAddress, Bucket);

pub const FUNGIBLE_RESOURCE_MANAGER_CREATE_WITH_INITIAL_SUPPLY_AND_ADDRESS_IDENT: &str =
    "create_fungible_with_initial_supply_and_address";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor, ManifestSbor)]
pub struct FungibleResourceManagerCreateWithInitialSupplyAndAddressInput {
    pub divisibility: u8,
    pub metadata: BTreeMap<String, String>,
    pub access_rules: BTreeMap<ResourceMethodAuthKey, (AccessRule, AccessRule)>,
    pub initial_supply: Decimal,
    pub resource_address: [u8; 26], // TODO: Clean this up
}

pub type FungibleResourceManagerCreateWithInitialSupplyAndAddressOutput = (ResourceAddress, Bucket);

pub const RESOURCE_MANAGER_BURN_IDENT: &str = "burn";
pub const FUNGIBLE_RESOURCE_MANAGER_BURN_EXPORT_NAME: &str = "burn_FungibleResourceManager";
pub const NON_FUNGIBLE_RESOURCE_MANAGER_BURN_EXPORT_NAME: &str = "burn_NonFungibleResourceManager";

#[derive(Debug, Eq, PartialEq, ScryptoSbor)]
pub struct ResourceManagerBurnInput {
    pub bucket: Bucket,
}

pub type ResourceManagerBurnOutput = ();

pub const RESOURCE_MANAGER_CREATE_VAULT_IDENT: &str = "create_vault";
pub const FUNGIBLE_RESOURCE_MANAGER_CREATE_VAULT_EXPORT_NAME: &str =
    "create_vault_FungibleResourceManager";
pub const NON_FUNGIBLE_RESOURCE_MANAGER_CREATE_VAULT_EXPORT_NAME: &str =
    "create_vault_NonFungibleResourceManager";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct ResourceManagerCreateVaultInput {}

pub type ResourceManagerCreateVaultOutput = Vault;

pub const RESOURCE_MANAGER_CREATE_BUCKET_IDENT: &str = "create_bucket";
pub const FUNGIBLE_RESOURCE_MANAGER_CREATE_BUCKET_EXPORT_NAME: &str =
    "create_bucket_FungibleResourceManager";
pub const NON_FUNGIBLE_RESOURCE_MANAGER_CREATE_BUCKET_EXPORT_NAME: &str =
    "create_bucket_NonFungibleResourceManager";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct ResourceManagerCreateBucketInput {}

pub type ResourceManagerCreateBucketOutput = Bucket;

pub const FUNGIBLE_RESOURCE_MANAGER_MINT_IDENT: &str = "mint_fungible";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct FungibleResourceManagerMintInput {
    pub amount: Decimal,
}

pub type FungibleResourceManagerMintOutput = Bucket;

pub const RESOURCE_MANAGER_GET_RESOURCE_TYPE_IDENT: &str = "get_resource_type";
pub const NON_FUNGIBLE_RESOURCE_MANAGER_GET_RESOURCE_TYPE_EXPORT_NAME: &str =
    "get_resource_type_NonFungibleResourceManager";
pub const FUNGIBLE_RESOURCE_MANAGER_GET_RESOURCE_TYPE_EXPORT_NAME: &str =
    "get_resource_type_FungibleResourceManager";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct ResourceManagerGetResourceTypeInput {}

pub type ResourceManagerGetResourceTypeOutput = ResourceType;

pub const RESOURCE_MANAGER_GET_TOTAL_SUPPLY_IDENT: &str = "get_total_supply";
pub const NON_FUNGIBLE_RESOURCE_MANAGER_GET_TOTAL_SUPPLY_EXPORT_NAME: &str =
    "get_total_supply_NonFungibleResourceManager";
pub const FUNGIBLE_RESOURCE_MANAGER_GET_TOTAL_SUPPLY_EXPORT_NAME: &str =
    "get_total_supply_FungibleResourceManager";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct ResourceManagerGetTotalSupplyInput {}

pub type ResourceManagerGetTotalSupplyOutput = Decimal;
