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

/// Create a new component
pub const CREATE_COMPONENT: u32 = 0x10;
/// Retrieve component information
pub const GET_COMPONENT_INFO: u32 = 0x11;
/// Retrieve component state
pub const GET_COMPONENT_STATE: u32 = 0x12;
/// Update component state
pub const PUT_COMPONENT_STATE: u32 = 0x13;

/// Create a new storage
pub const CREATE_STORAGE: u32 = 0x20;
/// Retrieve an entry from a storage
pub const GET_STORAGE_ENTRY: u32 = 0x21;
/// Insert a key-value pair into a storage
pub const PUT_STORAGE_ENTRY: u32 = 0x22;

/// Create a new resource with mutable supply
pub const CREATE_RESOURCE_MUTABLE: u32 = 0x30;
/// Create a new resource with fixed supply
pub const CREATE_RESOURCE_FIXED: u32 = 0x31;
/// Retrieve resource information
pub const GET_RESOURCE_INFO: u32 = 0x32;
/// Mint resource
pub const MINT_RESOURCE: u32 = 0x33;

/// Create a new empty vault
pub const CREATE_EMPTY_VAULT: u32 = 0x40;
/// Combine vaults
pub const PUT_INTO_VAULT: u32 = 0x41;
/// Split a vault
pub const TAKE_FROM_VAULT: u32 = 0x42;
/// Get vault resource amount
pub const GET_VAULT_AMOUNT: u32 = 0x43;
/// Get vault resource address
pub const GET_VAULT_RESOURCE: u32 = 0x44;

/// Create a new empty bucket
pub const CREATE_EMPTY_BUCKET: u32 = 0x50;
/// Combine buckets
pub const PUT_INTO_BUCKET: u32 = 0x51;
/// Split a bucket
pub const TAKE_FROM_BUCKET: u32 = 0x52;
/// Get bucket resource amount
pub const GET_BUCKET_AMOUNT: u32 = 0x53;
/// Get bucket resource address
pub const GET_BUCKET_RESOURCE: u32 = 0x54;

/// Obtain an immutable reference to a bucket
pub const CREATE_REFERENCE: u32 = 0x60;
/// Drop a reference
pub const DROP_REFERENCE: u32 = 0x61;
/// Get the resource amount behind a reference
pub const GET_REF_AMOUNT: u32 = 0x62;
/// Get the resource address behind a reference
pub const GET_REF_RESOURCE: u32 = 0x63;

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
pub struct GetComponentInfoInput {
    pub component: Address,
}

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct GetComponentInfoOutput {
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
// Storage
//==========

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct CreateStorageInput {}

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct CreateStorageOutput {
    pub storage: SID,
}

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct GetStorageEntryInput {
    pub storage: SID,
    pub key: Vec<u8>,
}

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct GetStorageEntryOutput {
    pub value: Option<Vec<u8>>,
}

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct PutStorageEntryInput {
    pub storage: SID,
    pub key: Vec<u8>,
    pub value: Vec<u8>,
}

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct PutStorageEntryOutput {}

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
    pub resource: Address,
}

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct CreateResourceFixedInput {
    pub metadata: HashMap<String, String>,
    pub supply: Amount,
}

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct CreateResourceFixedOutput {
    pub bucket: BID,
}

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct GetResourceInfoInput {
    pub resource: Address,
}

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct GetResourceInfoOutput {
    pub metadata: HashMap<String, String>,
    pub minter: Option<Address>,
    pub supply: Amount,
}

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct MintResourceInput {
    pub resource: Address,
    pub amount: Amount,
}

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct MintResourceOutput {
    pub bucket: BID,
}

//==========
// vault
//==========

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct CreateEmptyVaultInput {
    pub resource: Address,
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
pub struct GetVaultResourceInput {
    pub vault: VID,
}

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct GetVaultResourceOutput {
    pub resource: Address,
}

//==========
// bucket
//==========

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct CreateEmptyBucketInput {
    pub resource: Address,
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
pub struct GetBucketResourceInput {
    pub bucket: BID,
}

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct GetBucketResourceOutput {
    pub resource: Address,
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
pub struct GetRefResourceInput {
    pub reference: RID,
}

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct GetRefResourceOutput {
    pub resource: Address,
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
