use sbor::{Decode, Encode, TypeId};

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
pub const GET_COMPONENT_BLUEPRINT: u32 = 0x11;
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

/// Create resource with mutable supply
pub const CREATE_RESOURCE_MUTABLE: u32 = 0x30;
/// Create resource with fixed supply
pub const CREATE_RESOURCE_FIXED: u32 = 0x31;
/// Get resource metadata
pub const GET_RESOURCE_METADATA: u32 = 0x32;
/// Get resource supply
pub const GET_RESOURCE_SUPPLY: u32 = 0x33;
/// Get resource mint auth
pub const GET_RESOURCE_MINT_BURN_AUTH: u32 = 0x34;
/// Mint resource
pub const MINT_RESOURCE: u32 = 0x35;
/// Burn resource
pub const BURN_RESOURCE: u32 = 0x36;

/// Create a new empty vault
pub const CREATE_EMPTY_VAULT: u32 = 0x40;
/// Combine vaults
pub const PUT_INTO_VAULT: u32 = 0x41;
/// Split a vault
pub const TAKE_FROM_VAULT: u32 = 0x42;
/// Get vault resource amount
pub const GET_VAULT_AMOUNT: u32 = 0x43;
/// Get vault resource definition
pub const GET_VAULT_RESOURCE_DEF: u32 = 0x44;

/// Create a new empty bucket
pub const CREATE_EMPTY_BUCKET: u32 = 0x50;
/// Combine buckets
pub const PUT_INTO_BUCKET: u32 = 0x51;
/// Split a bucket
pub const TAKE_FROM_BUCKET: u32 = 0x52;
/// Get bucket resource amount
pub const GET_BUCKET_AMOUNT: u32 = 0x53;
/// Get bucket resource definition
pub const GET_BUCKET_RESOURCE_DEF: u32 = 0x54;

/// Obtain a bucket ref
pub const CREATE_BUCKET_REF: u32 = 0x60;
/// Drop a bucket ref
pub const DROP_BUCKET_REF: u32 = 0x61;
/// Get the resource amount behind a bucket ref
pub const GET_BUCKET_REF_AMOUNT: u32 = 0x62;
/// Get the resource definition behind a bucket ref
pub const GET_BUCKET_REF_RESOURCE_DEF: u32 = 0x63;

/// Log a message
pub const EMIT_LOG: u32 = 0xf0;
/// Retrieve context package address
pub const GET_PACKAGE_ADDRESS: u32 = 0xf1;
/// Retrieve call data
pub const GET_CALL_DATA: u32 = 0xf2;
/// Retrieve transaction hash
pub const GET_TRANSACTION_HASH: u32 = 0xf3;
/// Retrieve current current_epoch
pub const GET_CURRENT_EPOCH: u32 = 0xf4;
/// Retrieve transaction signers
pub const GET_TRANSACTION_SIGNERS: u32 = 0xf5;

//==========
// blueprint
//==========

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct PublishPackageInput {
    pub code: Vec<u8>,
}

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct PublishPackageOutput {
    pub package: Address,
}

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct CallFunctionInput {
    pub package: Address,
    pub name: String,
    pub function: String,
    pub args: Vec<Vec<u8>>,
}

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct CallFunctionOutput {
    pub rtn: Vec<u8>,
}

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct CallMethodInput {
    pub component: Address,
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
    pub name: String,
    pub state: Vec<u8>,
}

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct CreateComponentOutput {
    pub component: Address,
}

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct GetComponentBlueprintInput {
    pub component: Address,
}

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct GetComponentBlueprintOutput {
    pub package: Address,
    pub name: String,
}

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct GetComponentStateInput {
    pub component: Address,
}

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct GetComponentStateOutput {
    pub state: Vec<u8>,
}

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct PutComponentStateInput {
    pub component: Address,
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
    pub lazy_map: Mid,
}

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct GetLazyMapEntryInput {
    pub lazy_map: Mid,
    pub key: Vec<u8>,
}

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct GetLazyMapEntryOutput {
    pub value: Option<Vec<u8>>,
}

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct PutLazyMapEntryInput {
    pub lazy_map: Mid,
    pub key: Vec<u8>,
    pub value: Vec<u8>,
}

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct PutLazyMapEntryOutput {}

