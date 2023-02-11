use crate::api::component::ComponentAddress;
use crate::api::types::*;
use crate::blueprints::resource::*;
use crate::*;
use radix_engine_interface::math::Decimal;
use sbor::rust::collections::BTreeSet;
use sbor::rust::fmt::Debug;

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

//==================
// Account Lock Fee
//==================

pub const ACCOUNT_LOCK_FEE_IDENT: &str = "lock_fee";

#[derive(Debug, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode, LegacyDescribe)]
pub struct AccountLockFeeInput {
    pub amount: Decimal,
}

//=============================
// Account Lock Contingent Fee
//=============================

pub const ACCOUNT_LOCK_CONTINGENT_FEE_IDENT: &str = "lock_contingent_fee";

#[derive(Debug, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode, LegacyDescribe)]
pub struct AccountLockContingentFeeInput {
    pub amount: Decimal,
}

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

//============================
// Account Withdraw
//============================

pub const ACCOUNT_WITHDRAW_IDENT: &str = "withdraw";

#[derive(Debug, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode, LegacyDescribe)]
pub struct AccountWithdrawInput {
    pub resource_address: ResourceAddress,
    pub amount: Decimal,
}

//==================
// Account Withdraw All
//==================

pub const ACCOUNT_WITHDRAW_ALL_IDENT: &str = "withdraw_all";

#[derive(Debug, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode, LegacyDescribe)]
pub struct AccountWithdrawAllInput {
    pub resource_address: ResourceAddress,
}

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

//===========================
// Account Withdraw All And Lock
//===========================

pub const ACCOUNT_LOCK_FEE_AND_WITHDRAW_ALL_IDENT: &str = "lock_fee_and_withdraw_all";

#[derive(Debug, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode, LegacyDescribe)]
pub struct AccountLockFeeAndWithdrawAllInput {
    pub amount_to_lock: Decimal,
    pub resource_address: ResourceAddress,
}

//==================================
// Account Withdraw By Ids And Lock
//==================================

#[derive(Debug, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode, LegacyDescribe)]
pub struct AccountLockFeeAndWithdrawNonFungiblesMethodArgs {
    pub amount_to_lock: Decimal,
    pub resource_address: ResourceAddress,
    pub ids: BTreeSet<NonFungibleLocalId>,
}

#[derive(
    Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode, LegacyDescribe,
)]
pub struct AccountLockFeeAndWithdrawNonFungiblesInvocation {
    pub receiver: ComponentAddress,
    pub amount_to_lock: Decimal,
    pub resource_address: ResourceAddress,
    pub ids: BTreeSet<NonFungibleLocalId>,
}

impl Invocation for AccountLockFeeAndWithdrawNonFungiblesInvocation {
    type Output = Bucket;

    fn fn_identifier(&self) -> FnIdentifier {
        FnIdentifier::Native(NativeFn::Account(AccountFn::LockFeeAndWithdrawNonFungibles))
    }
}

impl SerializableInvocation for AccountLockFeeAndWithdrawNonFungiblesInvocation {
    type ScryptoOutput = Bucket;

    fn native_fn() -> NativeFn {
        NativeFn::Account(AccountFn::LockFeeAndWithdrawNonFungibles)
    }
}

impl Into<CallTableInvocation> for AccountLockFeeAndWithdrawNonFungiblesInvocation {
    fn into(self) -> CallTableInvocation {
        NativeInvocation::Account(AccountInvocation::LockFeeAndWithdrawNonFungibles(self)).into()
    }
}

//======================
// Account Create Proof
//======================

#[derive(Debug, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode, LegacyDescribe)]
pub struct AccountCreateProofMethodArgs {
    pub resource_address: ResourceAddress,
}

#[derive(
    Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode, LegacyDescribe,
)]
pub struct AccountCreateProofInvocation {
    pub receiver: ComponentAddress,
    pub resource_address: ResourceAddress,
}

impl Invocation for AccountCreateProofInvocation {
    type Output = Proof;

    fn fn_identifier(&self) -> FnIdentifier {
        FnIdentifier::Native(NativeFn::Account(AccountFn::CreateProof))
    }
}

impl SerializableInvocation for AccountCreateProofInvocation {
    type ScryptoOutput = Proof;

    fn native_fn() -> NativeFn {
        NativeFn::Account(AccountFn::CreateProof)
    }
}

impl Into<CallTableInvocation> for AccountCreateProofInvocation {
    fn into(self) -> CallTableInvocation {
        NativeInvocation::Account(AccountInvocation::CreateProof(self)).into()
    }
}

//================================
// Account Create Proof By Amount
//================================

#[derive(Debug, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode, LegacyDescribe)]
pub struct AccountCreateProofByAmountMethodArgs {
    pub resource_address: ResourceAddress,
    pub amount: Decimal,
}

#[derive(
    Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode, LegacyDescribe,
)]
pub struct AccountCreateProofByAmountInvocation {
    pub receiver: ComponentAddress,
    pub resource_address: ResourceAddress,
    pub amount: Decimal,
}

impl Invocation for AccountCreateProofByAmountInvocation {
    type Output = Proof;

    fn fn_identifier(&self) -> FnIdentifier {
        FnIdentifier::Native(NativeFn::Account(AccountFn::CreateProofByAmount))
    }
}

impl SerializableInvocation for AccountCreateProofByAmountInvocation {
    type ScryptoOutput = Proof;

    fn native_fn() -> NativeFn {
        NativeFn::Account(AccountFn::CreateProofByAmount)
    }
}

impl Into<CallTableInvocation> for AccountCreateProofByAmountInvocation {
    fn into(self) -> CallTableInvocation {
        NativeInvocation::Account(AccountInvocation::CreateProofByAmount(self)).into()
    }
}

//=============================
// Account Create Proof By Ids
//=============================

#[derive(Debug, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode, LegacyDescribe)]
pub struct AccountCreateProofByIdsMethodArgs {
    pub resource_address: ResourceAddress,
    pub ids: BTreeSet<NonFungibleLocalId>,
}

#[derive(
    Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode, LegacyDescribe,
)]
pub struct AccountCreateProofByIdsInvocation {
    pub receiver: ComponentAddress,
    pub resource_address: ResourceAddress,
    pub ids: BTreeSet<NonFungibleLocalId>,
}

impl Invocation for AccountCreateProofByIdsInvocation {
    type Output = Proof;

    fn fn_identifier(&self) -> FnIdentifier {
        FnIdentifier::Native(NativeFn::Account(AccountFn::CreateProofByIds))
    }
}

impl SerializableInvocation for AccountCreateProofByIdsInvocation {
    type ScryptoOutput = Proof;

    fn native_fn() -> NativeFn {
        NativeFn::Account(AccountFn::CreateProofByIds)
    }
}

impl Into<CallTableInvocation> for AccountCreateProofByIdsInvocation {
    fn into(self) -> CallTableInvocation {
        NativeInvocation::Account(AccountInvocation::CreateProofByIds(self)).into()
    }
}
