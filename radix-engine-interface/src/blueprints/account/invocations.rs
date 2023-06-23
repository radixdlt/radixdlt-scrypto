use crate::blueprints::resource::*;
use crate::data::scrypto::model::*;
use crate::*;
#[cfg(feature = "radix_engine_fuzzing")]
use arbitrary::Arbitrary;
use radix_engine_common::types::*;
use radix_engine_interface::math::Decimal;
use sbor::rust::collections::BTreeSet;
use sbor::rust::fmt::Debug;
use sbor::rust::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, ScryptoSbor, ManifestSbor)]
pub enum ResourceDepositRule {
    /// The resource is neither on the allow or deny list.
    Neither,

    /// The resource is on the allow list.
    Allowed,

    /// The resource is on the deny list.
    Disallowed,
}

#[derive(Debug, Eq, PartialEq, ScryptoSbor, ManifestSbor, Clone)]
pub enum AccountDefaultDepositRule {
    /// Allows the deposit of all resources - the deny list is honored in this state.o
    Accept,

    /// Disallows the deposit of all resources - the allow list is honored in this state.
    Reject,

    /// Only deposits of existing resources is accepted - both allow and deny lists are honored in
    /// this mode.
    AllowExisting,
}

pub const ACCOUNT_BLUEPRINT: &str = "Account";

pub const ACCOUNT_CREATE_VIRTUAL_SECP256K1_ID: u8 = 0u8;
pub const ACCOUNT_CREATE_VIRTUAL_ED25519_ID: u8 = 1u8;

//=============
// Account Create Advanced
//=============

pub const ACCOUNT_CREATE_ADVANCED_IDENT: &str = "create_advanced";

#[cfg_attr(feature = "radix_engine_fuzzing", derive(Arbitrary))]
#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor, ManifestSbor)]
pub struct AccountCreateAdvancedInput {
    pub owner_role: OwnerRole,
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

pub const ACCOUNT_CHANGE_DEFAULT_DEPOSIT_RULE_IDENT: &str = "change_account_default_deposit_rule";

#[derive(Debug, Eq, PartialEq, ScryptoSbor, ManifestSbor)]
pub struct AccountChangeDefaultDepositRuleInput {
    pub default_deposit_rule: AccountDefaultDepositRule,
}

pub type AccountChangeDefaultDepositRuleOutput = ();

//============================
// Configure Resource Deposit Rule
//============================

pub const ACCOUNT_CONFIGURE_RESOURCE_DEPOSIT_RULE_IDENT: &str = "configure_resource_deposit_rule";

#[derive(Debug, Eq, PartialEq, ScryptoSbor, ManifestSbor)]
pub struct AccountConfigureResourceDepositRuleInput {
    pub resource_address: ResourceAddress,
    pub resource_deposit_configuration: ResourceDepositRule,
}

pub type AccountConfigureResourceDepositRuleOutput = ();

//===============================
// Account Try Deposit Or Refund
//===============================

pub const ACCOUNT_TRY_DEPOSIT_OR_REFUND_IDENT: &str = "try_deposit_or_refund";

#[derive(Debug, Eq, PartialEq, ScryptoSbor)]
pub struct AccountTryDepositOrRefundInput {
    pub bucket: Bucket,
}

pub type AccountTryDepositOrRefundOutput = Option<Bucket>;

//=====================================
// Account Try Deposit Batch Or Refund
//=====================================

pub const ACCOUNT_TRY_DEPOSIT_BATCH_OR_REFUND_IDENT: &str = "try_deposit_batch_or_refund";

#[derive(Debug, Eq, PartialEq, ScryptoSbor)]
pub struct AccountTryDepositBatchOrRefundInput {
    pub buckets: Vec<Bucket>,
}

pub type AccountTryDepositBatchOrRefundOutput = Vec<Bucket>;

//==============================
// Account Try Deposit Or Abort
//==============================

pub const ACCOUNT_TRY_DEPOSIT_OR_ABORT_IDENT: &str = "try_deposit_or_abort";

#[derive(Debug, Eq, PartialEq, ScryptoSbor)]
pub struct AccountTryDepositOrAbortInput {
    pub bucket: Bucket,
}

pub type AccountTryDepositOrAbortOutput = ();

//====================================
// Account Try Deposit Batch Or Abort
//====================================

pub const ACCOUNT_TRY_DEPOSIT_BATCH_OR_ABORT_IDENT: &str = "try_deposit_batch_or_abort";

#[derive(Debug, Eq, PartialEq, ScryptoSbor)]
pub struct AccountTryDepositBatchOrAbortInput {
    pub buckets: Vec<Bucket>,
}

pub type AccountTryDepositBatchOrAbortOutput = ();

//============================
// Account Burn
//============================

pub const ACCOUNT_BURN_IDENT: &str = "burn";

#[derive(Debug, Eq, PartialEq, ScryptoSbor, ManifestSbor)]
pub struct AccountBurnInput {
    pub resource_address: ResourceAddress,
    pub amount: Decimal,
}

pub type AccountBurnOutput = ();

//=========================
// Account Burn By Ids
//=========================

pub const ACCOUNT_BURN_NON_FUNGIBLES_IDENT: &str = "burn_non_fungibles";

#[derive(Debug, Eq, PartialEq, ScryptoSbor, ManifestSbor)]
pub struct AccountBurnNonFungiblesInput {
    pub resource_address: ResourceAddress,
    pub ids: BTreeSet<NonFungibleLocalId>,
}

pub type AccountBurnNonFungiblesOutput = ();
