use crate::types::*;
use radix_engine_interface::api::types::*;
use radix_engine_interface::blueprints::access_controller::*;
use radix_engine_interface::blueprints::account::*;
use radix_engine_interface::blueprints::clock::*;
use radix_engine_interface::blueprints::epoch_manager::*;
use radix_engine_interface::blueprints::identity::*;
use radix_engine_interface::blueprints::logger::*;
use radix_engine_interface::blueprints::resource::*;

pub enum CostingEntry {
    /* invoke */
    Invoke { input_size: u32 },

    /* node */
    CreateNode { size: u32 },
    DropNode { size: u32 },

    /* substate */
    LockSubstate,
    ReadSubstate { size: u32 },
    WriteSubstate { size: u32 },
    DropLock,
    // TODO: more costing after API becomes stable.
}

#[derive(Debug, Clone, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct FeeTable {
    tx_base_fee: u32,
    tx_payload_cost_per_byte: u32,
    tx_signature_verification_per_sig: u32,
    tx_blob_price_per_byte: u32,
    fixed_low: u32,
    fixed_medium: u32,
    fixed_high: u32,
    wasm_instantiation_per_byte: u32,
}

impl FeeTable {
    pub fn new() -> Self {
        Self {
            tx_base_fee: 50_000,
            tx_payload_cost_per_byte: 5,
            tx_signature_verification_per_sig: 100_000,
            tx_blob_price_per_byte: 5,
            wasm_instantiation_per_byte: 1, // TODO: Re-enable WASM instantiation cost if it's unavoidable
            fixed_low: 500,
            fixed_medium: 2500,
            fixed_high: 5000,
        }
    }

    pub fn tx_base_fee(&self) -> u32 {
        self.tx_base_fee
    }

    pub fn tx_payload_cost_per_byte(&self) -> u32 {
        self.tx_payload_cost_per_byte
    }

    pub fn tx_signature_verification_per_sig(&self) -> u32 {
        self.tx_signature_verification_per_sig
    }

    pub fn tx_blob_price_per_byte(&self) -> u32 {
        self.tx_blob_price_per_byte
    }

    pub fn wasm_instantiation_per_byte(&self) -> u32 {
        self.wasm_instantiation_per_byte
    }

