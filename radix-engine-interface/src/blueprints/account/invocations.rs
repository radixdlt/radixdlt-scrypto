use crate::api::component::ComponentAddress;
use crate::api::types::*;
use crate::blueprints::resource::*;
use crate::*;
use radix_engine_interface::math::Decimal;
use sbor::rust::collections::BTreeSet;
use sbor::rust::fmt::Debug;

//================
// Account Create
//================

#[derive(
    Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode, LegacyDescribe,
)]
pub struct AccountCreateInput {
    pub withdraw_rule: AccessRule,
}

//=============
// Account New
//=============

#[derive(
    Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode, LegacyDescribe,
)]
pub struct AccountNewInput {
    pub withdraw_rule: AccessRule,
}

//==================
// Account Lock Fee
//==================

#[derive(Debug, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode, LegacyDescribe)]
pub struct AccountLockFeeMethodArgs {
    pub amount: Decimal,
}

#[derive(
    Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode, LegacyDescribe,
)]
pub struct AccountLockFeeInvocation {
    pub receiver: ComponentAddress,
    pub amount: Decimal,
}

impl Invocation for AccountLockFeeInvocation {
    type Output = ();

    fn fn_identifier(&self) -> FnIdentifier {
        FnIdentifier::Native(NativeFn::Account(AccountFn::LockFee))
    }
}

impl SerializableInvocation for AccountLockFeeInvocation {
    type ScryptoOutput = ();

    fn native_fn() -> NativeFn {
        NativeFn::Account(AccountFn::LockFee)
    }
}

impl Into<CallTableInvocation> for AccountLockFeeInvocation {
    fn into(self) -> CallTableInvocation {
        NativeInvocation::Account(AccountInvocation::LockFee(self)).into()
    }
}

//=============================
// Account Lock Contingent Fee
//=============================

#[derive(Debug, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode, LegacyDescribe)]
pub struct AccountLockContingentFeeMethodArgs {
    pub amount: Decimal,
}

#[derive(
    Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode, LegacyDescribe,
)]
pub struct AccountLockContingentFeeInvocation {
    pub receiver: ComponentAddress,
    pub amount: Decimal,
}

impl Invocation for AccountLockContingentFeeInvocation {
    type Output = ();

    fn fn_identifier(&self) -> FnIdentifier {
        FnIdentifier::Native(NativeFn::Account(AccountFn::LockContingentFee))
    }
}

impl SerializableInvocation for AccountLockContingentFeeInvocation {
    type ScryptoOutput = ();

    fn native_fn() -> NativeFn {
        NativeFn::Account(AccountFn::LockContingentFee)
    }
}

impl Into<CallTableInvocation> for AccountLockContingentFeeInvocation {
    fn into(self) -> CallTableInvocation {
        NativeInvocation::Account(AccountInvocation::LockContingentFee(self)).into()
    }
}

//=================
// Account Deposit
//=================

#[derive(Debug, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode, LegacyDescribe)]
pub struct AccountDepositMethodArgs {
    pub bucket: Bucket,
}

#[derive(
    Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode, LegacyDescribe,
)]
pub struct AccountDepositInvocation {
    pub receiver: ComponentAddress,
    pub bucket: BucketId,
}

impl Invocation for AccountDepositInvocation {
    type Output = ();

    fn fn_identifier(&self) -> FnIdentifier {
        FnIdentifier::Native(NativeFn::Account(AccountFn::Deposit))
    }
}

impl SerializableInvocation for AccountDepositInvocation {
    type ScryptoOutput = ();

    fn native_fn() -> NativeFn {
        NativeFn::Account(AccountFn::Deposit)
    }
}

impl Into<CallTableInvocation> for AccountDepositInvocation {
    fn into(self) -> CallTableInvocation {
        NativeInvocation::Account(AccountInvocation::Deposit(self)).into()
    }
}

//=======================
// Account Deposit Batch
//=======================

#[derive(Debug, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode, LegacyDescribe)]
pub struct AccountDepositBatchMethodArgs {
    pub buckets: Vec<Bucket>,
}

#[derive(
    Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode, LegacyDescribe,
)]
pub struct AccountDepositBatchInvocation {
    pub receiver: ComponentAddress,
    pub buckets: Vec<BucketId>,
}

impl Invocation for AccountDepositBatchInvocation {
    type Output = ();

    fn fn_identifier(&self) -> FnIdentifier {
        FnIdentifier::Native(NativeFn::Account(AccountFn::DepositBatch))
    }
}

impl SerializableInvocation for AccountDepositBatchInvocation {
    type ScryptoOutput = ();

    fn native_fn() -> NativeFn {
        NativeFn::Account(AccountFn::DepositBatch)
    }
}

impl Into<CallTableInvocation> for AccountDepositBatchInvocation {
    fn into(self) -> CallTableInvocation {
        NativeInvocation::Account(AccountInvocation::DepositBatch(self)).into()
    }
}

//==================
// Account Withdraw
//==================

#[derive(Debug, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode, LegacyDescribe)]
pub struct AccountWithdrawAllMethodArgs {
    pub resource_address: ResourceAddress,
}

#[derive(
    Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode, LegacyDescribe,
)]
pub struct AccountWithdrawAllInvocation {
    pub receiver: ComponentAddress,
    pub resource_address: ResourceAddress,
}

impl Invocation for AccountWithdrawAllInvocation {
    type Output = Bucket;

    fn fn_identifier(&self) -> FnIdentifier {
        FnIdentifier::Native(NativeFn::Account(AccountFn::WithdrawAll))
    }
}

