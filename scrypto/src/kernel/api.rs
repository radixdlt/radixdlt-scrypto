use sbor::{Decode, Describe, Encode};

use crate::types::rust::string::String;
use crate::types::rust::vec::Vec;
use crate::types::*;

extern "C" {
    /// Entrance to Radix kernel.
    pub fn kernel(op: u32, input_ptr: *const u8, input_len: usize) -> *mut u8;
}

/// Publish a new blueprint
pub const PUBLISH_BLUEPRINT: u32 = 0x00;
/// Invoke a blueprint
pub const CALL_BLUEPRINT: u32 = 0x01;

/// Create a new component
pub const CREATE_COMPONENT: u32 = 0x10;
/// Retrieve component data
pub const GET_COMPONENT_INFO: u32 = 0x11;
/// Retrieve component state
pub const GET_COMPONENT_STATE: u32 = 0x12;
/// Update component state
pub const PUT_COMPONENT_STATE: u32 = 0x13;
/// TODO: Retrieve an entry from component storage
pub const GET_COMPONENT_STORAGE: u32 = 0x14;
/// TODO: Insert a key-value pair into component storage
pub const PUT_COMPONENT_STORAGE: u32 = 0x15;

/// Create a new resource
pub const CREATE_RESOURCE: u32 = 0x20;
/// Retrieve resource data
pub const GET_RESOURCE_INFO: u32 = 0x21;
/// Mint resource
pub const MINT_RESOURCE: u32 = 0x22;

/// Combine buckets
pub const COMBINE_BUCKETS: u32 = 0x30;
/// Split a bucket
pub const SPLIT_BUCKET: u32 = 0x31;
/// Get bucket resource amount
pub const GET_AMOUNT: u32 = 0x32;
/// Get bucket resource address
pub const GET_RESOURCE: u32 = 0x33;
/// Obtain an immutable reference to a bucket
pub const BORROW_IMMUTABLE: u32 = 0x34;
/// (TODO) Obtain a mutable reference to a bucket
pub const BORROW_MUTABLE: u32 = 0x35;
/// Drop a reference
pub const RETURN_REFERENCE: u32 = 0x36;
/// Get the resource amount behind a reference
pub const GET_AMOUNT_REF: u32 = 0x37;
/// Get the resource address behind a reference
pub const GET_RESOURCE_REF: u32 = 0x38;

/// Withdraw from an account
pub const WITHDRAW: u32 = 0x40;
/// Deposit into an account
pub const DEPOSIT: u32 = 0x41;

/// Log a message
pub const EMIT_LOG: u32 = 0x50;
/// Retrieve context address
pub const GET_CONTEXT_ADDRESS: u32 = 0x51;
/// Retrieve the call data
pub const GET_CALL_DATA: u32 = 0x52;

#[derive(Debug, Clone, Describe, Encode, Decode)]
pub struct ComponentInfo {
    pub blueprint: Address,
    pub name: String,
}

#[derive(Debug, Clone, Describe, Encode, Decode)]
pub struct ResourceInfo {
    pub symbol: String,
    pub name: String,
    pub description: String,
    pub url: String,
    pub icon_url: String,
    pub minter: Option<Address>,
    pub supply: Option<U256>,
}

/// Represents a logging level.
#[derive(Debug, Clone, Describe, Encode, Decode)]
pub enum Level {
    Error = 0,
    Warn,
    Info,
    Debug,
    Trace,
}

//==========
// blueprint
//==========

#[derive(Debug, Clone, Encode, Decode)]
pub struct PublishBlueprintInput {
    pub code: Vec<u8>,
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct PublishBlueprintOutput {
    pub blueprint: Address,
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct CallBlueprintInput {
    pub blueprint: Address,
    pub component: String,
    pub method: String,
    pub args: Vec<Vec<u8>>,
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct CallBlueprintOutput {
    pub rtn: Vec<u8>,
}

//==========
// component
//==========

#[derive(Debug, Clone, Encode, Decode)]
pub struct CreateComponentInput {
    pub name: String,
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
    pub result: Option<ComponentInfo>,
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
pub struct CreateResourceInput {
    pub info: ResourceInfo,
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct CreateResourceOutput {
    pub resource: Address,
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct GetResourceInfoInput {
    pub resource: Address,
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct GetResourceInfoOutput {
    pub result: Option<ResourceInfo>,
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
    pub reference: Reference,
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct BorrowMutableInput {
    pub bucket: BID,
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct BorrowMutableOutput {
    pub reference: Reference,
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct ReturnReferenceInput {
    pub reference: Reference,
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct ReturnReferenceOutput {}

#[derive(Debug, Clone, Encode, Decode)]
pub struct GetAmountRefInput {
    pub reference: Reference,
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct GetAmountRefOutput {
    pub amount: U256,
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct GetResourceRefInput {
    pub reference: Reference,
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct GetResourceRefOutput {
    pub resource: Address,
}

//========
// account
//========

#[derive(Debug, Clone, Encode, Decode)]
pub struct WithdrawInput {
    pub account: Address,
    pub amount: U256,
    pub resource: Address,
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct WithdrawOutput {
    pub bucket: BID,
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct DepositInput {
    pub account: Address,
    pub bucket: BID,
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct DepositOutput {}

//=======
// others
//=======

#[derive(Debug, Clone, Encode, Decode)]
pub struct EmitLogInput {
    pub level: Level,
    pub message: String,
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct EmitLogOutput {}

#[derive(Debug, Clone, Encode, Decode)]
pub struct GetContextAddressInput {}

#[derive(Debug, Clone, Encode, Decode)]
pub struct GetContextAddressOutput {
    pub address: Address,
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct GetCallDataInput {}

#[derive(Debug, Clone, Encode, Decode)]
pub struct GetCallDataOutput {
    pub method: String,
    pub args: Vec<Vec<u8>>,
}
