use sbor::{Decode, Encode, TypeId};

use crate::kernel::*;
use crate::rust::collections::HashMap;
use crate::rust::string::String;
use crate::rust::vec::Vec;
use crate::types::*;

#[cfg(target_arch = "wasm32")]
extern "C" {
    /// Entrance to Radix Engine.
    pub fn kernel(op: u32, input_ptr: *const u8, input_len: usize) -> *mut u8;
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
/// Get the data of an NFT
pub const GET_NFT_DATA: u32 = 0x3a;
/// Update the data of an NFT
pub const UPDATE_NFT_MUTABLE_DATA: u32 = 0x3b;
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
pub const GET_VAULT_RESOURCE_ADDRESS: u32 = 0x44;
/// Take an NFT from this vault, by key
pub const TAKE_NFT_FROM_VAULT: u32 = 0x45;
/// Get the IDs of all NFTs in this vault
pub const GET_NFT_IDS_IN_VAULT: u32 = 0x46;

/// Create an empty bucket
pub const CREATE_EMPTY_BUCKET: u32 = 0x50;
/// Put fungible resource into this bucket
pub const PUT_INTO_BUCKET: u32 = 0x51;
/// Take fungible resource from this bucket
pub const TAKE_FROM_BUCKET: u32 = 0x52;
/// Get bucket resource amount
pub const GET_BUCKET_AMOUNT: u32 = 0x53;
/// Get bucket resource definition
pub const GET_BUCKET_RESOURCE_ADDRESS: u32 = 0x54;
/// Take an NFT from this bucket, by key
pub const TAKE_NFT_FROM_BUCKET: u32 = 0x55;
/// Get the IDs of all NFTs in this bucket
pub const GET_NFT_IDS_IN_BUCKET: u32 = 0x56;

/// Obtain a bucket ref
pub const CREATE_BUCKET_REF: u32 = 0x60;
/// Drop a bucket ref
pub const DROP_BUCKET_REF: u32 = 0x61;
/// Get the resource amount behind a bucket ref
pub const GET_BUCKET_REF_AMOUNT: u32 = 0x62;
/// Get the resource definition behind a bucket ref
pub const GET_BUCKET_REF_RESOURCE_DEF: u32 = 0x63;
/// Get the NFT ids in the bucket referenced
pub const GET_NFT_IDS_IN_BUCKET_REF: u32 = 0x64;
/// Clone bucket ref
pub const CLONE_BUCKET_REF: u32 = 0x65;

/// Log a message
pub const EMIT_LOG: u32 = 0xf0;
/// Retrieve context package address
pub const GET_PACKAGE_ADDRESS: u32 = 0xf1;
/// Retrieve call data
pub const GET_CALL_DATA: u32 = 0xf2;
/// Retrieve current current_epoch
pub const GET_CURRENT_EPOCH: u32 = 0xf3;
/// Retrieve transaction hash
pub const GET_TRANSACTION_HASH: u32 = 0xf4;
/// Generate an UUID
pub const GENERATE_UUID: u32 = 0xf5;

//==========
// blueprint
//==========

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct PublishPackageInput {
    pub code: Vec<u8>,
}

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct PublishPackageOutput {
    pub package_address: Address,
}

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct CallFunctionInput {
    pub package_address: Address,
    pub blueprint_name: String,
    pub function: String,
    pub args: Vec<Vec<u8>>,
}

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct CallFunctionOutput {
    pub rtn: Vec<u8>,
}

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct CallMethodInput {
    pub component_address: Address,
    pub method: String,
    pub args: Vec<Vec<u8>>,
}

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct CallMethodOutput {
    pub rtn: Vec<u8>,
}

//==========
// component
//==========

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct CreateComponentInput {
    pub blueprint_name: String,
    pub state: Vec<u8>,
}

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct CreateComponentOutput {
    pub component_address: Address,
}

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct GetComponentInfoInput {
    pub component_address: Address,
}

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct GetComponentInfoOutput {
    pub package_address: Address,
    pub blueprint_name: String,
}

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct GetComponentStateInput {
    pub component_address: Address,
}

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct GetComponentStateOutput {
    pub state: Vec<u8>,
}

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct PutComponentStateInput {
    pub component_address: Address,
    pub state: Vec<u8>,
}

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct PutComponentStateOutput {}

//==========
// LazyMap
//==========

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct CreateLazyMapInput {}

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct CreateLazyMapOutput {
    pub mid: Mid,
}

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct GetLazyMapEntryInput {
    pub mid: Mid,
    pub key: Vec<u8>,
}

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct GetLazyMapEntryOutput {
    pub value: Option<Vec<u8>>,
}

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct PutLazyMapEntryInput {
    pub mid: Mid,
    pub key: Vec<u8>,
    pub value: Vec<u8>,
}

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct PutLazyMapEntryOutput {}

//=========
// resource
//=========

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct CreateResourceInput {
    pub resource_type: ResourceType,
    pub metadata: HashMap<String, String>,
    pub flags: u16,
    pub mutable_flags: u16,
    pub authorities: HashMap<Address, u16>,
    pub initial_supply: Option<NewSupply>,
}

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct CreateResourceOutput {
    pub resource_address: Address,
    pub bucket: Option<Bid>,
}

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct MintResourceInput {
    pub resource_address: Address,
    pub new_supply: NewSupply,
    pub auth: Rid,
}

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct MintResourceOutput {
    pub bid: Bid,
}

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct BurnResourceInput {
    pub bid: Bid,
    pub auth: Option<Rid>,
}

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct BurnResourceOutput {}

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct GetResourceMetadataInput {
    pub resource_address: Address,
}

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct GetResourceMetadataOutput {
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct GetResourceTypeInput {
    pub resource_address: Address,
}

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct GetResourceTypeOutput {
    pub resource_type: ResourceType,
}

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct GetResourceTotalSupplyInput {
    pub resource_address: Address,
}

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct GetResourceTotalSupplyOutput {
    pub total_supply: Decimal,
}

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct GetNftDataInput {
    pub resource_address: Address,
    pub key: NftKey,
}

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct GetNftDataOutput {
    pub immutable_data: Vec<u8>,
    pub mutable_data: Vec<u8>,
}

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct UpdateNftMutableDataInput {
    pub resource_address: Address,
    pub key: NftKey,
    pub new_mutable_data: Vec<u8>,
    pub auth: Rid,
}

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct UpdateNftMutableDataOutput {}

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct GetResourceFlagsInput {
    pub resource_address: Address,
}

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct GetResourceFlagsOutput {
    pub flags: u16,
}

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct UpdateResourceFlagsInput {
    pub resource_address: Address,
    pub new_flags: u16,
    pub auth: Rid,
}

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct UpdateResourceFlagsOutput {}

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct GetResourceMutableFlagsInput {
    pub resource_address: Address,
}

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct GetResourceMutableFlagsOutput {
    pub mutable_flags: u16,
}

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct UpdateResourceMutableFlagsInput {
    pub resource_address: Address,
    pub new_mutable_flags: u16,
    pub auth: Rid,
}

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct UpdateResourceMutableFlagsOutput {}

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct UpdateResourceMetadataInput {
    pub resource_address: Address,
    pub new_metadata: HashMap<String, String>,
    pub auth: Rid,
}

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct UpdateResourceMetadataOutput {}

//==========
// vault
//==========

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct CreateEmptyVaultInput {
    pub resource_address: Address,
}

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct CreateEmptyVaultOutput {
    pub vid: Vid,
}

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct PutIntoVaultInput {
    pub vid: Vid,
    pub bid: Bid,
}

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct PutIntoVaultOutput {}

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct TakeFromVaultInput {
    pub vid: Vid,
    pub amount: Decimal,
    pub auth: Option<Rid>,
}

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct TakeFromVaultOutput {
    pub bid: Bid,
}

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct GetVaultDecimalInput {
    pub vid: Vid,
}

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct GetVaultDecimalOutput {
    pub amount: Decimal,
}

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct GetVaultResourceAddressInput {
    pub vid: Vid,
}

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct GetVaultResourceAddressOutput {
    pub resource_address: Address,
}

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct TakeNftFromVaultInput {
    pub vid: Vid,
    pub key: NftKey,
    pub auth: Option<Rid>,
}

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct TakeNftFromVaultOutput {
    pub bid: Bid,
}

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct GetNftIdsInVaultInput {
    pub vid: Vid,
}

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct GetNftIdsInVaultOutput {
    pub ids: Vec<NftKey>,
}

//==========
// bucket
//==========

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct CreateEmptyBucketInput {
    pub resource_address: Address,
}

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct CreateEmptyBucketOutput {
    pub bid: Bid,
}

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct PutIntoBucketInput {
    pub bid: Bid,
    pub other: Bid,
}

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct PutIntoBucketOutput {}

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct TakeFromBucketInput {
    pub bid: Bid,
    pub amount: Decimal,
}

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct TakeFromBucketOutput {
    pub bid: Bid,
}

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct GetBucketDecimalInput {
    pub bid: Bid,
}

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct GetBucketDecimalOutput {
    pub amount: Decimal,
}

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct GetBucketResourceAddressInput {
    pub bid: Bid,
}

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct GetBucketResourceAddressOutput {
    pub resource_address: Address,
}

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct TakeNftFromBucketInput {
    pub bid: Bid,
    pub key: NftKey,
}

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct TakeNftFromBucketOutput {
    pub bid: Bid,
}

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct GetNftIdsInBucketInput {
    pub bid: Bid,
}

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct GetNftIdsInBucketOutput {
    pub ids: Vec<NftKey>,
}

//==========
// bucket ref
//==========

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct CreateBucketRefInput {
    pub bid: Bid,
}

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct CreateBucketRefOutput {
    pub rid: Rid,
}

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct DropBucketRefInput {
    pub rid: Rid,
}

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct DropBucketRefOutput {}

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct GetBucketRefDecimalInput {
    pub rid: Rid,
}

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct GetBucketRefDecimalOutput {
    pub amount: Decimal,
}

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct GetBucketRefResourceAddressInput {
    pub rid: Rid,
}

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct GetBucketRefResourceAddressOutput {
    pub resource_address: Address,
}

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct GetNftIdsInBucketRefInput {
    pub rid: Rid,
}

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct GetNftIdsInBucketRefOutput {
    pub ids: Vec<NftKey>,
}

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct CloneBucketRefInput {
    pub rid: Rid,
}

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct CloneBucketRefOutput {
    pub rid: Rid,
}

//=======
// others
//=======

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct EmitLogInput {
    pub level: LogLevel,
    pub message: String,
}

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct EmitLogOutput {}

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct GetPackageAddressInput {}

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct GetPackageAddressOutput {
    pub package_address: Address,
}

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct GetCallDataInput {}

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct GetCallDataOutput {
    pub function: String,
    pub args: Vec<Vec<u8>>,
}

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct GetCurrentEpochInput {}

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct GetCurrentEpochOutput {
    pub current_epoch: u64,
}

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct GetTransactionHashInput {}

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct GetTransactionHashOutput {
    pub transaction_hash: H256,
}

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct GetTransactionSignersInput {}

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct GenerateUuidInput {}

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct GenerateUuidOutput {
    pub uuid: u128,
}