//=========
// resource
//=========

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct CreateResourceMutableInput {
    pub metadata: HashMap<String, String>,
    pub mint_burn_auth: Address,
}

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct CreateResourceMutableOutput {
    pub resource_def: Address,
}

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct CreateResourceFixedInput {
    pub metadata: HashMap<String, String>,
    pub supply: Decimal,
}

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct CreateResourceFixedOutput {
    pub resource_def: Address,
    pub bucket: Bid,
}

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct GetResourceMetadataInput {
    pub resource_def: Address,
}

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct GetResourceMetadataOutput {
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct GetResourceSupplyInput {
    pub resource_def: Address,
}

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct GetResourceSupplyOutput {
    pub supply: Decimal,
}

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct GetResourceMintAuthInput {
    pub resource_def: Address,
}

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct GetResourceMintAuthOutput {
    pub mint_burn_auth: Option<Address>,
}

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct MintResourceInput {
    pub resource_def: Address,
    pub amount: Decimal,
    pub mint_burn_auth: Rid,
}

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct MintResourceOutput {
    pub bucket: Bid,
}

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct BurnResourceInput {
    pub bucket: Bid,
    pub mint_burn_auth: Rid,
}

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct BurnResourceOutput {}

//==========
// vault
//==========

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct CreateEmptyVaultInput {
    pub resource_def: Address,
}

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct CreateEmptyVaultOutput {
    pub vault: Vid,
}

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct PutIntoVaultInput {
    pub vault: Vid,
    pub bucket: Bid,
}

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct PutIntoVaultOutput {}

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct TakeFromVaultInput {
    pub vault: Vid,
    pub amount: Decimal,
}

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct TakeFromVaultOutput {
    pub bucket: Bid,
}

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct GetVaultDecimalInput {
    pub vault: Vid,
}

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct GetVaultDecimalOutput {
    pub amount: Decimal,
}

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct GetVaultResourceAddressInput {
    pub vault: Vid,
}

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct GetVaultResourceAddressOutput {
    pub resource_def: Address,
}

//==========
// bucket
//==========

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct CreateEmptyBucketInput {
    pub resource_def: Address,
}

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct CreateEmptyBucketOutput {
    pub bucket: Bid,
}

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct PutIntoBucketInput {
    pub bucket: Bid,
    pub other: Bid,
}

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct PutIntoBucketOutput {}

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct TakeFromBucketInput {
    pub bucket: Bid,
    pub amount: Decimal,
}

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct TakeFromBucketOutput {
    pub bucket: Bid,
}

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct GetBucketDecimalInput {
    pub bucket: Bid,
}

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct GetBucketDecimalOutput {
    pub amount: Decimal,
}

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct GetBucketResourceAddressInput {
    pub bucket: Bid,
}

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct GetBucketResourceAddressOutput {
    pub resource_def: Address,
}

//==========
// bucket ref
//==========

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct CreateBucketRefInput {
    pub bucket: Bid,
}

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct CreateBucketRefOutput {
    pub bucket_ref: Rid,
}

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct DropBucketRefInput {
    pub bucket_ref: Rid,
}

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct DropBucketRefOutput {}

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct GetBucketRefDecimalInput {
    pub bucket_ref: Rid,
}

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct GetBucketRefDecimalOutput {
    pub amount: Decimal,
}

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct GetBucketRefResourceDefInput {
    pub bucket_ref: Rid,
}

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct GetBucketRefResourceDefOutput {
    pub resource_def: Address,
}

//=======
// others
//=======

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct EmitLogInput {
    pub level: u8,
    pub message: String,
}

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct EmitLogOutput {}

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct GetPackageAddressInput {}

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct GetPackageAddressOutput {
    pub address: Address,
}

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct GetCallDataInput {}

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct GetCallDataOutput {
    pub function: String,
    pub args: Vec<Vec<u8>>,
}

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct GetTransactionHashInput {}

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct GetTransactionHashOutput {
    pub tx_hash: H256,
}

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct GetCurrentEpochInput {}

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct GetCurrentEpochOutput {
    pub current_epoch: u64,
}

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct GetTransactionSignersInput {}

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct GetTransactionSignersOutput {
    pub tx_signers: Vec<Address>,
}
