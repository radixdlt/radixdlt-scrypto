use radix_engine_common::data::scrypto::ScryptoValue;
use crate::blueprints::resource::*;
use crate::data::scrypto::model::*;
use crate::math::*;
use crate::*;
use sbor::rust::collections::BTreeMap;
use sbor::rust::string::String;
use sbor::rust::vec::Vec;
use scrypto_schema::NonFungibleSchema;

pub const RESOURCE_MANAGER_BLUEPRINT: &str = "ResourceManager";

pub const NON_FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT: &str = "NonFungibleResourceManager";

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

pub const RESOURCE_MANAGER_CREATE_FUNGIBLE_IDENT: &str = "create_fungible";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor, ManifestSbor)]
pub struct ResourceManagerCreateFungibleInput {
    pub divisibility: u8,
    pub metadata: BTreeMap<String, String>,
    pub access_rules: BTreeMap<ResourceMethodAuthKey, (AccessRule, AccessRule)>,
}

pub type ResourceManagerCreateFungibleOutput = ResourceAddress;

pub const RESOURCE_MANAGER_CREATE_FUNGIBLE_WITH_INITIAL_SUPPLY_IDENT: &str =
    "create_fungible_with_initial_supply";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor, ManifestSbor)]
pub struct ResourceManagerCreateFungibleWithInitialSupplyInput {
    pub divisibility: u8,
    pub metadata: BTreeMap<String, String>,
    pub access_rules: BTreeMap<ResourceMethodAuthKey, (AccessRule, AccessRule)>,
    pub initial_supply: Decimal,
}

pub type ResourceManagerCreateFungibleWithInitialSupplyOutput = (ResourceAddress, Bucket);

pub const RESOURCE_MANAGER_CREATE_FUNGIBLE_WITH_INITIAL_SUPPLY_AND_ADDRESS_IDENT: &str =
    "create_fungible_with_initial_supply_and_address";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor, ManifestSbor)]
pub struct ResourceManagerCreateFungibleWithInitialSupplyAndAddressInput {
    pub divisibility: u8,
    pub metadata: BTreeMap<String, String>,
    pub access_rules: BTreeMap<ResourceMethodAuthKey, (AccessRule, AccessRule)>,
    pub initial_supply: Decimal,
    pub resource_address: [u8; 26], // TODO: Clean this up
}

pub type ResourceManagerCreateFungibleWithInitialSupplyAndAddressOutput = (ResourceAddress, Bucket);

pub const NON_FUNGIBLE_RESOURCE_MANAGER_CREATE_IDENT: &str = "create_non_fungible";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor, ManifestSbor)]
pub struct ResourceManagerCreateNonFungibleInput {
    pub id_type: NonFungibleIdType,
    pub non_fungible_schema: NonFungibleSchema,
    pub metadata: BTreeMap<String, String>,
    pub access_rules: BTreeMap<ResourceMethodAuthKey, (AccessRule, AccessRule)>,
}

pub type ResourceManagerCreateNonFungibleOutput = ResourceAddress;

pub const NON_FUNGIBLE_RESOURCE_MANAGER_CREATE_WITH_INITIAL_SUPPLY_IDENT: &str =
    "create_non_fungible_with_initial_supply";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor, ManifestSbor)]
pub struct ResourceManagerCreateNonFungibleWithInitialSupplyInput {
    pub id_type: NonFungibleIdType,
    pub non_fungible_schema: NonFungibleSchema,
    pub metadata: BTreeMap<String, String>,
    pub access_rules: BTreeMap<ResourceMethodAuthKey, (AccessRule, AccessRule)>,
    pub entries: BTreeMap<NonFungibleLocalId, Vec<u8>>,
}

pub type ResourceManagerCreateNonFungibleWithInitialSupplyOutput = (ResourceAddress, Bucket);

pub const NON_FUNGIBLE_RESOURCE_MANAGER_CREATE_WITH_ADDRESS_IDENT: &str =
    "create_non_fungible_with_address";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor, ManifestSbor)]
pub struct ResourceManagerCreateNonFungibleWithAddressInput {
    pub id_type: NonFungibleIdType,
    pub non_fungible_schema: NonFungibleSchema,
    pub metadata: BTreeMap<String, String>,
    pub access_rules: BTreeMap<ResourceMethodAuthKey, (AccessRule, AccessRule)>,
    pub resource_address: [u8; 26], // TODO: Clean this up
}

pub type ResourceManagerCreateNonFungibleWithAddressOutput = ResourceAddress;

pub const NON_FUNGIBLE_RESOURCE_MANAGER_CREATE_UUID_WITH_INITIAL_SUPPLY_IDENT: &str =
    "create_uuid_non_fungible_with_initial_supply";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor, ManifestSbor)]
