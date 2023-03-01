use crate::blueprints::resource::*;
use crate::data::scrypto::model::*;
use crate::math::*;
use crate::*;
use sbor::rust::collections::{BTreeMap, BTreeSet};
use sbor::rust::string::String;
use sbor::rust::vec::Vec;
use scrypto_schema::PackageSchema;

pub struct ResourceManagerAbi;

impl ResourceManagerAbi {
    pub fn schema() -> PackageSchema {
        PackageSchema::default()
    }
}

pub const RESOURCE_MANAGER_BLUEPRINT: &str = "ResourceManager";

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    PartialOrd,
    Ord,
    ScryptoSbor,
    ManifestCategorize,
    ManifestEncode,
    ManifestDecode,
)]
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

#[derive(
    Debug, Clone, Eq, PartialEq, ScryptoSbor, ManifestCategorize, ManifestEncode, ManifestDecode,
)]
pub struct ResourceManagerCreateFungibleInput {
    pub divisibility: u8,
    pub metadata: BTreeMap<String, String>,
    pub access_rules: BTreeMap<ResourceMethodAuthKey, (AccessRule, AccessRule)>,
}

pub const RESOURCE_MANAGER_CREATE_FUNGIBLE_WITH_INITIAL_SUPPLY_IDENT: &str =
    "create_fungible_with_initial_supply";

#[derive(
    Debug, Clone, Eq, PartialEq, ScryptoSbor, ManifestCategorize, ManifestEncode, ManifestDecode,
)]
pub struct ResourceManagerCreateFungibleWithInitialSupplyInput {
    pub divisibility: u8,
    pub metadata: BTreeMap<String, String>,
    pub access_rules: BTreeMap<ResourceMethodAuthKey, (AccessRule, AccessRule)>,
    pub initial_supply: Decimal,
}

pub const RESOURCE_MANAGER_CREATE_FUNGIBLE_WITH_INITIAL_SUPPLY_AND_ADDRESS_IDENT: &str =
    "create_fungible_with_initial_supply_and_address";

#[derive(
    Debug, Clone, Eq, PartialEq, ScryptoSbor, ManifestCategorize, ManifestEncode, ManifestDecode,
)]
pub struct ResourceManagerCreateFungibleWithInitialSupplyAndAddressInput {
    pub divisibility: u8,
    pub metadata: BTreeMap<String, String>,
    pub access_rules: BTreeMap<ResourceMethodAuthKey, (AccessRule, AccessRule)>,
    pub initial_supply: Decimal,
    pub resource_address: [u8; 26], // TODO: Clean this up
}

pub const RESOURCE_MANAGER_CREATE_NON_FUNGIBLE_IDENT: &str = "create_non_fungible";

#[derive(
    Debug, Clone, Eq, PartialEq, ScryptoSbor, ManifestCategorize, ManifestEncode, ManifestDecode,
)]
pub struct ResourceManagerCreateNonFungibleInput {
    pub id_type: NonFungibleIdType,
    pub metadata: BTreeMap<String, String>,
    pub access_rules: BTreeMap<ResourceMethodAuthKey, (AccessRule, AccessRule)>,
}

pub const RESOURCE_MANAGER_CREATE_NON_FUNGIBLE_WITH_INITIAL_SUPPLY_IDENT: &str =
    "create_non_fungible_with_initial_supply";

#[derive(
    Debug, Clone, Eq, PartialEq, ScryptoSbor, ManifestCategorize, ManifestEncode, ManifestDecode,
)]
pub struct ResourceManagerCreateNonFungibleWithInitialSupplyInput {
    pub id_type: NonFungibleIdType,
    pub metadata: BTreeMap<String, String>,
    pub access_rules: BTreeMap<ResourceMethodAuthKey, (AccessRule, AccessRule)>,
    pub entries: BTreeMap<NonFungibleLocalId, (Vec<u8>, Vec<u8>)>,
}

pub const RESOURCE_MANAGER_CREATE_NON_FUNGIBLE_WITH_ADDRESS_IDENT: &str =
    "create_non_fungible_with_address";

