use crate::blueprints::resource::*;
use crate::data::scrypto::model::Own;
use crate::data::scrypto::model::*;
use crate::*;
#[cfg(feature = "radix_engine_fuzzing")]
use arbitrary::Arbitrary;
use radix_engine_common::types::*;
use radix_engine_interface::math::Decimal;
use sbor::rust::collections::BTreeSet;
use sbor::rust::fmt::Debug;
use sbor::rust::prelude::*;

#[derive(Debug, Eq, PartialEq, ScryptoSbor, ManifestSbor, Clone)]
pub enum AccountDepositsMode {
    /// Allows the deposit of all resources. Equivalent to a DisallowList of an empty set.
    AllowAll,

    /// Only allows deposits of resources that the account has a vault for (i.e., existing).
    AllowExisting,

    /// Only allows deposits of resources specified by the owner of the account.
    AllowList(IndexSet<ResourceAddress>),

    /// Disallows deposits of resources specified by the owner of the account.
    DisallowList(IndexSet<ResourceAddress>),
}

pub const ACCOUNT_BLUEPRINT: &str = "Account";

pub const ACCOUNT_CREATE_VIRTUAL_ECDSA_SECP256K1_ID: u8 = 0u8;
pub const ACCOUNT_CREATE_VIRTUAL_EDDSA_ED25519_ID: u8 = 1u8;

//================
// Account Create Local
//================

pub const ACCOUNT_CREATE_LOCAL_IDENT: &str = "create_local";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor, ManifestSbor)]
pub struct AccountCreateLocalInput {}

pub type AccountCreateLocalOutput = Own;

//=============
// Account Create Advanced
//=============

pub const ACCOUNT_CREATE_ADVANCED_IDENT: &str = "create_advanced";

#[cfg_attr(feature = "radix_engine_fuzzing", derive(Arbitrary))]
#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor, ManifestSbor)]
pub struct AccountCreateAdvancedInput {
    pub authority_rules: AuthorityRules,
}

pub type AccountCreateAdvancedOutput = ComponentAddress;

//=============
// Account Create
//=============

pub const ACCOUNT_CREATE_IDENT: &str = "create";

#[cfg_attr(feature = "radix_engine_fuzzing", derive(Arbitrary))]
#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor, ManifestSbor)]
pub struct AccountCreateInput {}

pub type AccountCreateOutput = (ComponentAddress, Bucket);

//==================
// Account Securify
//==================

pub const ACCOUNT_SECURIFY_IDENT: &str = "securify";

#[derive(Debug, Eq, PartialEq, ScryptoSbor, ManifestSbor)]
pub struct AccountSecurifyInput {}

pub type AccountSecurifyOutput = Bucket;

//==================
// Account Lock Fee
//==================

pub const ACCOUNT_LOCK_FEE_IDENT: &str = "lock_fee";

#[derive(Debug, Eq, PartialEq, ScryptoSbor, ManifestSbor)]
pub struct AccountLockFeeInput {
    pub amount: Decimal,
}

pub type AccountLockFeeOutput = ();

//=============================
// Account Lock Contingent Fee
//=============================

pub const ACCOUNT_LOCK_CONTINGENT_FEE_IDENT: &str = "lock_contingent_fee";

#[derive(Debug, Eq, PartialEq, ScryptoSbor, ManifestSbor)]
pub struct AccountLockContingentFeeInput {
    pub amount: Decimal,
}

pub type AccountLockContingentFeeOutput = ();

//=================
// Account Deposit
//=================

pub const ACCOUNT_DEPOSIT_IDENT: &str = "deposit";

#[derive(Debug, Eq, PartialEq, ScryptoSbor)]
pub struct AccountDepositInput {
    pub bucket: Bucket,
}

pub type AccountDepositOutput = ();

//=======================
// Account Deposit Batch
//=======================

pub const ACCOUNT_DEPOSIT_BATCH_IDENT: &str = "deposit_batch";

#[derive(Debug, Eq, PartialEq, ScryptoSbor)]
pub struct AccountDepositBatchInput {
    pub buckets: Vec<Bucket>,
}

pub type AccountDepositBatchOutput = ();

//============================
// Account Withdraw
//============================

pub const ACCOUNT_WITHDRAW_IDENT: &str = "withdraw";

#[derive(Debug, Eq, PartialEq, ScryptoSbor, ManifestSbor)]
pub struct AccountWithdrawInput {
    pub resource_address: ResourceAddress,
    pub amount: Decimal,
}

pub type AccountWithdrawOutput = Bucket;

//=========================
// Account Withdraw By Ids
//=========================

pub const ACCOUNT_WITHDRAW_NON_FUNGIBLES_IDENT: &str = "withdraw_non_fungibles";

#[derive(Debug, Eq, PartialEq, ScryptoSbor, ManifestSbor)]
pub struct AccountWithdrawNonFungiblesInput {
    pub resource_address: ResourceAddress,
    pub ids: BTreeSet<NonFungibleLocalId>,
}

pub type AccountWithdrawNonFungiblesOutput = Bucket;

//=====================================
// Account Withdraw
//=====================================

pub const ACCOUNT_LOCK_FEE_AND_WITHDRAW_IDENT: &str = "lock_fee_and_withdraw";

#[derive(Debug, Eq, PartialEq, ScryptoSbor, ManifestSbor)]
pub struct AccountLockFeeAndWithdrawInput {
    pub amount_to_lock: Decimal,
    pub resource_address: ResourceAddress,
    pub amount: Decimal,
}

pub type AccountLockFeeAndWithdrawOutput = Bucket;

