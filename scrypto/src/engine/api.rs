use crate::core::SNodeRef;
use sbor::*;
use scrypto::prelude::Authorization;

use crate::engine::types::*;
use crate::rust::collections::BTreeSet;
use crate::rust::string::String;
use crate::rust::vec::Vec;

#[cfg(target_arch = "wasm32")]
extern "C" {
    /// Entrance to Radix Engine.
    pub fn radix_engine(op: u32, input_ptr: *const u8, input_len: usize) -> *mut u8;
}

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

/// Create an empty vault
pub const CREATE_EMPTY_VAULT: u32 = 0x40;
/// Get the IDs of all non-fungibles in this vault
pub const GET_NON_FUNGIBLE_IDS_IN_VAULT: u32 = 0x46;

/// Create a vault proof
pub const CREATE_VAULT_PROOF: u32 = 0x63;
/// Create a vault proof by amount
pub const CREATE_VAULT_PROOF_BY_AMOUNT: u32 = 0x64;
/// Create a vault proof by ids
pub const CREATE_VAULT_PROOF_BY_IDS: u32 = 0x65;

pub const INVOKE_SNODE: u32 = 0x70;

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

#[derive(Debug, TypeId, Encode, Decode)]
pub struct InvokeSNodeInput {
    pub snode_ref: SNodeRef,
    pub function: String,
    pub args: Vec<Vec<u8>>,
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct InvokeSNodeOutput {
    pub rtn: Vec<u8>,
}

//==========
// component
//==========

#[derive(Debug, TypeId, Encode, Decode)]
pub struct CreateComponentInput {
    pub blueprint_name: String,
    pub state: Vec<u8>,
    pub authorization: Vec<Authorization>,
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct CreateComponentOutput {
    pub component_address: ComponentAddress,
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct GetComponentInfoInput {
    pub component_address: ComponentAddress,
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct GetComponentInfoOutput {
    pub package_address: PackageAddress,
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

//==========
// vault
//==========

#[derive(Debug, TypeId, Encode, Decode)]
pub struct CreateEmptyVaultInput {
    pub resource_address: ResourceAddress,
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct CreateEmptyVaultOutput {
    pub vault_id: VaultId,
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct GetNonFungibleIdsInVaultInput {
    pub vault_id: VaultId,
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct GetNonFungibleIdsInVaultOutput {
    pub non_fungible_ids: BTreeSet<NonFungibleId>,
}

//==========
// proof
//==========

#[derive(Debug, TypeId, Encode, Decode)]
pub struct CreateVaultProofInput {
    pub vault_id: VaultId,
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct CreateVaultProofOutput {
    pub proof_id: ProofId,
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct CreateVaultProofByAmountInput {
    pub vault_id: VaultId,
    pub amount: Decimal,
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct CreateVaultProofByAmountOutput {
    pub proof_id: ProofId,
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct CreateVaultProofByIdsInput {
    pub vault_id: VaultId,
    pub ids: BTreeSet<NonFungibleId>,
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct CreateVaultProofByIdsOutput {
    pub proof_id: ProofId,
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
    pub actor: ScryptoActorInfo,
}
