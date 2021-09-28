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
pub const PUBLISH: u32 = 0x00;
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
/// Get resource minter
pub const GET_RESOURCE_MINTER: u32 = 0x34;
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
/// Get vault resource address
pub const GET_VAULT_RESOURCE_ADDRESS: u32 = 0x44;

/// Create a new empty bucket
pub const CREATE_EMPTY_BUCKET: u32 = 0x50;
/// Combine buckets
pub const PUT_INTO_BUCKET: u32 = 0x51;
/// Split a bucket
pub const TAKE_FROM_BUCKET: u32 = 0x52;
/// Get bucket resource amount
pub const GET_BUCKET_AMOUNT: u32 = 0x53;
/// Get bucket resource address
pub const GET_BUCKET_RESOURCE_ADDRESS: u32 = 0x54;

/// Obtain an immutable reference to a bucket
pub const CREATE_REFERENCE: u32 = 0x60;
/// Drop a reference
pub const DROP_REFERENCE: u32 = 0x61;
/// Get the resource amount behind a reference
pub const GET_REF_AMOUNT: u32 = 0x62;
/// Get the resource address behind a reference
pub const GET_REF_RESOURCE_ADDRESS: u32 = 0x63;

/// Log a message
pub const EMIT_LOG: u32 = 0xf0;
/// Retrieve context package address
pub const GET_PACKAGE_ADDRESS: u32 = 0xf1;
/// Retrieve call data
pub const GET_CALL_DATA: u32 = 0xf2;
/// Retrieve transaction hash
pub const GET_TRANSACTION_HASH: u32 = 0xf3;

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
    pub blueprint: (Address, String),
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
    pub blueprint: (Address, String),
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
    pub lazy_map: MID,
}

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct GetLazyMapEntryInput {
    pub lazy_map: MID,
    pub key: Vec<u8>,
}

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct GetLazyMapEntryOutput {
    pub value: Option<Vec<u8>>,
}

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct PutLazyMapEntryInput {
    pub lazy_map: MID,
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
    pub minter: Address,
}

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct CreateResourceMutableOutput {
    pub resource_address: Address,
}

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct CreateResourceFixedInput {
    pub metadata: HashMap<String, String>,
    pub supply: Amount,
}

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct CreateResourceFixedOutput {
    pub resource_address: Address,
    pub bucket: BID,
}

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct GetResourceMetadataInput {
    pub resource_address: Address,
}

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct GetResourceMetadataOutput {
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct GetResourceSupplyInput {
    pub resource_address: Address,
}

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct GetResourceSupplyOutput {
    pub supply: Amount,
}

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct GetResourceMinterInput {
    pub resource_address: Address,
}

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct GetResourceMinterOutput {
    pub minter: Option<Address>,
}

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct MintResourceInput {
    pub amount: Amount,
    pub resource_address: Address,
}

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct MintResourceOutput {
    pub bucket: BID,
}

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct BurnResourceInput {
    pub bucket: BID,
}

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct BurnResourceOutput {}

//==========
// vault
//==========

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct CreateEmptyVaultInput {
    pub resource_address: Address,
}

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct CreateEmptyVaultOutput {
    pub vault: VID,
}

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct PutIntoVaultInput {
    pub vault: VID,
    pub bucket: BID,
}

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct PutIntoVaultOutput {}

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct TakeFromVaultInput {
    pub vault: VID,
    pub amount: Amount,
}

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct TakeFromVaultOutput {
    pub bucket: BID,
}

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct GetVaultAmountInput {
    pub vault: VID,
}

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct GetVaultAmountOutput {
    pub amount: Amount,
}

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct GetVaultResourceAddressInput {
    pub vault: VID,
}

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct GetVaultResourceAddressOutput {
    pub resource_address: Address,
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
    pub bucket: BID,
}

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct PutIntoBucketInput {
    pub bucket: BID,
    pub other: BID,
}

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct PutIntoBucketOutput {}

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct TakeFromBucketInput {
    pub bucket: BID,
    pub amount: Amount,
}

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct TakeFromBucketOutput {
    pub bucket: BID,
}

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct GetBucketAmountInput {
    pub bucket: BID,
}

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct GetBucketAmountOutput {
    pub amount: Amount,
}

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct GetBucketResourceAddressInput {
    pub bucket: BID,
}

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct GetBucketResourceAddressOutput {
    pub resource_address: Address,
}

//==========
// reference
//==========

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct CreateReferenceInput {
    pub bucket: BID,
}

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct CreateReferenceOutput {
    pub reference: RID,
}

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct DropReferenceInput {
    pub reference: RID,
}

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct DropReferenceOutput {}

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct GetRefAmountInput {
    pub reference: RID,
}

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct GetRefAmountOutput {
    pub amount: Amount,
}

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct GetRefResourceAddressInput {
    pub reference: RID,
}

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct GetRefResourceAddressOutput {
    pub resource_address: Address,
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
