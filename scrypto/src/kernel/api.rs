extern crate alloc;
use alloc::string::String;
use alloc::vec::Vec;

use sbor::{Decode, Encode};

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
/// Mint tokens
pub const MINT_TOKENS: u32 = 0x22;
/// Combine tokens
pub const COMBINE_TOKENS: u32 = 0x23;
/// Split tokens
pub const SPLIT_TOKENS: u32 = 0x24;
/// TODO: Obtain an immutable reference to tokens
pub const BORROW_TOKENS: u32 = 0x25;
/// TODO: Return a reference to tokens
pub const RETURN_TOKENS: u32 = 0x26;
/// Mint badges
pub const MINT_BADGES: u32 = 0x27;
/// Combine badges
pub const COMBINE_BADGES: u32 = 0x28;
/// Split Badges
pub const SPLIT_BADGES: u32 = 0x29;
/// Obtain an immutable reference to badges
pub const BORROW_BADGES: u32 = 0x2a;
/// Return an reference to badges
pub const RETURN_BADGES: u32 = 0x2b;
/// Get token amount
pub const GET_TOKENS_AMOUNT: u32 = 0x2c;
/// Get token resource address
pub const GET_TOKENS_RESOURCE: u32 = 0x2d;
/// Get badge amount
pub const GET_BADGES_AMOUNT: u32 = 0x2e;
/// Get badge resource address
pub const GET_BADGES_RESOURCE: u32 = 0x2f;

/// Withdraw tokens from an account
pub const WITHDRAW_TOKENS: u32 = 0x40;
/// Deposit tokens into an account
pub const DEPOSIT_TOKENS: u32 = 0x41;
/// Withdraw badges from an account
pub const WITHDRAW_BADGES: u32 = 0x42;
/// Deposit badges into an account
pub const DEPOSIT_BADGES: u32 = 0x43;

/// Log a message
pub const EMIT_LOG: u32 = 0x50;
/// Retrieve context address
pub const GET_CONTEXT_ADDRESS: u32 = 0x51;

#[derive(Debug, Encode, Decode)]
pub struct ComponentInfo {
    pub blueprint: Address,
    pub kind: String,
}

#[derive(Debug, Encode, Decode)]
pub struct ResourceInfo {
    pub symbol: String,
    pub name: String,
    pub description: String,
    pub url: String,
    pub icon_url: String,
    pub minter: Option<Address>,
    pub supply: Option<U256>,
}

//==========
// blueprint
//==========

#[derive(Debug, Encode, Decode)]
pub struct PublishBlueprintInput {
    pub code: Vec<u8>,
}

#[derive(Debug, Encode, Decode)]
pub struct PublishBlueprintOutput {
    pub blueprint: Address,
}

#[derive(Debug, Encode, Decode)]
pub struct CallBlueprintInput {
    pub blueprint: Address,
    pub component: String,
    pub method: String,
    pub args: Vec<Vec<u8>>,
}

#[derive(Debug, Encode, Decode)]
pub struct CallBlueprintOutput {
    pub rtn: Vec<u8>,
}

//==========
// component
//==========

#[derive(Debug, Encode, Decode)]
pub struct CreateComponentInput {
    pub kind: String,
    pub state: Vec<u8>,
}

#[derive(Debug, Encode, Decode)]
pub struct CreateComponentOutput {
    pub component: Address,
}

#[derive(Debug, Encode, Decode)]
pub struct GetComponentInfoInput {
    pub component: Address,
}

#[derive(Debug, Encode, Decode)]
pub struct GetComponentInfoOutput {
    pub result: Option<ComponentInfo>,
}

#[derive(Debug, Encode, Decode)]
pub struct GetComponentStateInput {
    pub component: Address,
}

#[derive(Debug, Encode, Decode)]
pub struct GetComponentStateOutput {
    pub state: Vec<u8>,
}

#[derive(Debug, Encode, Decode)]
pub struct PutComponentStateInput {
    pub component: Address,
    pub state: Vec<u8>,
}

#[derive(Debug, Encode, Decode)]
pub struct PutComponentStateOutput {}

#[derive(Debug, Encode, Decode)]
pub struct GetComponentStorageInput {
    pub component: Address,
    pub key: Vec<u8>,
}

#[derive(Debug, Encode, Decode)]
pub struct GetComponentStorageOutput {
    pub value: Option<Vec<u8>>,
}

#[derive(Debug, Encode, Decode)]
pub struct PutComponentStorageInput {
    pub component: Address,
    pub key: Vec<u8>,
    pub value: Vec<u8>,
}

#[derive(Debug, Encode, Decode)]
pub struct PutComponentStorageOutput {}

//=========
// resource
//=========

#[derive(Debug, Encode, Decode)]
pub struct CreateResourceInput {
    pub info: ResourceInfo,
}

#[derive(Debug, Encode, Decode)]
pub struct CreateResourceOutput {
    pub resource: Address,
}

#[derive(Debug, Encode, Decode)]
pub struct GetResourceInfoInput {
    pub resource: Address,
}

