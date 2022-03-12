use sbor::*;

use crate::engine::types::*;
use crate::prelude::NonFungibleAddress;
use crate::rust::collections::HashMap;
use crate::rust::string::String;
use crate::rust::vec::Vec;

#[cfg(target_arch = "wasm32")]
extern "C" {
    /// Entrance to Radix Engine.
    pub fn radix_engine(op: u32, input_ptr: *const u8, input_len: usize) -> *mut u8;
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
/// Get the data of a non-fungible
pub const GET_NON_FUNGIBLE_DATA: u32 = 0x3a;
/// Update the data of a non-fungible
pub const UPDATE_NON_FUNGIBLE_MUTABLE_DATA: u32 = 0x3b;
/// Update resource metadata
pub const UPDATE_RESOURCE_METADATA: u32 = 0x3c;
/// Check if non-fungible resource with id exists
pub const NON_FUNGIBLE_EXISTS: u32 = 0x3d;

/// Create an empty vault
pub const CREATE_EMPTY_VAULT: u32 = 0x40;
/// Put fungible resource into this vault
pub const PUT_INTO_VAULT: u32 = 0x41;
/// Take fungible resource from this vault
pub const TAKE_FROM_VAULT: u32 = 0x42;
/// Get vault resource amount
pub const GET_VAULT_AMOUNT: u32 = 0x43;
/// Get vault resource definition
pub const GET_VAULT_RESOURCE_DEF_ID: u32 = 0x44;
/// Take a non-fungible from this vault, by id
pub const TAKE_NON_FUNGIBLE_FROM_VAULT: u32 = 0x45;
/// Get the IDs of all non-fungibles in this vault
pub const GET_NON_FUNGIBLE_IDS_IN_VAULT: u32 = 0x46;

/// Create an empty bucket
pub const CREATE_EMPTY_BUCKET: u32 = 0x50;
/// Put fungible resource into this bucket
pub const PUT_INTO_BUCKET: u32 = 0x51;
/// Take fungible resource from this bucket
pub const TAKE_FROM_BUCKET: u32 = 0x52;
/// Get bucket resource amount
pub const GET_BUCKET_AMOUNT: u32 = 0x53;
/// Get bucket resource definition
pub const GET_BUCKET_RESOURCE_DEF_ID: u32 = 0x54;
/// Take a non-fungible from this bucket, by id
pub const TAKE_NON_FUNGIBLE_FROM_BUCKET: u32 = 0x55;
/// Get the IDs of all non-fungibles in this bucket
pub const GET_NON_FUNGIBLE_IDS_IN_BUCKET: u32 = 0x56;

/// Create a bucket proof
pub const CREATE_BUCKET_PROOF: u32 = 0x60;
/// Create a vault proof
pub const CREATE_VAULT_PROOF: u32 = 0x61;
/// Clone proof
pub const CLONE_PROOF: u32 = 0x62;
/// Drop a proof
pub const DROP_PROOF: u32 = 0x63;
/// Get the resource amount behind a proof
pub const GET_PROOF_AMOUNT: u32 = 0x64;
/// Get the resource definition behind a proof
pub const GET_PROOF_RESOURCE_DEF_ID: u32 = 0x65;
/// Get the non-fungible ids in the proof
pub const GET_NON_FUNGIBLE_IDS_IN_PROOF: u32 = 0x66;

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

//==========
// blueprint
//==========

#[derive(Debug, TypeId, Encode, Decode)]
pub struct PublishPackageInput {
    pub code: Vec<u8>,
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct PublishPackageOutput {
    pub package_id: PackageId,
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct CallFunctionInput {
    pub package_id: PackageId,
    pub blueprint_name: String,
    pub function: String,
    pub args: Vec<Vec<u8>>,
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct CallFunctionOutput {
    pub rtn: Vec<u8>,
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct CallMethodInput {
    pub component_id: ComponentId,
    pub method: String,
    pub args: Vec<Vec<u8>>,
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct CallMethodOutput {
    pub rtn: Vec<u8>,
}

//==========
// component
//==========

#[derive(Debug, TypeId, Encode, Decode)]
pub struct CreateComponentInput {
    pub package_id: PackageId,
    pub blueprint_name: String,
    pub state: Vec<u8>,
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct CreateComponentOutput {
    pub component_id: ComponentId,
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct GetComponentInfoInput {
    pub component_id: ComponentId,
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct GetComponentInfoOutput {
    pub package_id: PackageId,
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

//=========
// resource
//=========

#[derive(Debug, TypeId, Encode, Decode)]
pub struct CreateResourceInput {
    pub resource_type: ResourceType,
    pub metadata: HashMap<String, String>,
    pub flags: u64,
    pub mutable_flags: u64,
    pub authorities: HashMap<ResourceDefId, u64>,
    pub initial_supply: Option<Supply>,
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct CreateResourceOutput {
    pub resource_def_id: ResourceDefId,
    pub bucket_id: Option<BucketId>,
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct MintResourceInput {
    pub resource_def_id: ResourceDefId,
    pub new_supply: Supply,
    pub auth: ProofId,
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct MintResourceOutput {
    pub bucket_id: BucketId,
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct BurnResourceInput {
    pub bucket_id: BucketId,
    pub auth: Option<ProofId>,
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct BurnResourceOutput {}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct GetResourceMetadataInput {
    pub resource_def_id: ResourceDefId,
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct GetResourceMetadataOutput {
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct GetResourceTypeInput {
    pub resource_def_id: ResourceDefId,
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct GetResourceTypeOutput {
    pub resource_type: ResourceType,
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct GetResourceTotalSupplyInput {
    pub resource_def_id: ResourceDefId,
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct GetResourceTotalSupplyOutput {
    pub total_supply: Decimal,
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct GetNonFungibleDataInput {
    pub non_fungible_address: NonFungibleAddress,
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct GetNonFungibleDataOutput {
    pub immutable_data: Vec<u8>,
    pub mutable_data: Vec<u8>,
}

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct NonFungibleExistsInput {
    pub non_fungible_address: NonFungibleAddress,
}

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct NonFungibleExistsOutput {
    pub non_fungible_exists: bool,
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct UpdateNonFungibleMutableDataInput {
    pub non_fungible_address: NonFungibleAddress,
    pub new_mutable_data: Vec<u8>,
    pub auth: ProofId,
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct UpdateNonFungibleMutableDataOutput {}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct GetResourceFlagsInput {
    pub resource_def_id: ResourceDefId,
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct GetResourceFlagsOutput {
    pub flags: u64,
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct UpdateResourceFlagsInput {
    pub resource_def_id: ResourceDefId,
    pub new_flags: u64,
    pub auth: ProofId,
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct UpdateResourceFlagsOutput {}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct GetResourceMutableFlagsInput {
    pub resource_def_id: ResourceDefId,
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct GetResourceMutableFlagsOutput {
    pub mutable_flags: u64,
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct UpdateResourceMutableFlagsInput {
    pub resource_def_id: ResourceDefId,
    pub new_mutable_flags: u64,
    pub auth: ProofId,
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct UpdateResourceMutableFlagsOutput {}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct UpdateResourceMetadataInput {
    pub resource_def_id: ResourceDefId,
    pub new_metadata: HashMap<String, String>,
    pub auth: ProofId,
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct UpdateResourceMetadataOutput {}

//==========
// vault
//==========

#[derive(Debug, TypeId, Encode, Decode)]
pub struct CreateEmptyVaultInput {
    pub resource_def_id: ResourceDefId,
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
pub struct TakeFromVaultInput {
    pub vault_id: VaultId,
    pub amount: Decimal,
    pub auth: Option<ProofId>,
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct TakeFromVaultOutput {
    pub bucket_id: BucketId,
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct GetVaultAmountInput {
    pub vault_id: VaultId,
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct GetVaultAmountOutput {
    pub amount: Decimal,
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct GetVaultResourceDefIdInput {
    pub vault_id: VaultId,
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct GetVaultResourceDefIdOutput {
    pub resource_def_id: ResourceDefId,
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct TakeNonFungibleFromVaultInput {
    pub vault_id: VaultId,
    pub non_fungible_id: NonFungibleId,
    pub auth: Option<ProofId>,
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct TakeNonFungibleFromVaultOutput {
    pub bucket_id: BucketId,
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct GetNonFungibleIdsInVaultInput {
    pub vault_id: VaultId,
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct GetNonFungibleIdsInVaultOutput {
    pub non_fungible_ids: Vec<NonFungibleId>,
}

//==========
// bucket
//==========

#[derive(Debug, TypeId, Encode, Decode)]
pub struct CreateEmptyBucketInput {
    pub resource_def_id: ResourceDefId,
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct CreateEmptyBucketOutput {
    pub bucket_id: BucketId,
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct PutIntoBucketInput {
    pub bucket_id: BucketId,
    pub other: BucketId,
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct PutIntoBucketOutput {}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct TakeFromBucketInput {
    pub bucket_id: BucketId,
    pub amount: Decimal,
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct TakeFromBucketOutput {
    pub bucket_id: BucketId,
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct GetBucketAmountInput {
    pub bucket_id: BucketId,
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct GetBucketAmountOutput {
    pub amount: Decimal,
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct GetBucketResourceDefIdInput {
    pub bucket_id: BucketId,
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct GetBucketResourceDefIdOutput {
    pub resource_def_id: ResourceDefId,
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct TakeNonFungibleFromBucketInput {
    pub bucket_id: BucketId,
    pub non_fungible_id: NonFungibleId,
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct TakeNonFungibleFromBucketOutput {
    pub bucket_id: BucketId,
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct GetNonFungibleIdsInBucketInput {
    pub bucket_id: BucketId,
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct GetNonFungibleIdsInBucketOutput {
    pub non_fungible_ids: Vec<NonFungibleId>,
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
pub struct GetProofResourceDefIdInput {
    pub proof_id: ProofId,
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct GetProofResourceDefIdOutput {
    pub resource_def_id: ResourceDefId,
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct GetNonFungibleIdsInProofInput {
    pub proof_id: ProofId,
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct GetNonFungibleIdsInProofOutput {
    pub non_fungible_ids: Vec<NonFungibleId>,
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct CloneProofInput {
    pub proof_id: ProofId,
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct CloneProofOutput {
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
    pub actor: Actor,
}
