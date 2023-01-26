use crate::api::types::BucketId;
use crate::api::types::ComponentId;
use crate::api::wasm::*;
use crate::api::Invocation;
use crate::math::Decimal;
use crate::model::*;
use crate::*;
use sbor::rust::collections::BTreeSet;
use sbor::rust::vec::Vec;

//================
// Account Create
//================

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct AccountCreateInvocation {
    pub withdraw_rule: AccessRule,
}

impl Invocation for AccountCreateInvocation {
    type Output = ComponentId;
}

impl SerializableInvocation for AccountCreateInvocation {
    type ScryptoOutput = ComponentId;
}

impl Into<CallTableInvocation> for AccountCreateInvocation {
    fn into(self) -> CallTableInvocation {
        NativeInvocation::Account(AccountInvocation::Create(self)).into()
    }
}

//=============
// Account New
//=============

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct AccountNewInvocation {
    pub withdraw_rule: AccessRule,
}

impl Invocation for AccountNewInvocation {
    type Output = ComponentAddress;
}

impl SerializableInvocation for AccountNewInvocation {
    type ScryptoOutput = ComponentAddress;
}

impl Into<CallTableInvocation> for AccountNewInvocation {
    fn into(self) -> CallTableInvocation {
        NativeInvocation::Account(AccountInvocation::New(self)).into()
    }
}

//===========================
// Account New With Resource
//===========================

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct AccountNewWithResourceInvocation {
    pub withdraw_rule: AccessRule,
    pub bucket: BucketId,
}

impl Invocation for AccountNewWithResourceInvocation {
    type Output = ComponentAddress;
}

impl SerializableInvocation for AccountNewWithResourceInvocation {
    type ScryptoOutput = ComponentAddress;
}

impl Into<CallTableInvocation> for AccountNewWithResourceInvocation {
    fn into(self) -> CallTableInvocation {
        NativeInvocation::Account(AccountInvocation::NewWithResource(self)).into()
    }
}

