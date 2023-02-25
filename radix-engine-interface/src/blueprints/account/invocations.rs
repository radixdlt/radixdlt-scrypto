use crate::api::types::*;
use crate::blueprints::resource::*;
use crate::data::model::Own;
use crate::*;
use radix_engine_interface::abi::LegacyDescribe;
use radix_engine_interface::math::Decimal;
use sbor::rust::collections::{BTreeMap, BTreeSet};
use sbor::rust::fmt::Debug;
use scrypto_abi::Fn;
use scrypto_abi::{BlueprintAbi, Fields, Type};
use transaction_data::*;

pub struct AccountAbi;

impl AccountAbi {
    pub fn blueprint_abis() -> BTreeMap<String, BlueprintAbi> {
        let fns = {
            let mut fns = Vec::new();
            // TODO: Add other functions/methods
            {
                let fn_def = Fn {
                    ident: ACCOUNT_LOCK_FEE_IDENT.to_string(),
                    export_name: ACCOUNT_LOCK_FEE_IDENT.to_string(),
                    mutability: Some(abi::SelfMutability::Mutable),
                    input: AccountLockFeeInput::describe(),
                    output: AccountLockFeeOutput::describe(),
                };
                fns.push(fn_def);
            }
            {
                let fn_def = Fn {
                    ident: ACCOUNT_LOCK_CONTINGENT_FEE_IDENT.to_string(),
                    export_name: ACCOUNT_LOCK_CONTINGENT_FEE_IDENT.to_string(),
                    mutability: Some(abi::SelfMutability::Mutable),
                    input: AccountLockContingentFeeInput::describe(),
                    output: AccountLockContingentFeeOutput::describe(),
                };
                fns.push(fn_def);
            }
            {
                let fn_def = Fn {
                    ident: ACCOUNT_DEPOSIT_IDENT.to_string(),
                    export_name: ACCOUNT_DEPOSIT_IDENT.to_string(),
                    mutability: Some(abi::SelfMutability::Mutable),
                    input: AccountDepositInput::describe(),
                    output: AccountDepositOutput::describe(),
                };
                fns.push(fn_def);
            }
            {
                let fn_def = Fn {
                    ident: ACCOUNT_DEPOSIT_BATCH_IDENT.to_string(),
                    export_name: ACCOUNT_DEPOSIT_BATCH_IDENT.to_string(),
                    mutability: Some(abi::SelfMutability::Mutable),
                    input: AccountDepositBatchInput::describe(),
                    output: AccountDepositBatchOutput::describe(),
                };
                fns.push(fn_def);
            }
            {
                let fn_def = Fn {
                    ident: ACCOUNT_WITHDRAW_IDENT.to_string(),
                    export_name: ACCOUNT_WITHDRAW_IDENT.to_string(),
                    mutability: Some(abi::SelfMutability::Mutable),
                    input: AccountWithdrawInput::describe(),
                    output: AccountWithdrawOutput::describe(),
                };
                fns.push(fn_def);
            }
            {
                let fn_def = Fn {
                    ident: ACCOUNT_WITHDRAW_NON_FUNGIBLES_IDENT.to_string(),
                    export_name: ACCOUNT_WITHDRAW_NON_FUNGIBLES_IDENT.to_string(),
                    mutability: Some(abi::SelfMutability::Mutable),
                    input: AccountWithdrawNonFungiblesInput::describe(),
                    output: AccountWithdrawNonFungiblesOutput::describe(),
                };
                fns.push(fn_def);
            }
            {
                let fn_def = Fn {
                    ident: ACCOUNT_LOCK_FEE_AND_WITHDRAW_IDENT.to_string(),
                    export_name: ACCOUNT_LOCK_FEE_AND_WITHDRAW_IDENT.to_string(),
                    mutability: Some(abi::SelfMutability::Mutable),
                    input: AccountLockFeeAndWithdrawInput::describe(),
                    output: AccountLockFeeAndWithdrawOutput::describe(),
                };
                fns.push(fn_def);
            }
            {
                let fn_def = Fn {
                    ident: ACCOUNT_LOCK_FEE_AND_WITHDRAW_NON_FUNGIBLES_IDENT.to_string(),
                    export_name: ACCOUNT_LOCK_FEE_AND_WITHDRAW_NON_FUNGIBLES_IDENT.to_string(),
                    mutability: Some(abi::SelfMutability::Mutable),
                    input: AccountLockFeeAndWithdrawNonFungiblesInput::describe(),
                    output: AccountLockFeeAndWithdrawNonFungiblesOutput::describe(),
                };
                fns.push(fn_def);
            }
            {
                let fn_def = Fn {
                    ident: ACCOUNT_CREATE_PROOF_IDENT.to_string(),
                    export_name: ACCOUNT_CREATE_PROOF_IDENT.to_string(),
                    mutability: Some(abi::SelfMutability::Mutable),
                    input: AccountCreateProofInput::describe(),
                    output: AccountCreateProofOutput::describe(),
                };
                fns.push(fn_def);
            }
            {
                let fn_def = Fn {
                    ident: ACCOUNT_CREATE_PROOF_BY_AMOUNT_IDENT.to_string(),
                    export_name: ACCOUNT_CREATE_PROOF_BY_AMOUNT_IDENT.to_string(),
                    mutability: Some(abi::SelfMutability::Mutable),
                    input: AccountCreateProofByAmountInput::describe(),
                    output: AccountCreateProofByAmountOutput::describe(),
                };
                fns.push(fn_def);
            }
            {
                let fn_def = Fn {
                    ident: ACCOUNT_CREATE_PROOF_BY_IDS_IDENT.to_string(),
                    export_name: ACCOUNT_CREATE_PROOF_BY_IDS_IDENT.to_string(),
                    mutability: Some(abi::SelfMutability::Mutable),
                    input: AccountCreateProofByIdsInput::describe(),
                    output: AccountCreateProofByIdsOutput::describe(),
                };
                fns.push(fn_def);
            }
            fns
        };
        let account_abi = BlueprintAbi {
            structure: Type::Struct {
                name: "Account".into(),
                fields: Fields::Unit, // TODO: Add fields
            },
            fns,
        };

        let mut abis = BTreeMap::new();
        abis.insert(ACCOUNT_BLUEPRINT.to_string(), account_abi);
        abis
    }
}

