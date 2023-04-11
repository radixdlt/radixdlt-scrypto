use crate::blueprints::resource::*;
use crate::data::scrypto::model::Own;
use crate::data::scrypto::model::*;
use crate::*;
use radix_engine_common::types::*;
use radix_engine_interface::math::Decimal;
use sbor::rust::collections::BTreeSet;
use sbor::rust::fmt::Debug;
use sbor::rust::prelude::*;

pub const ACCOUNT_BLUEPRINT: &str = "Account";

pub const ACCOUNT_CREATE_VIRTUAL_ECDSA_256K1_ID: u8 = 0u8;
pub const ACCOUNT_CREATE_VIRTUAL_EDDSA_255519_ID: u8 = 1u8;

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

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor, ManifestSbor)]
pub struct AccountCreateAdvancedInput {
    pub config: AccessRulesConfig,
}

pub type AccountCreateAdvancedOutput = ComponentAddress;

//=============
// Account Create
//=============

pub const ACCOUNT_CREATE_IDENT: &str = "create";

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

pub const ACCOUNT_CREATE_PROOF_BY_AMOUNT_IDENT: &str = "create_proof_by_amount";

#[derive(Debug, Eq, PartialEq, ScryptoSbor, ManifestSbor)]
pub struct AccountCreateProofByAmountInput {
    pub resource_address: ResourceAddress,
    pub amount: Decimal,
}

pub type AccountCreateProofByAmountOutput = Proof;

//=============================
// Account Create Proof By Ids
//=============================

pub const ACCOUNT_CREATE_PROOF_BY_IDS_IDENT: &str = "create_proof_by_ids";

#[derive(Debug, Eq, PartialEq, ScryptoSbor, ManifestSbor)]
pub struct AccountCreateProofByIdsInput {
    pub resource_address: ResourceAddress,
    pub ids: BTreeSet<NonFungibleLocalId>,
}

pub type AccountCreateProofByIdsOutput = Proof;