impl SerializableInvocation for AccountWithdrawAllInvocation {
    type ScryptoOutput = Bucket;

    fn native_fn() -> NativeFn {
        NativeFn::Account(AccountFn::WithdrawAll)
    }
}

impl Into<CallTableInvocation> for AccountWithdrawAllInvocation {
    fn into(self) -> CallTableInvocation {
        NativeInvocation::Account(AccountInvocation::WithdrawAll(self)).into()
    }
}

//============================
// Account Withdraw By Amount
//============================

#[derive(Debug, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode, LegacyDescribe)]
pub struct AccountWithdrawMethodArgs {
    pub resource_address: ResourceAddress,
    pub amount: Decimal,
}

#[derive(
    Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode, LegacyDescribe,
)]
pub struct AccountWithdrawInvocation {
    pub receiver: ComponentAddress,
    pub resource_address: ResourceAddress,
    pub amount: Decimal,
}

impl Invocation for AccountWithdrawInvocation {
    type Output = Bucket;

    fn fn_identifier(&self) -> FnIdentifier {
        FnIdentifier::Native(NativeFn::Account(AccountFn::Withdraw))
    }
}

impl SerializableInvocation for AccountWithdrawInvocation {
    type ScryptoOutput = Bucket;

    fn native_fn() -> NativeFn {
        NativeFn::Account(AccountFn::Withdraw)
    }
}

impl Into<CallTableInvocation> for AccountWithdrawInvocation {
    fn into(self) -> CallTableInvocation {
        NativeInvocation::Account(AccountInvocation::Withdraw(self)).into()
    }
}

//=========================
// Account Withdraw By Ids
//=========================

#[derive(Debug, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode, LegacyDescribe)]
pub struct AccountWithdrawNonFungiblesMethodArgs {
    pub resource_address: ResourceAddress,
    pub ids: BTreeSet<NonFungibleLocalId>,
}

#[derive(
    Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode, LegacyDescribe,
)]
pub struct AccountWithdrawNonFungiblesInvocation {
    pub receiver: ComponentAddress,
    pub resource_address: ResourceAddress,
    pub ids: BTreeSet<NonFungibleLocalId>,
}

impl Invocation for AccountWithdrawNonFungiblesInvocation {
    type Output = Bucket;

    fn fn_identifier(&self) -> FnIdentifier {
        FnIdentifier::Native(NativeFn::Account(AccountFn::WithdrawNonFungibles))
    }
}

impl SerializableInvocation for AccountWithdrawNonFungiblesInvocation {
    type ScryptoOutput = Bucket;

    fn native_fn() -> NativeFn {
        NativeFn::Account(AccountFn::WithdrawNonFungibles)
    }
}

impl Into<CallTableInvocation> for AccountWithdrawNonFungiblesInvocation {
    fn into(self) -> CallTableInvocation {
        NativeInvocation::Account(AccountInvocation::WithdrawNonFungibles(self)).into()
    }
}

//===========================
// Account Withdraw And Lock
//===========================

#[derive(Debug, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode, LegacyDescribe)]
pub struct AccountLockFeeAndWithdrawAllMethodArgs {
    pub amount_to_lock: Decimal,
    pub resource_address: ResourceAddress,
}

#[derive(
    Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode, LegacyDescribe,
)]
pub struct AccountLockFeeAndWithdrawAllInvocation {
    pub receiver: ComponentAddress,
    pub amount_to_lock: Decimal,
    pub resource_address: ResourceAddress,
}

impl Invocation for AccountLockFeeAndWithdrawAllInvocation {
    type Output = Bucket;

    fn fn_identifier(&self) -> FnIdentifier {
        FnIdentifier::Native(NativeFn::Account(AccountFn::LockFeeAndWithdrawAll))
    }
}

impl SerializableInvocation for AccountLockFeeAndWithdrawAllInvocation {
    type ScryptoOutput = Bucket;

    fn native_fn() -> NativeFn {
        NativeFn::Account(AccountFn::LockFeeAndWithdrawAll)
    }
}

impl Into<CallTableInvocation> for AccountLockFeeAndWithdrawAllInvocation {
    fn into(self) -> CallTableInvocation {
        NativeInvocation::Account(AccountInvocation::LockFeeAndWithdrawAll(self)).into()
    }
}

//=====================================
// Account Withdraw By Amount And Lock
//=====================================

#[derive(Debug, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode, LegacyDescribe)]
pub struct AccountLockFeeAndWithdrawMethodArgs {
    pub amount_to_lock: Decimal,
    pub resource_address: ResourceAddress,
    pub amount: Decimal,
}

#[derive(
    Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode, LegacyDescribe,
)]
pub struct AccountLockFeeAndWithdrawInvocation {
    pub receiver: ComponentAddress,
    pub amount_to_lock: Decimal,
    pub resource_address: ResourceAddress,
    pub amount: Decimal,
}

impl Invocation for AccountLockFeeAndWithdrawInvocation {
    type Output = Bucket;

    fn fn_identifier(&self) -> FnIdentifier {
        FnIdentifier::Native(NativeFn::Account(AccountFn::LockFeeAndWithdraw))
    }
}

impl SerializableInvocation for AccountLockFeeAndWithdrawInvocation {
    type ScryptoOutput = Bucket;

    fn native_fn() -> NativeFn {
        NativeFn::Account(AccountFn::LockFeeAndWithdraw)
    }
}

impl Into<CallTableInvocation> for AccountLockFeeAndWithdrawInvocation {
    fn into(self) -> CallTableInvocation {
        NativeInvocation::Account(AccountInvocation::LockFeeAndWithdraw(self)).into()
    }
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