    pub fn run_cost(&self, identifier: &ScryptoFnIdentifier) -> u32 {
        match (
            identifier.package_address,
            identifier.blueprint_name.as_str(),
        ) {
            (LOGGER_PACKAGE, RESOURCE_MANAGER_BLUEPRINT) => match identifier.ident.as_str() {
                LOGGER_LOG_IDENT => self.fixed_low,
                _ => self.fixed_low,
            },
            (RESOURCE_MANAGER_PACKAGE, RESOURCE_MANAGER_BLUEPRINT) => {
                match identifier.ident.as_str() {
                    RESOURCE_MANAGER_CREATE_FUNGIBLE_IDENT => self.fixed_high,
                    RESOURCE_MANAGER_CREATE_FUNGIBLE_WITH_INITIAL_SUPPLY_IDENT => self.fixed_high,
                    RESOURCE_MANAGER_CREATE_FUNGIBLE_WITH_INITIAL_SUPPLY_AND_ADDRESS_IDENT => {
                        self.fixed_high
                    }
                    RESOURCE_MANAGER_CREATE_NON_FUNGIBLE_IDENT => self.fixed_high,
                    RESOURCE_MANAGER_CREATE_NON_FUNGIBLE_WITH_INITIAL_SUPPLY_IDENT => {
                        self.fixed_high
                    }
                    RESOURCE_MANAGER_CREATE_NON_FUNGIBLE_WITH_ADDRESS_IDENT => self.fixed_high,
                    RESOURCE_MANAGER_CREATE_UUID_NON_FUNGIBLE_WITH_INITIAL_SUPPLY => {
                        self.fixed_high
                    }
                    RESOURCE_MANAGER_MINT_NON_FUNGIBLE => self.fixed_high,
                    RESOURCE_MANAGER_MINT_UUID_NON_FUNGIBLE => self.fixed_high,
                    RESOURCE_MANAGER_MINT_FUNGIBLE => self.fixed_high,
                    RESOURCE_MANAGER_BURN_BUCKET_IDENT => self.fixed_medium,
                    RESOURCE_MANAGER_BURN_IDENT => self.fixed_medium,
                    RESOURCE_MANAGER_CREATE_VAULT_IDENT => self.fixed_medium,
                    RESOURCE_MANAGER_CREATE_BUCKET_IDENT => self.fixed_medium,
                    RESOURCE_MANAGER_UPDATE_VAULT_AUTH_IDENT => self.fixed_medium,
                    RESOURCE_MANAGER_SET_VAULT_AUTH_MUTABILITY_IDENT => self.fixed_medium,
                    RESOURCE_MANAGER_UPDATE_NON_FUNGIBLE_DATA_IDENT => self.fixed_medium,
                    RESOURCE_MANAGER_NON_FUNGIBLE_EXISTS_IDENT => self.fixed_medium,
                    RESOURCE_MANAGER_GET_TOTAL_SUPPLY_IDENT => self.fixed_low,
                    RESOURCE_MANAGER_GET_NON_FUNGIBLE_IDENT => self.fixed_low,
                    RESOURCE_MANAGER_GET_RESOURCE_TYPE_IDENT => self.fixed_low,
                    _ => self.fixed_low,
                }
            }
            (RESOURCE_MANAGER_PACKAGE, VAULT_BLUEPRINT) => match identifier.ident.as_str() {
                VAULT_TAKE_IDENT => self.fixed_medium,
                VAULT_TAKE_NON_FUNGIBLES_IDENT => self.fixed_medium,
                VAULT_LOCK_FEE_IDENT => self.fixed_medium,
                VAULT_RECALL_IDENT => self.fixed_medium,
                VAULT_RECALL_NON_FUNGIBLES_IDENT => self.fixed_medium,
                VAULT_PUT_IDENT => self.fixed_medium,
                VAULT_GET_AMOUNT_IDENT => self.fixed_low,
                VAULT_GET_RESOURCE_ADDRESS_IDENT => self.fixed_low,
                VAULT_GET_NON_FUNGIBLE_LOCAL_IDS_IDENT => self.fixed_low,
                VAULT_CREATE_PROOF_IDENT => self.fixed_low,
                VAULT_CREATE_PROOF_BY_AMOUNT_IDENT => self.fixed_low,
                VAULT_CREATE_PROOF_BY_IDS_IDENT => self.fixed_low,
                _ => self.fixed_low,
            },
            (RESOURCE_MANAGER_PACKAGE, BUCKET_BLUEPRINT) => match identifier.ident.as_str() {
                BUCKET_PUT_IDENT => self.fixed_low,
                BUCKET_TAKE_IDENT => self.fixed_low,
                BUCKET_TAKE_NON_FUNGIBLES_IDENT => self.fixed_low,
                BUCKET_GET_AMOUNT_IDENT => self.fixed_low,
                BUCKET_GET_NON_FUNGIBLE_LOCAL_IDS_IDENT => self.fixed_low,
                BUCKET_GET_RESOURCE_ADDRESS_IDENT => self.fixed_low,
                BUCKET_CREATE_PROOF_IDENT => self.fixed_low,
                _ => self.fixed_low,
            },
            (RESOURCE_MANAGER_PACKAGE, PROOF_BLUEPRINT) => match identifier.ident.as_str() {
                PROOF_CLONE_IDENT => self.fixed_low,
                PROOF_GET_AMOUNT_IDENT => self.fixed_low,
                PROOF_GET_RESOURCE_ADDRESS_IDENT => self.fixed_low,
                PROOF_GET_NON_FUNGIBLE_LOCAL_IDS_IDENT => self.fixed_low,
                _ => self.fixed_low,
            },
            (RESOURCE_MANAGER_PACKAGE, WORKTOP_BLUEPRINT) => match identifier.ident.as_str() {
                WORKTOP_PUT_IDENT => self.fixed_low,
                WORKTOP_TAKE_IDENT => self.fixed_low,
                WORKTOP_TAKE_NON_FUNGIBLES_IDENT => self.fixed_low,
                WORKTOP_TAKE_ALL_IDENT => self.fixed_low,
                WORKTOP_ASSERT_CONTAINS_IDENT => self.fixed_low,
                WORKTOP_ASSERT_CONTAINS_NON_FUNGIBLES_IDENT => self.fixed_low,
                WORKTOP_ASSERT_CONTAINS_AMOUNT_IDENT => self.fixed_low,
                WORKTOP_DRAIN_IDENT => self.fixed_low,
                _ => self.fixed_low,
            },
            (IDENTITY_PACKAGE, IDENTITY_BLUEPRINT) => match identifier.ident.as_str() {
                IDENTITY_CREATE_IDENT => self.fixed_low,
                _ => self.fixed_low,
            },
            (EPOCH_MANAGER_PACKAGE, EPOCH_MANAGER_BLUEPRINT) => match identifier.ident.as_str() {
                EPOCH_MANAGER_CREATE_IDENT => self.fixed_low,
                EPOCH_MANAGER_GET_CURRENT_EPOCH_IDENT => self.fixed_low,
                EPOCH_MANAGER_SET_EPOCH_IDENT => self.fixed_low,
                EPOCH_MANAGER_NEXT_ROUND_IDENT => self.fixed_low,
                EPOCH_MANAGER_CREATE_VALIDATOR_IDENT => self.fixed_low,
                EPOCH_MANAGER_UPDATE_VALIDATOR_IDENT => self.fixed_low,
                _ => self.fixed_low,
            },
            (EPOCH_MANAGER_PACKAGE, VALIDATOR_BLUEPRINT) => match identifier.ident.as_str() {
                VALIDATOR_REGISTER_IDENT => self.fixed_low,
                VALIDATOR_UNREGISTER_IDENT => self.fixed_low,
                VALIDATOR_STAKE_IDENT => self.fixed_low,
                VALIDATOR_UNSTAKE_IDENT => self.fixed_low,
                VALIDATOR_CLAIM_XRD_IDENT => self.fixed_low,
                VALIDATOR_UPDATE_KEY_IDENT => self.fixed_low,
                VALIDATOR_UPDATE_ACCEPT_DELEGATED_STAKE_IDENT => self.fixed_low,
                _ => self.fixed_low,
            },
            (CLOCK_PACKAGE, CLOCK_BLUEPRINT) => match identifier.ident.as_str() {
                CLOCK_GET_CURRENT_TIME_IDENT => self.fixed_low,
                CLOCK_SET_CURRENT_TIME_IDENT => self.fixed_high,
                CLOCK_COMPARE_CURRENT_TIME_IDENT => self.fixed_high,
                _ => self.fixed_low,
            },
            (ACCESS_CONTROLLER_PACKAGE, ACCESS_CONTROLLER_BLUEPRINT) => {
                match identifier.ident.as_str() {
                    ACCESS_CONTROLLER_CREATE_GLOBAL_IDENT => self.fixed_low,
                    ACCESS_CONTROLLER_CREATE_PROOF_IDENT => self.fixed_low,
                    ACCESS_CONTROLLER_INITIATE_RECOVERY_AS_PRIMARY_IDENT => self.fixed_low,
                    ACCESS_CONTROLLER_INITIATE_RECOVERY_AS_RECOVERY_IDENT => self.fixed_low,
                    ACCESS_CONTROLLER_QUICK_CONFIRM_PRIMARY_ROLE_RECOVERY_PROPOSAL_IDENT => {
                        self.fixed_low
                    }
                    ACCESS_CONTROLLER_QUICK_CONFIRM_RECOVERY_ROLE_RECOVERY_PROPOSAL_IDENT => {
                        self.fixed_low
                    }
                    ACCESS_CONTROLLER_TIMED_CONFIRM_RECOVERY_IDENT => self.fixed_low,
                    ACCESS_CONTROLLER_CANCEL_PRIMARY_ROLE_RECOVERY_PROPOSAL_IDENT => self.fixed_low,
                    ACCESS_CONTROLLER_CANCEL_RECOVERY_ROLE_RECOVERY_PROPOSAL_IDENT => {
                        self.fixed_low
                    }
                    ACCESS_CONTROLLER_LOCK_PRIMARY_ROLE => self.fixed_low,
                    ACCESS_CONTROLLER_UNLOCK_PRIMARY_ROLE => self.fixed_low,
                    _ => self.fixed_low,
                }
            }
            (ACCOUNT_PACKAGE, ACCOUNT_BLUEPRINT) => match identifier.ident.as_str() {
                ACCOUNT_CREATE_LOCAL_IDENT => self.fixed_low,
                ACCOUNT_CREATE_GLOBAL_IDENT => self.fixed_low,
                ACCOUNT_LOCK_FEE_IDENT => self.fixed_low,
                ACCOUNT_LOCK_CONTINGENT_FEE_IDENT => self.fixed_low,
                ACCOUNT_DEPOSIT_IDENT => self.fixed_low,
                ACCOUNT_DEPOSIT_BATCH_IDENT => self.fixed_low,
                ACCOUNT_WITHDRAW_IDENT => self.fixed_low,
                ACCOUNT_WITHDRAW_ALL_IDENT => self.fixed_low,
                ACCOUNT_WITHDRAW_NON_FUNGIBLES_IDENT => self.fixed_low,
                ACCOUNT_LOCK_FEE_AND_WITHDRAW_IDENT => self.fixed_low,
                ACCOUNT_LOCK_FEE_AND_WITHDRAW_ALL_IDENT => self.fixed_low,
                ACCOUNT_LOCK_FEE_AND_WITHDRAW_NON_FUNGIBLES_IDENT => self.fixed_low,
                ACCOUNT_CREATE_PROOF_IDENT => self.fixed_low,
                ACCOUNT_CREATE_PROOF_BY_AMOUNT_IDENT => self.fixed_low,
                ACCOUNT_CREATE_PROOF_BY_IDS_IDENT => self.fixed_low,
                _ => self.fixed_low,
            },

            _ => 0u32,
        }
    }

