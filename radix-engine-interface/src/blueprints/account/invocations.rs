use crate::api::types::*;
use crate::blueprints::resource::*;
use crate::*;
use radix_engine_interface::math::Decimal;
use sbor::rust::collections::BTreeSet;
use sbor::rust::fmt::Debug;
use crate::data::types::Own;

pub const ACCOUNT_BLUEPRINT: &str = "Account";

//================
// Account Create
//================

pub const ACCOUNT_CREATE_LOCAL_IDENT: &str = "create_local";

#[derive(
    Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode, LegacyDescribe,
)]
pub struct AccountCreateLocalInput {
    pub withdraw_rule: AccessRule,
}


pub type AccountCreateLocalOutput = Own;

//=============
// Account New
//=============

pub const ACCOUNT_CREATE_GLOBAL_IDENT: &str = "create_global";

#[derive(
    Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode, LegacyDescribe,
)]
pub struct AccountCreateGlobalInput {
    pub withdraw_rule: AccessRule,
}

pub type AccountCreateGlobalOutput = ComponentAddress;

//==================
// Account Lock Fee
//==================

pub const ACCOUNT_LOCK_FEE_IDENT: &str = "lock_fee";

#[derive(Debug, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode, LegacyDescribe)]
pub struct AccountLockFeeInput {
    pub amount: Decimal,
}

pub type AccountLockFeeOutput = ();

//=============================
// Account Lock Contingent Fee
//=============================

pub const ACCOUNT_LOCK_CONTINGENT_FEE_IDENT: &str = "lock_contingent_fee";

#[derive(Debug, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode, LegacyDescribe)]
pub struct AccountLockContingentFeeInput {
    pub amount: Decimal,
}

pub type AccountLockContingentFeeOutput = ();

//=================
// Account Deposit
//=================

pub const ACCOUNT_DEPOSIT_IDENT: &str = "deposit";

#[derive(Debug, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode, LegacyDescribe)]
pub struct AccountDepositInput {
    pub bucket: Bucket,
}

pub type AccountDepositOutput = ();

//=======================
// Account Deposit Batch
//=======================

pub const ACCOUNT_DEPOSIT_BATCH_IDENT: &str = "deposit_batch";

#[derive(Debug, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode, LegacyDescribe)]
pub struct AccountDepositBatchInput {
    pub buckets: Vec<Bucket>,
}

pub type AccountDepositBatchOutput = ();

//============================
// Account Withdraw
//============================

pub const ACCOUNT_WITHDRAW_IDENT: &str = "withdraw";

#[derive(Debug, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode, LegacyDescribe)]
pub struct AccountWithdrawInput {
    pub resource_address: ResourceAddress,
    pub amount: Decimal,
}

pub type AccountWithdrawOutput = Bucket;

//==================
// Account Withdraw All
//==================

pub const ACCOUNT_WITHDRAW_ALL_IDENT: &str = "withdraw_all";

#[derive(Debug, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode, LegacyDescribe)]
pub struct AccountWithdrawAllInput {
    pub resource_address: ResourceAddress,
}

pub type AccountWithdrawAllOutput = Bucket;

//=========================
// Account Withdraw By Ids
//=========================

pub const ACCOUNT_WITHDRAW_NON_FUNGIBLES_IDENT: &str = "withdraw_non_fungibles";

#[derive(Debug, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode, LegacyDescribe)]
pub struct AccountWithdrawNonFungiblesInput {
    pub resource_address: ResourceAddress,
    pub ids: BTreeSet<NonFungibleLocalId>,
}

pub type AccountWithdrawNonFungiblesOutput = Bucket;

//=====================================
// Account Withdraw
//=====================================

pub const ACCOUNT_LOCK_FEE_AND_WITHDRAW_IDENT: &str = "lock_fee_and_withdraw";

#[derive(Debug, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode, LegacyDescribe)]
pub struct AccountLockFeeAndWithdrawInput {
    pub amount_to_lock: Decimal,
    pub resource_address: ResourceAddress,
    pub amount: Decimal,
}

pub type AccountLockFeeAndWithdrawOutput = ();

//===========================
// Account Withdraw All And Lock
//===========================

pub const ACCOUNT_LOCK_FEE_AND_WITHDRAW_ALL_IDENT: &str = "lock_fee_and_withdraw_all";


#[derive(Debug, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode, LegacyDescribe)]
pub struct AccountLockFeeAndWithdrawAllInput {
    pub amount_to_lock: Decimal,
    pub resource_address: ResourceAddress,
}

pub type AccountLockFeeAndWithdrawAllOutput = ();

//==================================
// Account Withdraw By Ids And Lock
//==================================

pub const ACCOUNT_LOCK_FEE_AND_WITHDRAW_NON_FUNGIBLES_IDENT: &str = "lock_fee_and_withdraw_non_fungibles";


#[derive(Debug, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode, LegacyDescribe)]
pub struct AccountLockFeeAndWithdrawNonFungiblesInput {
    pub amount_to_lock: Decimal,
    pub resource_address: ResourceAddress,
    pub ids: BTreeSet<NonFungibleLocalId>,
}

pub type AccountLockFeeAndWithdrawNonFungiblesOutput = ();

//======================
// Account Create Proof
//======================

pub const ACCOUNT_CREATE_PROOF_IDENT: &str = "create_proof";

#[derive(Debug, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode, LegacyDescribe)]
pub struct AccountCreateProofInput {
    pub resource_address: ResourceAddress,
}

pub type AccountCreateProofOutput = Proof;

//================================
// Account Create Proof By Amount
//================================

pub const ACCOUNT_CREATE_PROOF_BY_AMOUNT_IDENT: &str = "create_proof_by_amount";

#[derive(Debug, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode, LegacyDescribe)]
pub struct AccountCreateProofByAmountInput {
    pub resource_address: ResourceAddress,
    pub amount: Decimal,
}

pub type AccountCreateProofByAmountOutput = Proof;

//=============================
// Account Create Proof By Ids
//=============================

pub const ACCOUNT_CREATE_PROOF_BY_IDS_IDENT: &str = "create_proof_by_ids";

#[derive(Debug, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode, LegacyDescribe)]
pub struct AccountCreateProofByIdsInput {
    pub resource_address: ResourceAddress,
    pub ids: BTreeSet<NonFungibleLocalId>,
}

pub type AccountCreateProofByIdsOutput = Proof;