pub struct ResourceManagerCreateUuidNonFungibleWithInitialSupplyInput {
    pub non_fungible_schema: NonFungibleSchema,
    pub metadata: BTreeMap<String, String>,
    pub access_rules: BTreeMap<ResourceMethodAuthKey, (AccessRule, AccessRule)>,
    pub entries: Vec<Vec<u8>>,
}

pub type ResourceManagerCreateUuidNonFungibleWithInitialSupplyOutput = (ResourceAddress, Bucket);

pub const RESOURCE_MANAGER_BURN_BUCKET_IDENT: &str = "burn_bucket";

#[derive(Debug, Eq, PartialEq, ScryptoSbor)]
pub struct ResourceManagerBurnBucketInput {
    pub bucket: Bucket,
}

pub type ResourceManagerBurnBucketOutput = ();

impl Clone for ResourceManagerBurnBucketInput {
    fn clone(&self) -> Self {
        Self {
            bucket: Bucket(self.bucket.0),
        }
    }
}

pub const RESOURCE_MANAGER_BURN_IDENT: &str = "burn";
pub const FUNGIBLE_RESOURCE_MANAGER_BURN_EXPORT_NAME: &str = "burn_FungibleResourceManager";
pub const NON_FUNGIBLE_RESOURCE_MANAGER_BURN_EXPORT_NAME: &str = "burn_NonFungibleResourceManager";

#[derive(Debug, Eq, PartialEq, ScryptoSbor)]
pub struct ResourceManagerBurnInput {
    pub bucket: Bucket,
}

pub type ResourceManagerBurnOutput = ();

pub const RESOURCE_MANAGER_CREATE_VAULT_IDENT: &str = "create_vault";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct ResourceManagerCreateVaultInput {}

pub type ResourceManagerCreateVaultOutput = Vault;

pub const RESOURCE_MANAGER_CREATE_BUCKET_IDENT: &str = "create_bucket";
pub const FUNGIBLE_RESOURCE_MANAGER_CREATE_BUCKET_EXPORT_NAME: &str = "create_bucket_FungibleResourceManager";
pub const NON_FUNGIBLE_RESOURCE_MANAGER_CREATE_BUCKET_EXPORT_NAME: &str = "create_bucket_NonFungibleResourceManager";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct ResourceManagerCreateBucketInput {}

pub type ResourceManagerCreateBucketOutput = Bucket;

pub const RESOURCE_MANAGER_UPDATE_NON_FUNGIBLE_DATA_IDENT: &str = "update_non_fungible_data";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct ResourceManagerUpdateNonFungibleDataInput {
    pub id: NonFungibleLocalId,
    pub data: Vec<u8>,
}

pub type ResourceManagerUpdateNonFungibleDataOutput = ();

pub const RESOURCE_MANAGER_NON_FUNGIBLE_EXISTS_IDENT: &str = "non_fungible_exists";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct ResourceManagerNonFungibleExistsInput {
    pub id: NonFungibleLocalId,
}

pub type ResourceManagerNonFungibleExistsOutput = bool;

pub const RESOURCE_MANAGER_GET_NON_FUNGIBLE_IDENT: &str = "get_non_fungible";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct ResourceManagerGetNonFungibleInput {
    pub id: NonFungibleLocalId,
}

pub type ResourceManagerGetNonFungibleOutput = ScryptoValue;

pub const RESOURCE_MANAGER_MINT_NON_FUNGIBLE_IDENT: &str = "mint_non_fungible";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct ResourceManagerMintNonFungibleInput {
    pub entries: BTreeMap<NonFungibleLocalId, Vec<u8>>,
}

pub type ResourceManagerMintNonFungibleOutput = Bucket;

pub const RESOURCE_MANAGER_MINT_UUID_NON_FUNGIBLE_IDENT: &str = "mint_uuid_non_fungible";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct ResourceManagerMintUuidNonFungibleInput {
    pub entries: Vec<Vec<u8>>,
}

pub type ResourceManagerMintUuidNonFungibleOutput = Bucket;

pub const RESOURCE_MANAGER_MINT_FUNGIBLE_IDENT: &str = "mint_fungible";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct ResourceManagerMintFungibleInput {
    pub amount: Decimal,
}

pub type ResourceManagerMintFungibleOutput = Bucket;

pub const RESOURCE_MANAGER_GET_RESOURCE_TYPE_IDENT: &str = "get_resource_type";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct ResourceManagerGetResourceTypeInput {}

pub type ResourceManagerGetResourceTypeOutput = ResourceType;

pub const RESOURCE_MANAGER_GET_TOTAL_SUPPLY_IDENT: &str = "get_total_supply";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct ResourceManagerGetTotalSupplyInput {}

pub type ResourceManagerGetTotalSupplyOutput = Decimal;
