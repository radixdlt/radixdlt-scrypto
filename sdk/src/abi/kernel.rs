extern crate alloc;
use alloc::string::String;
use alloc::vec::Vec;

use serde::{Deserialize, Serialize};

use crate::types::*;

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
/// Mint badges
pub const MINT_BADGES: u32 = 0x25;
/// Combine badges
pub const COMBINE_BADGES: u32 = 0x26;
/// Split Badges
pub const SPLIT_BADGES: u32 = 0x27;
/// Obtain an immutable reference to badges
pub const BORROW_BADGES: u32 = 0x28;
/// Return an immutable reference to badges
pub const RETURN_BADGES: u32 = 0x29;
/// Get token amount
pub const GET_TOKENS_AMOUNT: u32 = 0x2A;
/// Get token resource address
pub const GET_TOKENS_RESOURCE: u32 = 0x2B;
/// Get badge amount
pub const GET_BADGES_AMOUNT: u32 = 0x2C;
/// Get badge resource address
pub const GET_BADGES_RESOURCE: u32 = 0x2D;

/// Withdraw tokens from an account
pub const WITHDRAW_TOKENS: u32 = 0x30;
/// Deposit tokens into an account
pub const DEPOSIT_TOKENS: u32 = 0x31;
/// Withdraw badges from an account
pub const WITHDRAW_BADGES: u32 = 0x32;
/// Deposit badges into an account
pub const DEPOSIT_BADGES: u32 = 0x33;

/// Log a message
pub const EMIT_LOG: u32 = 0x40;
/// Retrieve context address
pub const GET_CONTEXT_ADDRESS: u32 = 0x41;

#[derive(Debug, Serialize, Deserialize)]
pub struct ComponentInfo {
    pub blueprint: Address,
    pub kind: String,
}

#[derive(Debug, Serialize, Deserialize)]
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

#[derive(Debug, Serialize, Deserialize)]
pub struct PublishBlueprintInput {
    pub code: Vec<u8>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PublishBlueprintOutput {
    pub blueprint: Address,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CallBlueprintInput {
    pub blueprint: Address,
    pub component: String,
    pub method: String,
    pub args: Vec<SerializedValue>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CallBlueprintOutput {
    pub rtn: SerializedValue,
}

//==========
// component
//==========

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateComponentInput {
    pub kind: String,
    pub state: SerializedValue,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateComponentOutput {
    pub component: Address,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetComponentInfoInput {
    pub component: Address,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetComponentInfoOutput {
    pub result: Option<ComponentInfo>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetComponentStateInput {
    pub component: Address,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetComponentStateOutput {
    pub state: SerializedValue,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PutComponentStateInput {
    pub component: Address,
    pub state: SerializedValue,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PutComponentStateOutput {}

//=========
// resource
//=========

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateResourceInput {
    pub info: ResourceInfo,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateResourceOutput {
    pub resource: Address,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetResourceInfoInput {
    pub resource: Address,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetResourceInfoOutput {
    pub result: Option<ResourceInfo>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MintTokensInput {
    pub amount: U256,
    pub resource: Address,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MintTokensOutput {
    pub tokens: RID,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CombineTokensInput {
    pub tokens: RID,
    pub other: RID,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CombineTokensOutput {}

#[derive(Debug, Serialize, Deserialize)]
pub struct SplitTokensInput {
    pub tokens: RID,
    pub amount: U256,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SplitTokensOutput {
    pub tokens: RID,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MintBadgesInput {
    pub amount: U256,
    pub resource: Address,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MintBadgesOutput {
    pub badges: RID,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CombineBadgesInput {
    pub badges: RID,
    pub other: RID,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CombineBadgesOutput {}

#[derive(Debug, Serialize, Deserialize)]
pub struct SplitBadgesInput {
    pub badges: RID,
    pub amount: U256,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SplitBadgesOutput {
    pub badges: RID,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BorrowBadgesInput {
    pub badges: RID,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BorrowBadgesOutput {
    pub reference: RID,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ReturnBadgesInput {
    pub reference: RID,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ReturnBadgesOutput {}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetTokensAmountInput {
    pub tokens: RID,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetTokensAmountOutput {
    pub amount: U256,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetTokensResourceInput {
    pub tokens: RID,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetTokensResourceOutput {
    pub resource: Address,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetBadgesAmountInput {
    pub badges: RID,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetBadgesAmountOutput {
    pub amount: U256,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetBadgesResourceInput {
    pub badges: RID,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetBadgesResourceOutput {
    pub resource: Address,
}

//========
// account
//========

#[derive(Debug, Serialize, Deserialize)]
pub struct WithdrawTokensInput {
    pub account: Address,
    pub amount: U256,
    pub resource: Address,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WithdrawTokensOutput {
    pub tokens: RID,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DepositTokensInput {
    pub account: Address,
    pub tokens: RID,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DepositTokensOutput {}

#[derive(Debug, Serialize, Deserialize)]
pub struct WithdrawBadgesInput {
    pub account: Address,
    pub amount: U256,
    pub resource: Address,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WithdrawBadgesOutput {
    pub badges: RID,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DepositBadgesInput {
    pub account: Address,
    pub badges: RID,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DepositBadgesOutput {}

//=======
// others
//=======

#[derive(Debug, Serialize, Deserialize)]
pub struct EmitLogInput {
    pub level: String,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EmitLogOutput {}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetContextAddressInput {}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetContextAddressOutput {
    pub address: Address,
}
