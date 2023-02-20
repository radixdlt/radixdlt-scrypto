use crate::api::types::*;
use crate::blueprints::access_controller::*;
use crate::blueprints::resource::*;
use crate::*;
use sbor::rust::collections::BTreeMap;
use sbor::rust::fmt::Debug;
use scrypto_abi::BlueprintAbi;
use transaction_data::*;

pub struct AccessControllerAbi;

impl AccessControllerAbi {
    pub fn blueprint_abis() -> BTreeMap<String, BlueprintAbi> {
        BTreeMap::new()
    }
}

pub const ACCESS_CONTROLLER_BLUEPRINT: &str = "AccessController";

//=================================
// Access Controller Create Global
//=================================

pub const ACCESS_CONTROLLER_CREATE_GLOBAL_IDENT: &str = "create_global";

#[derive(Debug, Eq, PartialEq, ScryptoSbor)]
pub struct AccessControllerCreateGlobalInput {
    pub controlled_asset: Bucket,
    pub rule_set: RuleSet,
    pub timed_recovery_delay_in_minutes: Option<u32>,
}

impl Clone for AccessControllerCreateGlobalInput {
    fn clone(&self) -> Self {
        Self {
            controlled_asset: Bucket(self.controlled_asset.0),
            rule_set: self.rule_set.clone(),
            timed_recovery_delay_in_minutes: self.timed_recovery_delay_in_minutes.clone(),
        }
    }
}

//================================
// Access Controller Create Proof
//================================

pub const ACCESS_CONTROLLER_CREATE_PROOF_IDENT: &str = "create_proof";

#[derive(
    Debug, Clone, Eq, PartialEq, ScryptoSbor, ManifestCategorize, ManifestEncode, ManifestDecode,
)]
pub struct AccessControllerCreateProofInput {}

//================================================
// Access Controller Initiate Recovery As Primary
//================================================

pub const ACCESS_CONTROLLER_INITIATE_RECOVERY_AS_PRIMARY_IDENT: &str =
    "initiate_recovery_as_primary";

#[derive(
    Debug, Clone, Eq, PartialEq, ScryptoSbor, ManifestCategorize, ManifestEncode, ManifestDecode,
)]
pub struct AccessControllerInitiateRecoveryAsPrimaryInput {
    pub rule_set: RuleSet,
    pub timed_recovery_delay_in_minutes: Option<u32>,
}

//=================================================
// Access Controller Initiate Recovery As Recovery
//=================================================

pub const ACCESS_CONTROLLER_INITIATE_RECOVERY_AS_RECOVERY_IDENT: &str =
    "initiate_recovery_as_recovery";

#[derive(
    Debug, Clone, Eq, PartialEq, ScryptoSbor, ManifestCategorize, ManifestEncode, ManifestDecode,
)]
pub struct AccessControllerInitiateRecoveryAsRecoveryInput {
    pub rule_set: RuleSet,
    pub timed_recovery_delay_in_minutes: Option<u32>,
}

//=======================================================
// Access Controller Quick Confirm Primary Role Recovery
//=======================================================

pub const ACCESS_CONTROLLER_QUICK_CONFIRM_PRIMARY_ROLE_RECOVERY_PROPOSAL_IDENT: &str =
    "quick_confirm_primary_role_recovery_proposal";

#[derive(
    Debug, Clone, Eq, PartialEq, ScryptoSbor, ManifestCategorize, ManifestEncode, ManifestDecode,
)]
pub struct AccessControllerQuickConfirmPrimaryRoleRecoveryProposalInput {
    pub rule_set: RuleSet,
    pub timed_recovery_delay_in_minutes: Option<u32>,
}

//========================================================
// Access Controller Quick Confirm Recovery Role Recovery
//========================================================

pub const ACCESS_CONTROLLER_QUICK_CONFIRM_RECOVERY_ROLE_RECOVERY_PROPOSAL_IDENT: &str =
    "quick_confirm_recovery_role_recovery_proposal";

#[derive(
    Debug, Clone, Eq, PartialEq, ScryptoSbor, ManifestCategorize, ManifestEncode, ManifestDecode,
)]
pub struct AccessControllerQuickConfirmRecoveryRoleRecoveryProposalInput {
    pub rule_set: RuleSet,
    pub timed_recovery_delay_in_minutes: Option<u32>,
}

//=================================
// Access Controller Timed Confirm
//=================================

pub const ACCESS_CONTROLLER_TIMED_CONFIRM_RECOVERY_IDENT: &str = "timed_confirm_recovery";

#[derive(
    Debug, Clone, Eq, PartialEq, ScryptoSbor, ManifestCategorize, ManifestEncode, ManifestDecode,
)]
pub struct AccessControllerTimedConfirmRecoveryInput {
    pub rule_set: RuleSet,
    pub timed_recovery_delay_in_minutes: Option<u32>,
}

//=========================================================
// Access Controller Cancel Primary Role Recovery Proposal
//=========================================================

pub const ACCESS_CONTROLLER_CANCEL_PRIMARY_ROLE_RECOVERY_PROPOSAL_IDENT: &str =
    "cancel_primary_role_recovery_proposal";

#[derive(
    Debug, Clone, Eq, PartialEq, ScryptoSbor, ManifestCategorize, ManifestEncode, ManifestDecode,
)]
pub struct AccessControllerCancelPrimaryRoleRecoveryProposalInput;

//==========================================================
// Access Controller Cancel Recovery Role Recovery Proposal
//==========================================================

pub const ACCESS_CONTROLLER_CANCEL_RECOVERY_ROLE_RECOVERY_PROPOSAL_IDENT: &str =
    "cancel_recovery_role_recovery_proposal";

#[derive(
    Debug, Clone, Eq, PartialEq, ScryptoSbor, ManifestCategorize, ManifestEncode, ManifestDecode,
)]
pub struct AccessControllerCancelRecoveryRoleRecoveryProposalInput;

//=====================================
// Access Controller Lock Primary Role
//=====================================

pub const ACCESS_CONTROLLER_LOCK_PRIMARY_ROLE: &str = "lock_primary_role";

#[derive(
    Debug, Clone, Eq, PartialEq, ScryptoSbor, ManifestCategorize, ManifestEncode, ManifestDecode,
)]
pub struct AccessControllerLockPrimaryRoleInput;

//=======================================
// Access Controller Unlock Primary Role
//=======================================

pub const ACCESS_CONTROLLER_UNLOCK_PRIMARY_ROLE: &str = "unlock_primary_role";

#[derive(
    Debug, Clone, Eq, PartialEq, ScryptoSbor, ManifestCategorize, ManifestEncode, ManifestDecode,
)]
pub struct AccessControllerUnlockPrimaryRoleInput;

//=======================================
// Access Controller Stop Timed Recovery
//=======================================

pub const ACCESS_CONTROLLER_STOP_TIMED_RECOVERY: &str = "stop_timed_recovery";

#[derive(
    Debug, Clone, Eq, PartialEq, ScryptoSbor, ManifestCategorize, ManifestEncode, ManifestDecode,
)]
pub struct AccessControllerStopTimedRecoveryInput {
    pub rule_set: RuleSet,
    pub timed_recovery_delay_in_minutes: Option<u32>,
}