#[derive(
    Debug, Clone, Eq, PartialEq, ScryptoSbor, ManifestCategorize, ManifestEncode, ManifestDecode,
)]
pub struct ResourceManagerCreateNonFungibleWithAddressInput {
    pub id_type: NonFungibleIdType,
    pub metadata: BTreeMap<String, String>,
    pub access_rules: BTreeMap<ResourceMethodAuthKey, (AccessRule, AccessRule)>,
    pub resource_address: [u8; 26], // TODO: Clean this up
}

pub const RESOURCE_MANAGER_CREATE_UUID_NON_FUNGIBLE_WITH_INITIAL_SUPPLY: &str =
    "create_uuid_non_fungible_with_initial_supply";

#[derive(
    Debug, Clone, Eq, PartialEq, ScryptoSbor, ManifestCategorize, ManifestEncode, ManifestDecode,
)]
pub struct ResourceManagerCreateUuidNonFungibleWithInitialSupplyInput {
    pub metadata: BTreeMap<String, String>,
    pub access_rules: BTreeMap<ResourceMethodAuthKey, (AccessRule, AccessRule)>,
    pub entries: BTreeSet<(Vec<u8>, Vec<u8>)>,
}

pub const RESOURCE_MANAGER_BURN_BUCKET_IDENT: &str = "burn_bucket";

#[derive(Debug, Eq, PartialEq, ScryptoSbor)]
pub struct ResourceManagerBurnBucketInput {
    pub bucket: Bucket,
}

impl Clone for ResourceManagerBurnBucketInput {
    fn clone(&self) -> Self {
        Self {
            bucket: Bucket(self.bucket.0),
        }
    }
}

pub const RESOURCE_MANAGER_BURN_IDENT: &str = "burn";

#[derive(Debug, Eq, PartialEq, ScryptoSbor)]
pub struct ResourceManagerBurnInput {
    pub bucket: Bucket,
}

pub const RESOURCE_MANAGER_CREATE_VAULT_IDENT: &str = "create_vault";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct ResourceManagerCreateVaultInput {}

pub const RESOURCE_MANAGER_CREATE_BUCKET_IDENT: &str = "create_bucket";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct ResourceManagerCreateBucketInput {}

pub const RESOURCE_MANAGER_UPDATE_NON_FUNGIBLE_DATA_IDENT: &str = "update_non_fungible_data";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct ResourceManagerUpdateNonFungibleDataInput {
    pub id: NonFungibleLocalId,
    pub data: Vec<u8>,
}

pub const RESOURCE_MANAGER_NON_FUNGIBLE_EXISTS_IDENT: &str = "non_fungible_exists";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct ResourceManagerNonFungibleExistsInput {
    pub id: NonFungibleLocalId,
}

pub const RESOURCE_MANAGER_GET_NON_FUNGIBLE_IDENT: &str = "get_non_fungible";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct ResourceManagerGetNonFungibleInput {
    pub id: NonFungibleLocalId,
}

pub const RESOURCE_MANAGER_MINT_NON_FUNGIBLE: &str = "mint_non_fungible";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct ResourceManagerMintNonFungibleInput {
    pub entries: BTreeMap<NonFungibleLocalId, (Vec<u8>, Vec<u8>)>,
}

pub const RESOURCE_MANAGER_MINT_UUID_NON_FUNGIBLE: &str = "mint_uuid_non_fungible";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct ResourceManagerMintUuidNonFungibleInput {
    pub entries: Vec<(Vec<u8>, Vec<u8>)>,
}

pub const RESOURCE_MANAGER_MINT_FUNGIBLE: &str = "mint_fungible";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct ResourceManagerMintFungibleInput {
    pub amount: Decimal,
}

pub const RESOURCE_MANAGER_GET_RESOURCE_TYPE_IDENT: &str = "get_resource_type";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct ResourceManagerGetResourceTypeInput {}

pub const RESOURCE_MANAGER_GET_TOTAL_SUPPLY_IDENT: &str = "get_total_supply";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct ResourceManagerGetTotalSupplyInput {}