//==================================
// Account Withdraw By Ids And Lock
//==================================

pub const ACCOUNT_LOCK_FEE_AND_WITHDRAW_NON_FUNGIBLES_IDENT: &str =
    "lock_fee_and_withdraw_non_fungibles";

#[derive(Debug, Eq, PartialEq, ScryptoSbor, ManifestSbor)]
pub struct AccountLockFeeAndWithdrawNonFungiblesInput {
    pub amount_to_lock: Decimal,
    pub resource_address: ResourceAddress,
    pub ids: BTreeSet<NonFungibleLocalId>,
}

pub type AccountLockFeeAndWithdrawNonFungiblesOutput = Bucket;

//======================
// Account Create Proof
//======================

pub const ACCOUNT_CREATE_PROOF_IDENT: &str = "create_proof";

#[derive(Debug, Eq, PartialEq, ScryptoSbor, ManifestSbor)]
pub struct AccountCreateProofInput {
    pub resource_address: ResourceAddress,
}

pub type AccountCreateProofOutput = Proof;

//================================
// Account Create Proof By Amount
//================================

pub const ACCOUNT_CREATE_PROOF_OF_AMOUNT_IDENT: &str = "create_proof_of_amount";

#[derive(Debug, Eq, PartialEq, ScryptoSbor, ManifestSbor)]
pub struct AccountCreateProofOfAmountInput {
    pub resource_address: ResourceAddress,
    pub amount: Decimal,
}

pub type AccountCreateProofOfAmountOutput = Proof;

//=============================
// Account Create Proof By Ids
//=============================

pub const ACCOUNT_CREATE_PROOF_OF_NON_FUNGIBLES_IDENT: &str = "create_proof_of_non_fungibles";

#[derive(Debug, Eq, PartialEq, ScryptoSbor, ManifestSbor)]
pub struct AccountCreateProofOfNonFungiblesInput {
    pub resource_address: ResourceAddress,
    pub ids: BTreeSet<NonFungibleLocalId>,
}

pub type AccountCreateProofOfNonFungiblesOutput = Proof;

//=================================
// Account Transition Deposit Mode
//=================================

pub const ACCOUNT_CHANGE_ALLOWED_DEPOSITS_MODE_IDENT: &str = "change_allowed_deposits_mode";

#[derive(Debug, Eq, PartialEq, ScryptoSbor, ManifestSbor)]
pub struct AccountChangeAllowedDepositsModeInput {
    pub deposit_mode: AccountDepositsMode,
}

pub type AccountChangeAllowedDepositsModeOutput = ();

//=======================================
// Add Resource To Allowed Deposits List
//=======================================

pub const ACCOUNT_ADD_RESOURCE_TO_ALLOWED_DEPOSITS_LIST_IDENT: &str =
    "add_resource_to_allowed_deposits_list";

#[derive(Debug, Eq, PartialEq, ScryptoSbor, ManifestSbor)]
pub struct AccountAddResourceToAllowedDepositsListInput {
    pub resource_address: ResourceAddress,
}

pub type AccountAddResourceToAllowedDepositsListOutput = bool;

//============================================
// Remove Resource From Allowed Deposits List
//============================================

pub const ACCOUNT_REMOVE_RESOURCE_FROM_ALLOWED_DEPOSITS_LIST_IDENT: &str =
    "remove_resource_from_allowed_deposits_list";

#[derive(Debug, Eq, PartialEq, ScryptoSbor, ManifestSbor)]
pub struct AccountRemoveResourceFromAllowedDepositsListInput {
    pub resource_address: ResourceAddress,
}

pub type AccountRemoveResourceFromAllowedDepositsListOutput = bool;

//==========================================
// Add Resource To Disallowed Deposits List
//==========================================

pub const ACCOUNT_ADD_RESOURCE_TO_DISALLOWED_DEPOSITS_LIST_IDENT: &str =
    "add_resource_to_disallowed_deposits_list";

#[derive(Debug, Eq, PartialEq, ScryptoSbor, ManifestSbor)]
pub struct AccountAddResourceToDisallowedDepositsListInput {
    pub resource_address: ResourceAddress,
}

pub type AccountAddResourceToDisallowedDepositsListOutput = bool;

//===============================================
// Remove Resource From Disallowed Deposits List
//===============================================

pub const ACCOUNT_REMOVE_RESOURCE_FROM_DISALLOWED_DEPOSITS_LIST_IDENT: &str =
    "remove_resource_from_disallowed_deposits_list";

#[derive(Debug, Eq, PartialEq, ScryptoSbor, ManifestSbor)]
pub struct AccountRemoveResourceFromDisallowedDepositsListInput {
    pub resource_address: ResourceAddress,
}

pub type AccountRemoveResourceFromDisallowedDepositsListOutput = bool;

//======================
// Account Safe Deposit
//======================

pub const ACCOUNT_SAFE_DEPOSIT_IDENT: &str = "safe_deposit";

#[derive(Debug, Eq, PartialEq, ScryptoSbor)]
pub struct AccountSafeDepositInput {
    pub bucket: Bucket,
}

pub type AccountSafeDepositOutput = Option<Bucket>;

//============================
// Account Safe Deposit Batch
//============================

pub const ACCOUNT_SAFE_DEPOSIT_BATCH_IDENT: &str = "safe_deposit_batch";

#[derive(Debug, Eq, PartialEq, ScryptoSbor)]
pub struct AccountSafeDepositBatchInput {
    pub buckets: Vec<Bucket>,
}

pub type AccountSafeDepositBatchOutput = Vec<Bucket>;