pub const ACCOUNT_BLUEPRINT: &str = "Account";

//================
// Account Create
//================

pub const ACCOUNT_CREATE_LOCAL_IDENT: &str = "create_local";

#[derive(
    Debug,
    Clone,
    Eq,
    PartialEq,
    ScryptoSbor,
    ManifestCategorize,
    ManifestEncode,
    ManifestDecode,
    LegacyDescribe,
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
    Debug,
    Clone,
    Eq,
    PartialEq,
    ScryptoSbor,
    ManifestCategorize,
    ManifestEncode,
    ManifestDecode,
    LegacyDescribe,
)]
pub struct AccountCreateGlobalInput {
    pub withdraw_rule: AccessRule,
}

pub type AccountCreateGlobalOutput = ComponentAddress;

//==================
// Account Lock Fee
//==================

pub const ACCOUNT_LOCK_FEE_IDENT: &str = "lock_fee";

#[derive(
    Debug,
    Eq,
    PartialEq,
    ScryptoSbor,
    ManifestCategorize,
    ManifestEncode,
    ManifestDecode,
    LegacyDescribe,
)]
pub struct AccountLockFeeInput {
    pub amount: Decimal,
}

pub type AccountLockFeeOutput = ();

//=============================
// Account Lock Contingent Fee
//=============================

pub const ACCOUNT_LOCK_CONTINGENT_FEE_IDENT: &str = "lock_contingent_fee";

#[derive(
    Debug,
    Eq,
    PartialEq,
    ScryptoSbor,
    ManifestCategorize,
    ManifestEncode,
    ManifestDecode,
    LegacyDescribe,
)]
pub struct AccountLockContingentFeeInput {
    pub amount: Decimal,
}

pub type AccountLockContingentFeeOutput = ();

//=================
// Account Deposit
//=================

pub const ACCOUNT_DEPOSIT_IDENT: &str = "deposit";

#[derive(Debug, Eq, PartialEq, ScryptoSbor, LegacyDescribe)]
pub struct AccountDepositInput {
    pub bucket: Bucket,
}

