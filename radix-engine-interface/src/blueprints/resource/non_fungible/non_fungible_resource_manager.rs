use crate::blueprints::resource::*;
use crate::data::scrypto::model::*;
use crate::*;
use radix_engine_common::data::manifest::ManifestValue;
use radix_engine_common::data::scrypto::{ScryptoCustomTypeKind, ScryptoSchema, ScryptoValue};
use radix_engine_common::types::*;
use radix_engine_interface::types::NonFungibleData;
use sbor::rust::collections::{BTreeMap, BTreeSet};
use sbor::rust::string::String;
use sbor::rust::string::ToString;
use sbor::rust::vec::Vec;
use sbor::{generate_full_schema, LocalTypeIndex, TypeAggregator};

pub const NON_FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT: &str = "NonFungibleResourceManager";

pub const NON_FUNGIBLE_RESOURCE_MANAGER_CREATE_IDENT: &str = "create";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor, ManifestSbor)]
pub struct NonFungibleResourceManagerCreateInput {
    pub id_type: NonFungibleIdType,
    pub non_fungible_schema: NonFungibleDataSchema,
    pub metadata: BTreeMap<String, String>,
    pub access_rules: BTreeMap<ResourceMethodAuthKey, (AccessRule, AccessRule)>,
}

pub type NonFungibleResourceManagerCreateOutput = ResourceAddress;

pub const NON_FUNGIBLE_RESOURCE_MANAGER_CREATE_WITH_INITIAL_SUPPLY_IDENT: &str =
    "create_with_initial_supply";

#[derive(Debug, Clone, Eq, PartialEq, ManifestSbor)]
pub struct NonFungibleResourceManagerCreateWithInitialSupplyManifestInput {
    pub id_type: NonFungibleIdType,
    pub non_fungible_schema: NonFungibleDataSchema,
    pub metadata: BTreeMap<String, String>,
    pub access_rules: BTreeMap<ResourceMethodAuthKey, (AccessRule, AccessRule)>,
    pub entries: BTreeMap<NonFungibleLocalId, (ManifestValue,)>,
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct NonFungibleResourceManagerCreateWithInitialSupplyInput {
    pub id_type: NonFungibleIdType,
    pub non_fungible_schema: NonFungibleDataSchema,
    pub metadata: BTreeMap<String, String>,
    pub access_rules: BTreeMap<ResourceMethodAuthKey, (AccessRule, AccessRule)>,
    pub entries: BTreeMap<NonFungibleLocalId, (ScryptoValue,)>,
}

pub type NonFungibleResourceManagerCreateWithInitialSupplyOutput = (ResourceAddress, Bucket);

pub const NON_FUNGIBLE_RESOURCE_MANAGER_CREATE_WITH_ADDRESS_IDENT: &str =
    "create_non_fungible_with_address";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor, ManifestSbor)]
pub struct NonFungibleResourceManagerCreateWithAddressInput {
    pub id_type: NonFungibleIdType,
    pub non_fungible_schema: NonFungibleDataSchema,
    pub metadata: BTreeMap<String, String>,
    pub access_rules: BTreeMap<ResourceMethodAuthKey, (AccessRule, AccessRule)>,
    pub resource_address: [u8; 27], // TODO: Clean this up
}

pub type NonFungibleResourceManagerCreateWithAddressOutput = ResourceAddress;

pub const NON_FUNGIBLE_RESOURCE_MANAGER_CREATE_UUID_WITH_INITIAL_SUPPLY_IDENT: &str =
    "create_uuid_non_fungible_with_initial_supply";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct NonFungibleResourceManagerCreateUuidWithInitialSupplyInput {
    pub non_fungible_schema: NonFungibleDataSchema,
    pub metadata: BTreeMap<String, String>,
    pub access_rules: BTreeMap<ResourceMethodAuthKey, (AccessRule, AccessRule)>,
    pub entries: Vec<(ScryptoValue,)>,
}

pub type NonFungibleResourceManagerCreateUuidWithInitialSupplyOutput = (ResourceAddress, Bucket);

pub const NON_FUNGIBLE_RESOURCE_MANAGER_UPDATE_DATA_IDENT: &str = "update_non_fungible_data";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct NonFungibleResourceManagerUpdateDataInput {
    pub id: NonFungibleLocalId,
    pub field_name: String,
    pub data: ScryptoValue,
}

pub type NonFungibleResourceManagerUpdateDataOutput = ();

pub const NON_FUNGIBLE_RESOURCE_MANAGER_EXISTS_IDENT: &str = "non_fungible_exists";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct NonFungibleResourceManagerExistsInput {
    pub id: NonFungibleLocalId,
}

pub type NonFungibleResourceManagerExistsOutput = bool;

pub const NON_FUNGIBLE_RESOURCE_MANAGER_GET_NON_FUNGIBLE_IDENT: &str = "get_non_fungible";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct NonFungibleResourceManagerGetNonFungibleInput {
    pub id: NonFungibleLocalId,
}

pub type NonFungibleResourceManagerGetNonFungibleOutput = ScryptoValue;

pub const NON_FUNGIBLE_RESOURCE_MANAGER_MINT_IDENT: &str = "mint";

#[derive(Debug, Clone, Eq, PartialEq, ManifestSbor)]
pub struct NonFungibleResourceManagerMintManifestInput {
    pub entries: BTreeMap<NonFungibleLocalId, (ManifestValue,)>,
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct NonFungibleResourceManagerMintInput {
    pub entries: BTreeMap<NonFungibleLocalId, (ScryptoValue,)>,
}

pub type NonFungibleResourceManagerMintOutput = Bucket;

pub const NON_FUNGIBLE_RESOURCE_MANAGER_MINT_UUID_IDENT: &str = "mint_uuid";

#[derive(Debug, Clone, Eq, PartialEq, ManifestSbor)]
pub struct NonFungibleResourceManagerMintUuidManifestInput {
    pub entries: Vec<(ManifestValue,)>,
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct NonFungibleResourceManagerMintUuidInput {
    pub entries: Vec<(ScryptoValue,)>,
}

pub type NonFungibleResourceManagerMintUuidOutput = Bucket;

pub const NON_FUNGIBLE_RESOURCE_MANAGER_MINT_SINGLE_UUID_IDENT: &str = "mint_single_uuid";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct NonFungibleResourceManagerMintSingleUuidInput {
    pub entry: ScryptoValue,
}
pub type NonFungibleResourceManagerMintSingleUuidOutput = (Bucket, NonFungibleLocalId);

#[derive(Debug, Clone, PartialEq, Eq, Sbor)]
pub struct NonFungibleDataSchema {
    pub schema: ScryptoSchema,
    pub non_fungible: LocalTypeIndex,
    pub mutable_fields: BTreeSet<String>,
}

impl NonFungibleData for () {
    const MUTABLE_FIELDS: &'static [&'static str] = &[];
}

impl NonFungibleDataSchema {
    pub fn new_schema<N: NonFungibleData>() -> Self {
        let mut aggregator = TypeAggregator::<ScryptoCustomTypeKind>::new();
        let non_fungible_type = aggregator.add_child_type_and_descendents::<N>();
        let schema = generate_full_schema(aggregator);
        Self {
            schema,
            non_fungible: non_fungible_type,
            mutable_fields: N::MUTABLE_FIELDS.iter().map(|s| s.to_string()).collect(),
        }
    }
}