#[derive(Debug, Encode, Decode)]
pub struct GetResourceInfoOutput {
    pub result: Option<ResourceInfo>,
}

#[derive(Debug, Encode, Decode)]
pub struct MintTokensInput {
    pub amount: U256,
    pub resource: Address,
}

#[derive(Debug, Encode, Decode)]
pub struct MintTokensOutput {
    pub tokens: RID,
}

#[derive(Debug, Encode, Decode)]
pub struct CombineTokensInput {
    pub tokens: RID,
    pub other: RID,
}

#[derive(Debug, Encode, Decode)]
pub struct CombineTokensOutput {}

#[derive(Debug, Encode, Decode)]
pub struct SplitTokensInput {
    pub tokens: RID,
    pub amount: U256,
}

#[derive(Debug, Encode, Decode)]
pub struct SplitTokensOutput {
    pub tokens: RID,
}

#[derive(Debug, Encode, Decode)]
pub struct BorrowTokensInput {
    pub tokens: RID,
}

#[derive(Debug, Encode, Decode)]
pub struct BorrowTokensOutput {
    pub reference: RID,
}

#[derive(Debug, Encode, Decode)]
pub struct ReturnTokensInput {
    pub reference: RID,
}

#[derive(Debug, Encode, Decode)]
pub struct ReturnTokensOutput {}

#[derive(Debug, Encode, Decode)]
pub struct MintBadgesInput {
    pub amount: U256,
    pub resource: Address,
}

#[derive(Debug, Encode, Decode)]
pub struct MintBadgesOutput {
    pub badges: RID,
}

#[derive(Debug, Encode, Decode)]
pub struct CombineBadgesInput {
    pub badges: RID,
    pub other: RID,
}

#[derive(Debug, Encode, Decode)]
pub struct CombineBadgesOutput {}

#[derive(Debug, Encode, Decode)]
pub struct SplitBadgesInput {
    pub badges: RID,
    pub amount: U256,
}

#[derive(Debug, Encode, Decode)]
pub struct SplitBadgesOutput {
    pub badges: RID,
}

#[derive(Debug, Encode, Decode)]
pub struct BorrowBadgesInput {
    pub badges: RID,
}

#[derive(Debug, Encode, Decode)]
pub struct BorrowBadgesOutput {
    pub reference: RID,
}

#[derive(Debug, Encode, Decode)]
pub struct ReturnBadgesInput {
    pub reference: RID,
}

#[derive(Debug, Encode, Decode)]
pub struct ReturnBadgesOutput {}

#[derive(Debug, Encode, Decode)]
pub struct GetTokensAmountInput {
    pub tokens: RID,
}

#[derive(Debug, Encode, Decode)]
pub struct GetTokensAmountOutput {
    pub amount: U256,
}

#[derive(Debug, Encode, Decode)]
pub struct GetTokensResourceInput {
    pub tokens: RID,
}

#[derive(Debug, Encode, Decode)]
pub struct GetTokensResourceOutput {
    pub resource: Address,
}

#[derive(Debug, Encode, Decode)]
pub struct GetBadgesAmountInput {
    pub badges: RID,
}

#[derive(Debug, Encode, Decode)]
pub struct GetBadgesAmountOutput {
    pub amount: U256,
}

#[derive(Debug, Encode, Decode)]
pub struct GetBadgesResourceInput {
    pub badges: RID,
}

#[derive(Debug, Encode, Decode)]
pub struct GetBadgesResourceOutput {
    pub resource: Address,
}

//========
// account
//========

#[derive(Debug, Encode, Decode)]
pub struct WithdrawTokensInput {
    pub account: Address,
    pub amount: U256,
    pub resource: Address,
}

#[derive(Debug, Encode, Decode)]
pub struct WithdrawTokensOutput {
    pub tokens: RID,
}

#[derive(Debug, Encode, Decode)]
pub struct DepositTokensInput {
    pub account: Address,
    pub tokens: RID,
}

#[derive(Debug, Encode, Decode)]
pub struct DepositTokensOutput {}

#[derive(Debug, Encode, Decode)]
pub struct WithdrawBadgesInput {
    pub account: Address,
    pub amount: U256,
    pub resource: Address,
}

#[derive(Debug, Encode, Decode)]
pub struct WithdrawBadgesOutput {
    pub badges: RID,
}

#[derive(Debug, Encode, Decode)]
pub struct DepositBadgesInput {
    pub account: Address,
    pub badges: RID,
}

#[derive(Debug, Encode, Decode)]
pub struct DepositBadgesOutput {}

//=======
// others
//=======

#[derive(Debug, Encode, Decode)]
pub struct EmitLogInput {
    pub level: String,
    pub message: String,
}

#[derive(Debug, Encode, Decode)]
pub struct EmitLogOutput {}

#[derive(Debug, Encode, Decode)]
pub struct GetContextAddressInput {}

#[derive(Debug, Encode, Decode)]
pub struct GetContextAddressOutput {
    pub address: Address,
}
