use sbor::*;

use crate::engine::types::*;
use crate::rust::collections::HashMap;
use crate::rust::string::String;
use crate::rust::vec::Vec;

#[cfg(target_arch = "wasm32")]
extern "C" {
    /// Entrance to Radix Engine.
    pub fn radix_engine(op: u32, input_ptr: *const u8, input_len: usize) -> *mut u8;
}

/// Publish a code package
pub const PUBLISH_PACKAGE: u32 = 0x00;
/// Call a function
pub const CALL_FUNCTION: u32 = 0x01;
/// Call a method
pub const CALL_METHOD: u32 = 0x02;

/// Create a component
pub const CREATE_COMPONENT: u32 = 0x10;
/// Retrieve component information
pub const GET_COMPONENT_INFO: u32 = 0x11;
/// Retrieve component state
pub const GET_COMPONENT_STATE: u32 = 0x12;
/// Update component state
pub const PUT_COMPONENT_STATE: u32 = 0x13;

/// Create a lazy map
pub const CREATE_LAZY_MAP: u32 = 0x20;
/// Retrieve an entry from a lazy map
pub const GET_LAZY_MAP_ENTRY: u32 = 0x21;
/// Insert a key-value pair into a lazy map
pub const PUT_LAZY_MAP_ENTRY: u32 = 0x22;

/// Create resource
pub const CREATE_RESOURCE: u32 = 0x30;
/// Mint resource
pub const MINT_RESOURCE: u32 = 0x31;
/// Burn resource
pub const BURN_RESOURCE: u32 = 0x32;
/// Get resource type
pub const GET_RESOURCE_TYPE: u32 = 0x33;
/// Get resource metadata
pub const GET_RESOURCE_METADATA: u32 = 0x34;
/// Get resource supply
pub const GET_RESOURCE_TOTAL_SUPPLY: u32 = 0x35;
/// Get feature flags
pub const GET_RESOURCE_FLAGS: u32 = 0x36;
/// Update feature flags
pub const UPDATE_RESOURCE_FLAGS: u32 = 0x37;
/// Get mutable feature flags
pub const GET_RESOURCE_MUTABLE_FLAGS: u32 = 0x38;
/// Update mutable feature flags
pub const UPDATE_RESOURCE_MUTABLE_FLAGS: u32 = 0x39;
/// Get the data of a non-fungible
pub const GET_NON_FUNGIBLE_DATA: u32 = 0x3a;
/// Update the data of a non-fungible
pub const UPDATE_NON_FUNGIBLE_MUTABLE_DATA: u32 = 0x3b;
/// Update resource metadata
pub const UPDATE_RESOURCE_METADATA: u32 = 0x3c;

/// Create an empty vault
pub const CREATE_EMPTY_VAULT: u32 = 0x40;
/// Put fungible resource into this vault
pub const PUT_INTO_VAULT: u32 = 0x41;
/// Take fungible resource from this vault
pub const TAKE_FROM_VAULT: u32 = 0x42;
/// Get vault resource amount
pub const GET_VAULT_AMOUNT: u32 = 0x43;
/// Get vault resource definition
pub const GET_VAULT_RESOURCE_DEF_REF: u32 = 0x44;
/// Take a non-fungible from this vault, by key
pub const TAKE_NON_FUNGIBLE_FROM_VAULT: u32 = 0x45;
/// Get the IDs of all non-fungibles in this vault
pub const GET_NON_FUNGIBLE_KEYS_IN_VAULT: u32 = 0x46;

/// Create an empty bucket
pub const CREATE_EMPTY_BUCKET: u32 = 0x50;
/// Put fungible resource into this bucket
pub const PUT_INTO_BUCKET: u32 = 0x51;
/// Take fungible resource from this bucket
pub const TAKE_FROM_BUCKET: u32 = 0x52;
/// Get bucket resource amount
pub const GET_BUCKET_AMOUNT: u32 = 0x53;
/// Get bucket resource definition
pub const GET_BUCKET_RESOURCE_DEF_REF: u32 = 0x54;
/// Take a non-fungible from this bucket, by key
pub const TAKE_NON_FUNGIBLE_FROM_BUCKET: u32 = 0x55;
/// Get the IDs of all non-fungibles in this bucket
pub const GET_NON_FUNGIBLE_KEYS_IN_BUCKET: u32 = 0x56;

/// Obtain a bucket ref
pub const CREATE_BUCKET_REF: u32 = 0x60;
/// Drop a bucket ref
pub const DROP_BUCKET_REF: u32 = 0x61;
/// Get the resource amount behind a bucket ref
pub const GET_BUCKET_REF_AMOUNT: u32 = 0x62;
/// Get the resource definition behind a bucket ref
pub const GET_BUCKET_REF_RESOURCE_DEF_REF: u32 = 0x63;
/// Get the non-fungible keys in the bucket referenced
pub const GET_NON_FUNGIBLE_KEYS_IN_BUCKET_REF: u32 = 0x64;
/// Clone bucket ref
pub const CLONE_BUCKET_REF: u32 = 0x65;

