use sbor::{Decode, Encode};

use crate::rust::string::String;
use crate::rust::vec::Vec;
use crate::types::*;

extern "C" {
    /// Entrance to Radix kernel.
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
/// TODO: Retrieve an entry from component storage
pub const GET_COMPONENT_STORAGE: u32 = 0x14;
/// TODO: Insert a key-value pair into component storage
pub const PUT_COMPONENT_STORAGE: u32 = 0x15;

/// Create a new resource with mutable supply
pub const CREATE_RESOURCE_MUTABLE: u32 = 0x20;
/// Create a new resource with fixed supply
pub const CREATE_RESOURCE_FIXED: u32 = 0x21;
/// Retrieve resource information
pub const GET_RESOURCE_INFO: u32 = 0x22;
/// Mint resource
pub const MINT_RESOURCE: u32 = 0x23;

/// Creates a new empty bucket
pub const NEW_EMPTY_BUCKET: u32 = 0x30;
/// Combine buckets
pub const COMBINE_BUCKETS: u32 = 0x31;
/// Split a bucket
pub const SPLIT_BUCKET: u32 = 0x32;
/// Get bucket resource amount
pub const GET_AMOUNT: u32 = 0x33;
/// Get bucket resource address
pub const GET_RESOURCE: u32 = 0x34;
/// Obtain an immutable reference to a bucket
pub const BORROW_IMMUTABLE: u32 = 0x35;
/// TODO: Obtain a mutable reference to a bucket
pub const BORROW_MUTABLE: u32 = 0x36;
/// Drop a reference
pub const DROP_REFERENCE: u32 = 0x37;
/// Get the resource amount behind a reference
pub const GET_AMOUNT_REF: u32 = 0x38;
/// Get the resource address behind a reference
pub const GET_RESOURCE_REF: u32 = 0x39;

/// Log a message
pub const EMIT_LOG: u32 = 0x40;
/// Retrieve context package address
pub const GET_PACKAGE_ADDRESS: u32 = 0x41;
/// Retrieve call data
pub const GET_CALL_DATA: u32 = 0x42;
/// Retrieve transaction hash
pub const GET_TRANSACTION_HASH: u32 = 0x43;

//==========
// code
//==========

#[derive(Debug, Clone, Encode, Decode)]
pub struct PublishPackageInput {
    pub code: Vec<u8>,
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct PublishPackageOutput {
    pub package: Address,
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct CallFunctionInput {
    pub package: Address,
    pub blueprint: String,
    pub function: String,
    pub args: Vec<Vec<u8>>,
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct CallFunctionOutput {
    pub rtn: Vec<u8>,
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct CallMethodInput {
    pub component: Address,
    pub method: String,
    pub args: Vec<Vec<u8>>,
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct CallMethodOutput {
    pub rtn: Vec<u8>,
}

//==========
// component
//==========

#[derive(Debug, Clone, Encode, Decode)]
pub struct CreateComponentInput {
    pub blueprint: String,
    pub state: Vec<u8>,
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct CreateComponentOutput {
    pub component: Address,
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct GetComponentInfoInput {
    pub component: Address,
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct GetComponentInfoOutput {
    pub package: Address,
    pub blueprint: String,
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct GetComponentStateInput {
    pub component: Address,
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct GetComponentStateOutput {
    pub state: Vec<u8>,
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct PutComponentStateInput {
    pub component: Address,
    pub state: Vec<u8>,
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct PutComponentStateOutput {}

#[derive(Debug, Clone, Encode, Decode)]
pub struct GetComponentStorageInput {
    pub component: Address,
    pub key: Vec<u8>,
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct GetComponentStorageOutput {
    pub value: Option<Vec<u8>>,
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct PutComponentStorageInput {
    pub component: Address,
    pub key: Vec<u8>,
    pub value: Vec<u8>,
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct PutComponentStorageOutput {}

//=========
// resource
//=========

#[derive(Debug, Clone, Encode, Decode)]
pub struct CreateResourceMutableInput {
    pub symbol: String,
    pub name: String,
    pub description: String,
    pub url: String,
    pub icon_url: String,
    pub minter: Address,
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct CreateResourceMutableOutput {
    pub resource: Address,
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct CreateResourceFixedInput {
    pub symbol: String,
    pub name: String,
    pub description: String,
    pub url: String,
    pub icon_url: String,
    pub supply: U256,
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct CreateResourceFixedOutput {
    pub resource: Address,
    pub bucket: BID,
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct GetResourceInfoInput {
    pub resource: Address,
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct GetResourceInfoOutput {
    pub symbol: String,
    pub name: String,
    pub description: String,
    pub url: String,
    pub icon_url: String,
    pub minter: Option<Address>,
    pub supply: Option<U256>,
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct MintResourceInput {
    pub resource: Address,
    pub amount: U256,
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct MintResourceOutput {
    pub bucket: BID,
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct NewEmptyBucketInput {
    pub resource: Address,
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct NewEmptyBucketOutput {
    pub bucket: BID,
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct CombineBucketsInput {
    pub bucket: BID,
    pub other: BID,
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct CombineBucketsOutput {}

#[derive(Debug, Clone, Encode, Decode)]
pub struct SplitBucketInput {
    pub bucket: BID,
    pub amount: U256,
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct SplitBucketOutput {
    pub bucket: BID,
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct GetAmountInput {
    pub bucket: BID,
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct GetAmountOutput {
    pub amount: U256,
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct GetResourceInput {
    pub bucket: BID,
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct GetResourceOutput {
    pub resource: Address,
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct BorrowImmutableInput {
    pub bucket: BID,
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct BorrowImmutableOutput {
    pub reference: RID,
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct BorrowMutableInput {
    pub bucket: BID,
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct BorrowMutableOutput {
    pub reference: RID,
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct DropReferenceInput {
    pub reference: RID,
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct DropReferenceOutput {}

#[derive(Debug, Clone, Encode, Decode)]
pub struct GetAmountRefInput {
    pub reference: RID,
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct GetAmountRefOutput {
    pub amount: U256,
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct GetResourceRefInput {
    pub reference: RID,
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct GetResourceRefOutput {
    pub resource: Address,
}

//=======
// others
//=======

#[derive(Debug, Clone, Encode, Decode)]
pub struct EmitLogInput {
    pub level: u8,
    pub message: String,
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct EmitLogOutput {}

#[derive(Debug, Clone, Encode, Decode)]
pub struct GetPackageAddressInput {}

#[derive(Debug, Clone, Encode, Decode)]
pub struct GetPackageAddressOutput {
    pub address: Address,
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct GetCallDataInput {}

#[derive(Debug, Clone, Encode, Decode)]
pub struct GetCallDataOutput {
    pub function: String,
    pub args: Vec<Vec<u8>>,
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct GetTransactionHashInput {}

#[derive(Debug, Clone, Encode, Decode)]
pub struct GetTransactionHashOutput {
    pub tx_hash: H256,
}