pub type AccountDepositOutput = ();

//=======================
// Account Deposit Batch
//=======================

pub const ACCOUNT_DEPOSIT_BATCH_IDENT: &str = "deposit_batch";

#[derive(Debug, Eq, PartialEq, ScryptoSbor, LegacyDescribe)]
pub struct AccountDepositBatchInput {
    pub buckets: Vec<Bucket>,
}

pub type AccountDepositBatchOutput = ();

//============================
// Account Withdraw
//============================

pub const ACCOUNT_WITHDRAW_IDENT: &str = "withdraw";

#[derive(
    Debug,
    Eq,
    PartialEq,
    ScryptoSbor,
    ManifestCategorize,
    ManifestEncode,
    ManifestDecode,
    LegacyDescribe,
)]
pub struct AccountWithdrawInput {
    pub resource_address: ResourceAddress,
    pub amount: Decimal,
}

pub type AccountWithdrawOutput = Bucket;

//=========================
// Account Withdraw By Ids
//=========================

pub const ACCOUNT_WITHDRAW_NON_FUNGIBLES_IDENT: &str = "withdraw_non_fungibles";

#[derive(
    Debug,
    Eq,
    PartialEq,
    ScryptoSbor,
    ManifestCategorize,
    ManifestEncode,
    ManifestDecode,
    LegacyDescribe,
)]
pub struct AccountWithdrawNonFungiblesInput {
    pub resource_address: ResourceAddress,
    pub ids: BTreeSet<NonFungibleLocalId>,
}

pub type AccountWithdrawNonFungiblesOutput = Bucket;

//=====================================
// Account Withdraw
//=====================================

pub const ACCOUNT_LOCK_FEE_AND_WITHDRAW_IDENT: &str = "lock_fee_and_withdraw";

#[derive(
    Debug,
    Eq,
    PartialEq,
    ScryptoSbor,
    ManifestCategorize,
    ManifestEncode,
    ManifestDecode,
    LegacyDescribe,
)]
pub struct AccountLockFeeAndWithdrawInput {
    pub amount_to_lock: Decimal,
    pub resource_address: ResourceAddress,
    pub amount: Decimal,
}

pub type AccountLockFeeAndWithdrawOutput = ();

//==================================
// Account Withdraw By Ids And Lock
//==================================

pub const ACCOUNT_LOCK_FEE_AND_WITHDRAW_NON_FUNGIBLES_IDENT: &str =
    "lock_fee_and_withdraw_non_fungibles";

#[derive(
    Debug,
    Eq,
    PartialEq,
    ScryptoSbor,
    ManifestCategorize,
    ManifestEncode,
    ManifestDecode,
    LegacyDescribe,
)]
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

#[derive(
    Debug,
    Eq,
    PartialEq,
    ScryptoSbor,
    ManifestCategorize,
    ManifestEncode,
    ManifestDecode,
    LegacyDescribe,
)]
pub struct AccountCreateProofInput {
    pub resource_address: ResourceAddress,
}

pub type AccountCreateProofOutput = Proof;

//================================
// Account Create Proof By Amount
//================================

pub const ACCOUNT_CREATE_PROOF_BY_AMOUNT_IDENT: &str = "create_proof_by_amount";

#[derive(
    Debug,
    Eq,
    PartialEq,
    ScryptoSbor,
    ManifestCategorize,
    ManifestEncode,
    ManifestDecode,
    LegacyDescribe,
)]
pub struct AccountCreateProofByAmountInput {
    pub resource_address: ResourceAddress,
    pub amount: Decimal,
}

pub type AccountCreateProofByAmountOutput = Proof;

//=============================
// Account Create Proof By Ids
//=============================

pub const ACCOUNT_CREATE_PROOF_BY_IDS_IDENT: &str = "create_proof_by_ids";

#[derive(
    Debug,
    Eq,
    PartialEq,
    ScryptoSbor,
    ManifestCategorize,
    ManifestEncode,
    ManifestDecode,
    LegacyDescribe,
)]
pub struct AccountCreateProofByIdsInput {
    pub resource_address: ResourceAddress,
    pub ids: BTreeSet<NonFungibleLocalId>,
}

pub type AccountCreateProofByIdsOutput = Proof;