/// Log a message
pub const EMIT_LOG: u32 = 0xf0;
/// Generate a UUID
pub const GENERATE_UUID: u32 = 0xf1;
/// Retrieve call data
pub const GET_CALL_DATA: u32 = 0xf2;
/// Retrieve current current_epoch
pub const GET_CURRENT_EPOCH: u32 = 0xf3;
/// Retrieve transaction hash
pub const GET_TRANSACTION_HASH: u32 = 0xf4;
/// Retrieve the running entity
pub const GET_ACTOR: u32 = 0xf5;

//==========
// blueprint
//==========

#[derive(Debug, TypeId, Encode, Decode)]
pub struct PublishPackageInput {
    pub code: Vec<u8>,
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct PublishPackageOutput {
    pub package_ref: PackageRef,
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct CallFunctionInput {
    pub package_ref: PackageRef,
    pub blueprint_name: String,
    pub function: String,
    pub args: Vec<Vec<u8>>,
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct CallFunctionOutput {
    pub rtn: Vec<u8>,
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct CallMethodInput {
    pub component_ref: ComponentRef,
    pub method: String,
    pub args: Vec<Vec<u8>>,
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct CallMethodOutput {
    pub rtn: Vec<u8>,
}

//==========
// component
//==========

#[derive(Debug, TypeId, Encode, Decode)]
pub struct CreateComponentInput {
    pub package_ref: PackageRef,
    pub blueprint_name: String,
    pub state: Vec<u8>,
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct CreateComponentOutput {
    pub component_ref: ComponentRef,
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct GetComponentInfoInput {
    pub component_ref: ComponentRef,
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct GetComponentInfoOutput {
    pub package_ref: PackageRef,
    pub blueprint_name: String,
}

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct GetComponentStateInput {}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct GetComponentStateOutput {
    pub state: Vec<u8>,
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct PutComponentStateInput {
    pub state: Vec<u8>,
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct PutComponentStateOutput {}

//==========
// LazyMap
//==========

#[derive(Debug, TypeId, Encode, Decode)]
pub struct CreateLazyMapInput {}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct CreateLazyMapOutput {
    pub lazy_map_id: LazyMapId,
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct GetLazyMapEntryInput {
    pub lazy_map_id: LazyMapId,
    pub key: Vec<u8>,
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct GetLazyMapEntryOutput {
    pub value: Option<Vec<u8>>,
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct PutLazyMapEntryInput {
    pub lazy_map_id: LazyMapId,
    pub key: Vec<u8>,
    pub value: Vec<u8>,
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct PutLazyMapEntryOutput {}

//=========
// resource
//=========

#[derive(Debug, TypeId, Encode, Decode)]
pub struct CreateResourceInput {
    pub resource_type: ResourceType,
    pub metadata: HashMap<String, String>,
    pub flags: u64,
    pub mutable_flags: u64,
    pub authorities: HashMap<ResourceDefRef, u64>,
    pub initial_supply: Option<Supply>,
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct CreateResourceOutput {
    pub resource_def_ref: ResourceDefRef,
    pub bucket_id: Option<BucketId>,
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct MintResourceInput {
    pub resource_def_ref: ResourceDefRef,
    pub new_supply: Supply,
    pub auth: BucketRefId,
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct MintResourceOutput {
    pub bucket_id: BucketId,
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct BurnResourceInput {
    pub bucket_id: BucketId,
    pub auth: Option<BucketRefId>,
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct BurnResourceOutput {}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct GetResourceMetadataInput {
    pub resource_def_ref: ResourceDefRef,
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct GetResourceMetadataOutput {
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct GetResourceTypeInput {
    pub resource_def_ref: ResourceDefRef,
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct GetResourceTypeOutput {
    pub resource_type: ResourceType,
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct GetResourceTotalSupplyInput {
    pub resource_def_ref: ResourceDefRef,
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct GetResourceTotalSupplyOutput {
    pub total_supply: Decimal,
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct GetNonFungibleDataInput {
    pub resource_def_ref: ResourceDefRef,
    pub key: NonFungibleKey,
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct GetNonFungibleDataOutput {
    pub immutable_data: Vec<u8>,
    pub mutable_data: Vec<u8>,
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct UpdateNonFungibleMutableDataInput {
    pub resource_def_ref: ResourceDefRef,
    pub key: NonFungibleKey,
    pub new_mutable_data: Vec<u8>,
    pub auth: BucketRefId,
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct UpdateNonFungibleMutableDataOutput {}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct GetResourceFlagsInput {
    pub resource_def_ref: ResourceDefRef,
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct GetResourceFlagsOutput {
    pub flags: u64,
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct UpdateResourceFlagsInput {
    pub resource_def_ref: ResourceDefRef,
    pub new_flags: u64,
    pub auth: BucketRefId,
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct UpdateResourceFlagsOutput {}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct GetResourceMutableFlagsInput {
    pub resource_def_ref: ResourceDefRef,
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct GetResourceMutableFlagsOutput {
    pub mutable_flags: u64,
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct UpdateResourceMutableFlagsInput {
    pub resource_def_ref: ResourceDefRef,
    pub new_mutable_flags: u64,
    pub auth: BucketRefId,
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct UpdateResourceMutableFlagsOutput {}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct UpdateResourceMetadataInput {
    pub resource_def_ref: ResourceDefRef,
    pub new_metadata: HashMap<String, String>,
    pub auth: BucketRefId,
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct UpdateResourceMetadataOutput {}

//==========
// vault
//==========

#[derive(Debug, TypeId, Encode, Decode)]
pub struct CreateEmptyVaultInput {
    pub resource_def_ref: ResourceDefRef,
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct CreateEmptyVaultOutput {
    pub vault_id: VaultId,
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct PutIntoVaultInput {
    pub vault_id: VaultId,
    pub bucket_id: BucketId,
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct PutIntoVaultOutput {}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct TakeFromVaultInput {
    pub vault_id: VaultId,
    pub amount: Decimal,
    pub auth: Option<BucketRefId>,
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct TakeFromVaultOutput {
    pub bucket_id: BucketId,
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct GetVaultDecimalInput {
    pub vault_id: VaultId,
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct GetVaultDecimalOutput {
    pub amount: Decimal,
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct GetVaultResourceDefRefInput {
    pub vault_id: VaultId,
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct GetVaultResourceDefRefOutput {
    pub resource_def_ref: ResourceDefRef,
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct TakeNonFungibleFromVaultInput {
    pub vault_id: VaultId,
    pub key: NonFungibleKey,
    pub auth: Option<BucketRefId>,
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct TakeNonFungibleFromVaultOutput {
    pub bucket_id: BucketId,
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct GetNonFungibleKeysInVaultInput {
    pub vault_id: VaultId,
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct GetNonFungibleKeysInVaultOutput {
    pub keys: Vec<NonFungibleKey>,
}

//==========
// bucket
//==========

#[derive(Debug, TypeId, Encode, Decode)]
pub struct CreateEmptyBucketInput {
    pub resource_def_ref: ResourceDefRef,
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct CreateEmptyBucketOutput {
    pub bucket_id: BucketId,
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct PutIntoBucketInput {
    pub bucket_id: BucketId,
    pub other: BucketId,
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct PutIntoBucketOutput {}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct TakeFromBucketInput {
    pub bucket_id: BucketId,
    pub amount: Decimal,
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct TakeFromBucketOutput {
    pub bucket_id: BucketId,
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct GetBucketDecimalInput {
    pub bucket_id: BucketId,
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct GetBucketDecimalOutput {
    pub amount: Decimal,
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct GetBucketResourceDefRefInput {
    pub bucket_id: BucketId,
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct GetBucketResourceDefRefOutput {
    pub resource_def_ref: ResourceDefRef,
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct TakeNonFungibleFromBucketInput {
    pub bucket_id: BucketId,
    pub key: NonFungibleKey,
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct TakeNonFungibleFromBucketOutput {
    pub bucket_id: BucketId,
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct GetNonFungibleKeysInBucketInput {
    pub bucket_id: BucketId,
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct GetNonFungibleKeysInBucketOutput {
    pub keys: Vec<NonFungibleKey>,
}

//==========
// bucket ref
//==========

#[derive(Debug, TypeId, Encode, Decode)]
pub struct CreateBucketRefInput {
    pub bucket_id: BucketId,
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct CreateBucketRefOutput {
    pub bucket_ref_id: BucketRefId,
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct DropBucketRefInput {
    pub bucket_ref_id: BucketRefId,
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct DropBucketRefOutput {}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct GetBucketRefDecimalInput {
    pub bucket_ref_id: BucketRefId,
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct GetBucketRefDecimalOutput {
    pub amount: Decimal,
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct GetBucketRefResourceDefRefInput {
    pub bucket_ref_id: BucketRefId,
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct GetBucketRefResourceDefRefOutput {
    pub resource_def_ref: ResourceDefRef,
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct GetNonFungibleKeysInBucketRefInput {
    pub bucket_ref_id: BucketRefId,
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct GetNonFungibleKeysInBucketRefOutput {
    pub keys: Vec<NonFungibleKey>,
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct CloneBucketRefInput {
    pub bucket_ref_id: BucketRefId,
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct CloneBucketRefOutput {
    pub bucket_ref_id: BucketRefId,
}

//=======
// others
//=======

#[derive(Debug, TypeId, Encode, Decode)]
pub struct EmitLogInput {
    pub level: Level,
    pub message: String,
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct EmitLogOutput {}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct GetCallDataInput {}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct GetCallDataOutput {
    pub function: String,
    pub args: Vec<Vec<u8>>,
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct GetCurrentEpochInput {}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct GetCurrentEpochOutput {
    pub current_epoch: u64,
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct GetTransactionHashInput {}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct GetTransactionHashOutput {
    pub transaction_hash: Hash,
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct GetTransactionSignersInput {}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct GenerateUuidInput {}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct GenerateUuidOutput {
    pub uuid: u128,
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct GetActorInput {}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct GetActorOutput {
    pub actor: Actor,
}