    pub fn run_native_fn_cost(&self, native_fn: &NativeFn) -> u32 {
        match native_fn {
            NativeFn::Root => panic!("Should not get here"),
            NativeFn::AuthZoneStack(auth_zone_ident) => {
                match auth_zone_ident {
                    AuthZoneStackFn::Pop => self.fixed_low,
                    AuthZoneStackFn::Push => self.fixed_low,
                    AuthZoneStackFn::CreateProof => self.fixed_high, // TODO: charge differently based on auth zone size and fungibility
                    AuthZoneStackFn::CreateProofByAmount => self.fixed_high,
                    AuthZoneStackFn::CreateProofByIds => self.fixed_high,
                    AuthZoneStackFn::Clear => self.fixed_high,
                    AuthZoneStackFn::Drain => self.fixed_high,
                    AuthZoneStackFn::AssertAccessRule => self.fixed_high,
                }
            }
            NativeFn::AccessRulesChain(component_ident) => match component_ident {
                AccessRulesChainFn::AddAccessCheck => self.fixed_low,
                AccessRulesChainFn::SetMethodAccessRule => self.fixed_low,
                AccessRulesChainFn::SetMethodMutability => self.fixed_low,
                AccessRulesChainFn::SetGroupAccessRule => self.fixed_low,
                AccessRulesChainFn::SetGroupMutability => self.fixed_low,
                AccessRulesChainFn::GetLength => self.fixed_low,
            },
            NativeFn::Metadata(metadata_method) => match metadata_method {
                MetadataFn::Set => self.fixed_low,
                MetadataFn::Get => self.fixed_low,
            },
            NativeFn::Component(method_ident) => match method_ident {
                ComponentFn::Globalize => self.fixed_high,
                ComponentFn::GlobalizeWithOwner => self.fixed_high,
                ComponentFn::SetRoyaltyConfig => self.fixed_medium,
                ComponentFn::ClaimRoyalty => self.fixed_medium,
            },
            NativeFn::Package(method_ident) => match method_ident {
                PackageFn::Publish => self.fixed_high,
                PackageFn::PublishNative => self.fixed_high,
                PackageFn::SetRoyaltyConfig => self.fixed_medium,
                PackageFn::ClaimRoyalty => self.fixed_medium,
            },
            NativeFn::TransactionRuntime(ident) => match ident {
                TransactionRuntimeFn::GetHash => self.fixed_low,
                TransactionRuntimeFn::GenerateUuid => self.fixed_low,
            },
            NativeFn::TransactionProcessor(transaction_processor_fn) => {
                match transaction_processor_fn {
                    TransactionProcessorFn::Run => self.fixed_high,
                }
            }
        }
    }

    pub fn kernel_api_cost(&self, entry: CostingEntry) -> u32 {
        match entry {
            CostingEntry::Invoke { input_size } => self.fixed_low + (10 * input_size) as u32,

            CostingEntry::CreateNode { size } => self.fixed_medium + (100 * size) as u32,
            CostingEntry::DropNode { size } => self.fixed_medium + (100 * size) as u32,

            CostingEntry::LockSubstate => self.fixed_high,
            CostingEntry::ReadSubstate { size } => self.fixed_medium + 100 * size,
            CostingEntry::WriteSubstate { size } => self.fixed_medium + 1000 * size,
            CostingEntry::DropLock => self.fixed_high,
        }
    }
}
