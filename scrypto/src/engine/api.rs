use crate::core::SNodeRef;
use sbor::*;
use scrypto::prelude::ComponentAuthorization;

use crate::engine::types::*;
use crate::rust::collections::BTreeSet;
use crate::rust::string::String;
use crate::rust::vec::Vec;

#[cfg(target_arch = "wasm32")]
extern "C" {
    /// Entrance to Radix Engine.
    pub fn radix_engine(op: u32, input_ptr: *const u8, input_len: usize) -> *mut u8;
}

/// Publish a code package
pub const PUBLISH_PACKAGE: u32 = 0x00;

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
/// Put fungible resource into this vault
pub const PUT_INTO_VAULT: u32 = 0x41;
/// Get vault resource amount
pub const GET_VAULT_AMOUNT: u32 = 0x43;
/// Get vault resource address
pub const GET_VAULT_RESOURCE_ADDRESS: u32 = 0x44;
/// Get the IDs of all non-fungibles in this vault
pub const GET_NON_FUNGIBLE_IDS_IN_VAULT: u32 = 0x46;

/// Create an empty bucket
pub const CREATE_EMPTY_BUCKET: u32 = 0x50;
/// Get the IDs of all non-fungibles in this bucket
pub const GET_NON_FUNGIBLE_IDS_IN_BUCKET: u32 = 0x56;

/// Create a bucket proof
pub const CREATE_BUCKET_PROOF: u32 = 0x60;
/// Create a vault proof
pub const CREATE_VAULT_PROOF: u32 = 0x63;
/// Create a vault proof by amount
pub const CREATE_VAULT_PROOF_BY_AMOUNT: u32 = 0x64;
/// Create a vault proof by ids
pub const CREATE_VAULT_PROOF_BY_IDS: u32 = 0x65;
/// Create an auth zone proof
pub const CREATE_AUTH_ZONE_PROOF: u32 = 0x66;
/// Create an auth zone proof by amount
pub const CREATE_AUTH_ZONE_PROOF_BY_AMOUNT: u32 = 0x67;
/// Create an auth zone proof by ids
pub const CREATE_AUTH_ZONE_PROOF_BY_IDS: u32 = 0x68;
/// Clone proof
pub const CLONE_PROOF: u32 = 0x69;
/// Drop a proof
pub const DROP_PROOF: u32 = 0x6A;
/// Get the resource amount
pub const GET_PROOF_AMOUNT: u32 = 0x6B;
/// Get the resource address
pub const GET_PROOF_RESOURCE_ADDRESS: u32 = 0x6C;
/// Get the non-fungible ids
pub const GET_NON_FUNGIBLE_IDS_IN_PROOF: u32 = 0x6D;
/// Push a proof onto auth zone
pub const PUSH_TO_AUTH_ZONE: u32 = 0x6E;
/// Pop a proof from auth zone
pub const POP_FROM_AUTH_ZONE: u32 = 0x6F;

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
// blueprint
//==========

#[derive(Debug, TypeId, Encode, Decode)]
pub struct PublishPackageInput {
    pub code: Vec<u8>,
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct PublishPackageOutput {
    pub package_address: PackageAddress,
}

//==========
// component
//==========

#[derive(Debug, TypeId, Encode, Decode)]
pub struct CreateComponentInput {
    pub blueprint_name: String,
    pub state: Vec<u8>,
    pub authorization: ComponentAuthorization,
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
pub struct PutIntoVaultInput {
    pub vault_id: VaultId,
    pub bucket_id: BucketId,
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct PutIntoVaultOutput {}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct GetVaultAmountInput {
    pub vault_id: VaultId,
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct GetVaultAmountOutput {
    pub amount: Decimal,
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct GetVaultResourceAddressInput {
    pub vault_id: VaultId,
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct GetVaultResourceAddressOutput {
    pub resource_address: ResourceAddress,
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
// bucket
//==========

#[derive(Debug, TypeId, Encode, Decode)]
pub struct CreateEmptyBucketInput {
    pub resource_address: ResourceAddress,
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct CreateEmptyBucketOutput {
    pub bucket_id: BucketId,
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct GetNonFungibleIdsInBucketInput {
    pub bucket_id: BucketId,
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct GetNonFungibleIdsInBucketOutput {
    pub non_fungible_ids: BTreeSet<NonFungibleId>,
}

//==========
// proof
//==========

#[derive(Debug, TypeId, Encode, Decode)]
pub struct CreateBucketProofInput {
    pub bucket_id: BucketId,
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct CreateBucketProofOutput {
    pub proof_id: ProofId,
}

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

#[derive(Debug, TypeId, Encode, Decode)]
pub struct CreateAuthZoneProofInput {
    pub resource_address: ResourceAddress,
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct CreateAuthZoneProofOutput {
    pub proof_id: ProofId,
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct CreateAuthZoneProofByAmountInput {
    pub resource_address: ResourceAddress,
    pub amount: Decimal,
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct CreateAuthZoneProofByAmountOutput {
    pub proof_id: ProofId,
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct CreateAuthZoneProofByIdsInput {
    pub resource_address: ResourceAddress,
    pub ids: BTreeSet<NonFungibleId>,
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct CreateAuthZoneProofByIdsOutput {
    pub proof_id: ProofId,
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct DropProofInput {
    pub proof_id: ProofId,
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct DropProofOutput {}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct GetProofAmountInput {
    pub proof_id: ProofId,
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct GetProofAmountOutput {
    pub amount: Decimal,
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct GetProofResourceAddressInput {
    pub proof_id: ProofId,
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct GetProofResourceAddressOutput {
    pub resource_address: ResourceAddress,
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct GetNonFungibleIdsInProofInput {
    pub proof_id: ProofId,
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct GetNonFungibleIdsInProofOutput {
    pub non_fungible_ids: BTreeSet<NonFungibleId>,
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct CloneProofInput {
    pub proof_id: ProofId,
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct CloneProofOutput {
    pub proof_id: ProofId,
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct PushToAuthZoneInput {
    pub proof_id: ProofId,
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct PushToAuthZoneOutput {}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct PopFromAuthZoneInput {}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct PopFromAuthZoneOutput {
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