//==================
// Account Lock Fee
//==================

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct AccountLockFeeMethodArgs {
    pub amount: Decimal,
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct AccountLockFeeInvocation {
    pub receiver: ComponentAddress,
    pub amount: Decimal,
}

impl Invocation for AccountLockFeeInvocation {
    type Output = ();
}

impl SerializableInvocation for AccountLockFeeInvocation {
    type ScryptoOutput = ();
}

impl Into<CallTableInvocation> for AccountLockFeeInvocation {
    fn into(self) -> CallTableInvocation {
        NativeInvocation::Account(AccountInvocation::LockFee(self)).into()
    }
}

//=============================
// Account Lock Contingent Fee
//=============================

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct AccountLockContingentFeeMethodArgs {
    pub amount: Decimal,
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct AccountLockContingentFeeInvocation {
    pub receiver: ComponentAddress,
    pub amount: Decimal,
}

impl Invocation for AccountLockContingentFeeInvocation {
    type Output = ();
}

impl SerializableInvocation for AccountLockContingentFeeInvocation {
    type ScryptoOutput = ();
}

impl Into<CallTableInvocation> for AccountLockContingentFeeInvocation {
    fn into(self) -> CallTableInvocation {
        NativeInvocation::Account(AccountInvocation::LockContingentFee(self)).into()
    }
}

//=================
// Account Deposit
//=================

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct AccountDepositMethodArgs {
    pub bucket: BucketId,
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct AccountDepositInvocation {
    pub receiver: ComponentAddress,
    pub bucket: BucketId,
}

impl Invocation for AccountDepositInvocation {
    type Output = ();
}

impl SerializableInvocation for AccountDepositInvocation {
    type ScryptoOutput = ();
}

impl Into<CallTableInvocation> for AccountDepositInvocation {
    fn into(self) -> CallTableInvocation {
        NativeInvocation::Account(AccountInvocation::Deposit(self)).into()
    }
}

//=======================
// Account Deposit Batch
//=======================

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct AccountDepositBatchMethodArgs {
    pub buckets: Vec<BucketId>,
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct AccountDepositBatchInvocation {
    pub receiver: ComponentAddress,
    pub buckets: Vec<BucketId>,
}

impl Invocation for AccountDepositBatchInvocation {
    type Output = ();
}

impl SerializableInvocation for AccountDepositBatchInvocation {
    type ScryptoOutput = ();
}

impl Into<CallTableInvocation> for AccountDepositBatchInvocation {
    fn into(self) -> CallTableInvocation {
        NativeInvocation::Account(AccountInvocation::DepositBatch(self)).into()
    }
}

//==================
// Account Withdraw
//==================

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct AccountWithdrawMethodArgs {
    pub resource_address: ResourceAddress,
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct AccountWithdrawInvocation {
    pub receiver: ComponentAddress,
    pub resource_address: ResourceAddress,
}

impl Invocation for AccountWithdrawInvocation {
    type Output = Bucket;
}

impl SerializableInvocation for AccountWithdrawInvocation {
    type ScryptoOutput = Bucket;
}

impl Into<CallTableInvocation> for AccountWithdrawInvocation {
    fn into(self) -> CallTableInvocation {
        NativeInvocation::Account(AccountInvocation::Withdraw(self)).into()
    }
}

//============================
// Account Withdraw By Amount
//============================

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct AccountWithdrawByAmountMethodArgs {
    pub amount: Decimal,
    pub resource_address: ResourceAddress,
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct AccountWithdrawByAmountInvocation {
    pub receiver: ComponentAddress,
    pub amount: Decimal,
    pub resource_address: ResourceAddress,
}

impl Invocation for AccountWithdrawByAmountInvocation {
    type Output = Bucket;
}

impl SerializableInvocation for AccountWithdrawByAmountInvocation {
    type ScryptoOutput = Bucket;
}

impl Into<CallTableInvocation> for AccountWithdrawByAmountInvocation {
    fn into(self) -> CallTableInvocation {
        NativeInvocation::Account(AccountInvocation::WithdrawByAmount(self)).into()
    }
}

//=========================
// Account Withdraw By Ids
//=========================

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct AccountWithdrawByIdsMethodArgs {
    pub ids: BTreeSet<NonFungibleLocalId>,
    pub resource_address: ResourceAddress,
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct AccountWithdrawByIdsInvocation {
    pub receiver: ComponentAddress,
    pub ids: BTreeSet<NonFungibleLocalId>,
    pub resource_address: ResourceAddress,
}

impl Invocation for AccountWithdrawByIdsInvocation {
    type Output = Bucket;
}

impl SerializableInvocation for AccountWithdrawByIdsInvocation {
    type ScryptoOutput = Bucket;
}

impl Into<CallTableInvocation> for AccountWithdrawByIdsInvocation {
    fn into(self) -> CallTableInvocation {
        NativeInvocation::Account(AccountInvocation::WithdrawByIds(self)).into()
    }
}

//===========================
// Account Withdraw And Lock
//===========================

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct AccountLockFeeAndWithdrawMethodArgs {
    pub amount_to_lock: Decimal,
    pub resource_address: ResourceAddress,
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct AccountLockFeeAndWithdrawInvocation {
    pub receiver: ComponentAddress,
    pub amount_to_lock: Decimal,
    pub resource_address: ResourceAddress,
}

impl Invocation for AccountLockFeeAndWithdrawInvocation {
    type Output = Bucket;
}

impl SerializableInvocation for AccountLockFeeAndWithdrawInvocation {
    type ScryptoOutput = Bucket;
}

impl Into<CallTableInvocation> for AccountLockFeeAndWithdrawInvocation {
    fn into(self) -> CallTableInvocation {
        NativeInvocation::Account(AccountInvocation::LockFeeAndWithdraw(self)).into()
    }
}

//=====================================
// Account Withdraw By Amount And Lock
//=====================================

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct AccountLockFeeAndWithdrawByAmountMethodArgs {
    pub amount_to_lock: Decimal,
    pub amount: Decimal,
    pub resource_address: ResourceAddress,
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct AccountLockFeeAndWithdrawByAmountInvocation {
    pub receiver: ComponentAddress,
    pub amount_to_lock: Decimal,
    pub amount: Decimal,
    pub resource_address: ResourceAddress,
}

impl Invocation for AccountLockFeeAndWithdrawByAmountInvocation {
    type Output = Bucket;
}

impl SerializableInvocation for AccountLockFeeAndWithdrawByAmountInvocation {
    type ScryptoOutput = Bucket;
}

impl Into<CallTableInvocation> for AccountLockFeeAndWithdrawByAmountInvocation {
    fn into(self) -> CallTableInvocation {
        NativeInvocation::Account(AccountInvocation::LockFeeAndWithdrawByAmount(self)).into()
    }
}

//==================================
// Account Withdraw By Ids And Lock
//==================================

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct AccountLockFeeAndWithdrawByIdsMethodArgs {
    pub amount_to_lock: Decimal,
    pub ids: BTreeSet<NonFungibleLocalId>,
    pub resource_address: ResourceAddress,
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct AccountLockFeeAndWithdrawByIdsInvocation {
    pub receiver: ComponentAddress,
    pub amount_to_lock: Decimal,
    pub ids: BTreeSet<NonFungibleLocalId>,
    pub resource_address: ResourceAddress,
}

impl Invocation for AccountLockFeeAndWithdrawByIdsInvocation {
    type Output = Bucket;
}

impl SerializableInvocation for AccountLockFeeAndWithdrawByIdsInvocation {
    type ScryptoOutput = Bucket;
}

impl Into<CallTableInvocation> for AccountLockFeeAndWithdrawByIdsInvocation {
    fn into(self) -> CallTableInvocation {
        NativeInvocation::Account(AccountInvocation::LockFeeAndWithdrawByIds(self)).into()
    }
}

//======================
// Account Create Proof
//======================

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct AccountCreateProofMethodArgs {
    pub resource_address: ResourceAddress,
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct AccountCreateProofInvocation {
    pub receiver: ComponentAddress,
    pub resource_address: ResourceAddress,
}

impl Invocation for AccountCreateProofInvocation {
    type Output = Proof;
}

impl SerializableInvocation for AccountCreateProofInvocation {
    type ScryptoOutput = Proof;
}

impl Into<CallTableInvocation> for AccountCreateProofInvocation {
    fn into(self) -> CallTableInvocation {
        NativeInvocation::Account(AccountInvocation::CreateProof(self)).into()
    }
}

//================================
// Account Create Proof By Amount
//================================

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct AccountCreateProofByAmountMethodArgs {
    pub amount: Decimal,
    pub resource_address: ResourceAddress,
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct AccountCreateProofByAmountInvocation {
    pub receiver: ComponentAddress,
    pub amount: Decimal,
    pub resource_address: ResourceAddress,
}

impl Invocation for AccountCreateProofByAmountInvocation {
    type Output = Proof;
}

impl SerializableInvocation for AccountCreateProofByAmountInvocation {
    type ScryptoOutput = Proof;
}

impl Into<CallTableInvocation> for AccountCreateProofByAmountInvocation {
    fn into(self) -> CallTableInvocation {
        NativeInvocation::Account(AccountInvocation::CreateProofByAmount(self)).into()
    }
}

//=============================
// Account Create Proof By Ids
//=============================

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct AccountCreateProofByIdsMethodArgs {
    pub ids: BTreeSet<NonFungibleLocalId>,
    pub resource_address: ResourceAddress,
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct AccountCreateProofByIdsInvocation {
    pub receiver: ComponentAddress,
    pub ids: BTreeSet<NonFungibleLocalId>,
    pub resource_address: ResourceAddress,
}

impl Invocation for AccountCreateProofByIdsInvocation {
    type Output = Proof;
}

impl SerializableInvocation for AccountCreateProofByIdsInvocation {
    type ScryptoOutput = Proof;
}

impl Into<CallTableInvocation> for AccountCreateProofByIdsInvocation {
    fn into(self) -> CallTableInvocation {
        NativeInvocation::Account(AccountInvocation::CreateProofByIds(self)).into()
    }
}
