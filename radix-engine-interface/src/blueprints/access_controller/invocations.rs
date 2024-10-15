use crate::blueprints::access_controller::*;
use crate::blueprints::component::*;
use crate::blueprints::resource::*;
use radix_common::data::scrypto::model::NonFungibleLocalId;
use radix_common::prelude::*;
use radix_common::types::GlobalAddressReservation;
use radix_rust::rust::prelude::IndexSet;
use sbor::rust::fmt::Debug;

pub const ACCESS_CONTROLLER_BLUEPRINT: &str = "AccessController";

define_type_marker!(Some(ACCESS_CONTROLLER_PACKAGE), AccessController);

//=================================
// Access Controller Create Global
//=================================

pub const ACCESS_CONTROLLER_CREATE_IDENT: &str = "create";

#[derive(Debug, Eq, PartialEq, ScryptoSbor)]
pub struct AccessControllerCreateInput {
    pub controlled_asset: Bucket,
    pub rule_set: RuleSet,
    pub timed_recovery_delay_in_minutes: Option<u32>,
    pub address_reservation: Option<GlobalAddressReservation>,
}

#[derive(Debug, Eq, PartialEq, ManifestSbor)]
pub struct AccessControllerCreateManifestInput {
    pub controlled_asset: ManifestBucket,
    pub rule_set: RuleSet,
    pub timed_recovery_delay_in_minutes: Option<u32>,
    pub address_reservation: Option<ManifestAddressReservation>,
}

pub type AccessControllerCreateOutput = Global<AccessControllerMarker>;

//================================
// Access Controller Create Proof
//================================

pub const ACCESS_CONTROLLER_CREATE_PROOF_IDENT: &str = "create_proof";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor, ManifestSbor)]
pub struct AccessControllerCreateProofInput {}

pub type AccessControllerCreateProofManifestInput = AccessControllerCreateProofInput;

pub type AccessControllerCreateProofOutput = Proof;

//================================================
// Access Controller Initiate Recovery As Primary
//================================================

pub const ACCESS_CONTROLLER_INITIATE_RECOVERY_AS_PRIMARY_IDENT: &str =
    "initiate_recovery_as_primary";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor, ManifestSbor)]
pub struct AccessControllerInitiateRecoveryAsPrimaryInput {
    pub rule_set: RuleSet,
    pub timed_recovery_delay_in_minutes: Option<u32>,
}

pub type AccessControllerInitiateRecoveryAsPrimaryManifestInput =
    AccessControllerInitiateRecoveryAsPrimaryInput;

pub type AccessControllerInitiateRecoveryAsPrimaryOutput = ();

//=================================================
// Access Controller Initiate Recovery As Recovery
//=================================================

pub const ACCESS_CONTROLLER_INITIATE_RECOVERY_AS_RECOVERY_IDENT: &str =
    "initiate_recovery_as_recovery";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor, ManifestSbor)]
pub struct AccessControllerInitiateRecoveryAsRecoveryInput {
    pub rule_set: RuleSet,
    pub timed_recovery_delay_in_minutes: Option<u32>,
}

pub type AccessControllerInitiateRecoveryAsRecoveryManifestInput =
    AccessControllerInitiateRecoveryAsRecoveryInput;

pub type AccessControllerInitiateRecoveryAsRecoveryOutput = ();

//==============================================================
// Access Controller Initiate Badge Withdraw Attempt As Primary
//==============================================================

pub const ACCESS_CONTROLLER_INITIATE_BADGE_WITHDRAW_ATTEMPT_AS_PRIMARY_IDENT: &str =
    "initiate_badge_withdraw_attempt_as_primary";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor, ManifestSbor)]
pub struct AccessControllerInitiateBadgeWithdrawAttemptAsPrimaryInput;

pub type AccessControllerInitiateBadgeWithdrawAttemptAsPrimaryManifestInput =
    AccessControllerInitiateBadgeWithdrawAttemptAsPrimaryInput;

pub type AccessControllerInitiateBadgeWithdrawAttemptAsPrimaryOutput = ();

//===============================================================
// Access Controller Initiate Badge Withdraw Attempt As Recovery
//===============================================================

pub const ACCESS_CONTROLLER_INITIATE_BADGE_WITHDRAW_ATTEMPT_AS_RECOVERY_IDENT: &str =
    "initiate_badge_withdraw_attempt_as_recovery";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor, ManifestSbor)]
pub struct AccessControllerInitiateBadgeWithdrawAttemptAsRecoveryInput;

pub type AccessControllerInitiateBadgeWithdrawAttemptAsRecoveryManifestInput =
    AccessControllerInitiateBadgeWithdrawAttemptAsRecoveryInput;

pub type AccessControllerInitiateBadgeWithdrawAttemptAsRecoveryOutput = ();

//=======================================================
// Access Controller Quick Confirm Primary Role Recovery
//=======================================================

pub const ACCESS_CONTROLLER_QUICK_CONFIRM_PRIMARY_ROLE_RECOVERY_PROPOSAL_IDENT: &str =
    "quick_confirm_primary_role_recovery_proposal";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor, ManifestSbor)]
pub struct AccessControllerQuickConfirmPrimaryRoleRecoveryProposalInput {
    pub rule_set: RuleSet,
    pub timed_recovery_delay_in_minutes: Option<u32>,
}

pub type AccessControllerQuickConfirmPrimaryRoleRecoveryProposalManifestInput =
    AccessControllerQuickConfirmPrimaryRoleRecoveryProposalInput;

pub type AccessControllerQuickConfirmPrimaryRoleRecoveryProposalOutput = ();

//========================================================
// Access Controller Quick Confirm Recovery Role Recovery
//========================================================

pub const ACCESS_CONTROLLER_QUICK_CONFIRM_RECOVERY_ROLE_RECOVERY_PROPOSAL_IDENT: &str =
    "quick_confirm_recovery_role_recovery_proposal";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor, ManifestSbor)]
pub struct AccessControllerQuickConfirmRecoveryRoleRecoveryProposalInput {
    pub rule_set: RuleSet,
    pub timed_recovery_delay_in_minutes: Option<u32>,
}

pub type AccessControllerQuickConfirmRecoveryRoleRecoveryProposalManifestInput =
    AccessControllerQuickConfirmRecoveryRoleRecoveryProposalInput;

pub type AccessControllerQuickConfirmRecoveryRoleRecoveryProposalOutput = ();

//=====================================================================
// Access Controller Quick Confirm Primary Role Badge Withdraw Attempt
//=====================================================================

pub const ACCESS_CONTROLLER_QUICK_CONFIRM_PRIMARY_ROLE_BADGE_WITHDRAW_ATTEMPT_IDENT: &str =
    "quick_confirm_primary_role_badge_withdraw_attempt";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor, ManifestSbor)]
pub struct AccessControllerQuickConfirmPrimaryRoleBadgeWithdrawAttemptInput;

pub type AccessControllerQuickConfirmPrimaryRoleBadgeWithdrawAttemptManifestInput =
    AccessControllerQuickConfirmPrimaryRoleBadgeWithdrawAttemptInput;

pub type AccessControllerQuickConfirmPrimaryRoleBadgeWithdrawAttemptOutput = Bucket;

//======================================================================
// Access Controller Quick Confirm Recovery Role Badge Withdraw Attempt
//======================================================================

pub const ACCESS_CONTROLLER_QUICK_CONFIRM_RECOVERY_ROLE_BADGE_WITHDRAW_ATTEMPT_IDENT: &str =
    "quick_confirm_recovery_role_badge_withdraw_attempt";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor, ManifestSbor)]
pub struct AccessControllerQuickConfirmRecoveryRoleBadgeWithdrawAttemptInput;

pub type AccessControllerQuickConfirmRecoveryRoleBadgeWithdrawAttemptManifestInput =
    AccessControllerQuickConfirmRecoveryRoleBadgeWithdrawAttemptInput;

pub type AccessControllerQuickConfirmRecoveryRoleBadgeWithdrawAttemptOutput = Bucket;

//=========================================
// Access Controller Timed Confirm Recovery
//=========================================

pub const ACCESS_CONTROLLER_TIMED_CONFIRM_RECOVERY_IDENT: &str = "timed_confirm_recovery";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor, ManifestSbor)]
pub struct AccessControllerTimedConfirmRecoveryInput {
    pub rule_set: RuleSet,
    pub timed_recovery_delay_in_minutes: Option<u32>,
}

pub type AccessControllerTimedConfirmRecoveryManifestInput =
    AccessControllerTimedConfirmRecoveryInput;

pub type AccessControllerTimedConfirmRecoveryOutput = ();

//=========================================================
// Access Controller Cancel Primary Role Recovery Proposal
//=========================================================

pub const ACCESS_CONTROLLER_CANCEL_PRIMARY_ROLE_RECOVERY_PROPOSAL_IDENT: &str =
    "cancel_primary_role_recovery_proposal";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor, ManifestSbor)]
pub struct AccessControllerCancelPrimaryRoleRecoveryProposalInput;

pub type AccessControllerCancelPrimaryRoleRecoveryProposalManifestInput =
    AccessControllerCancelPrimaryRoleRecoveryProposalInput;

pub type AccessControllerCancelPrimaryRoleRecoveryProposalOutput = ();

//==========================================================
// Access Controller Cancel Recovery Role Recovery Proposal
//==========================================================

pub const ACCESS_CONTROLLER_CANCEL_RECOVERY_ROLE_RECOVERY_PROPOSAL_IDENT: &str =
    "cancel_recovery_role_recovery_proposal";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor, ManifestSbor)]
pub struct AccessControllerCancelRecoveryRoleRecoveryProposalInput;

pub type AccessControllerCancelRecoveryRoleRecoveryProposalManifestInput =
    AccessControllerCancelRecoveryRoleRecoveryProposalInput;

pub type AccessControllerCancelRecoveryRoleRecoveryProposalOutput = ();

//==============================================================
// Access Controller Cancel Primary Role Badge Withdraw Attempt
//==============================================================

pub const ACCESS_CONTROLLER_CANCEL_PRIMARY_ROLE_BADGE_WITHDRAW_ATTEMPT_IDENT: &str =
    "cancel_primary_role_badge_withdraw_attempt";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor, ManifestSbor)]
pub struct AccessControllerCancelPrimaryRoleBadgeWithdrawAttemptInput;

pub type AccessControllerCancelPrimaryRoleBadgeWithdrawAttemptManifestInput =
    AccessControllerCancelPrimaryRoleBadgeWithdrawAttemptInput;

pub type AccessControllerCancelPrimaryRoleBadgeWithdrawAttemptOutput = ();

//===============================================================
// Access Controller Cancel Recovery Role Badge Withdraw Attempt
//===============================================================

pub const ACCESS_CONTROLLER_CANCEL_RECOVERY_ROLE_BADGE_WITHDRAW_ATTEMPT_IDENT: &str =
    "cancel_recovery_role_badge_withdraw_attempt";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor, ManifestSbor)]
pub struct AccessControllerCancelRecoveryRoleBadgeWithdrawAttemptInput;

pub type AccessControllerCancelRecoveryRoleBadgeWithdrawAttemptManifestInput =
    AccessControllerCancelRecoveryRoleBadgeWithdrawAttemptInput;

pub type AccessControllerCancelRecoveryRoleBadgeWithdrawAttemptOutput = ();

//=====================================
// Access Controller Lock Primary Role
//=====================================

pub const ACCESS_CONTROLLER_LOCK_PRIMARY_ROLE_IDENT: &str = "lock_primary_role";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor, ManifestSbor)]
pub struct AccessControllerLockPrimaryRoleInput;

pub type AccessControllerLockPrimaryRoleManifestInput = AccessControllerLockPrimaryRoleInput;

pub type AccessControllerLockPrimaryRoleOutput = ();

//=======================================
// Access Controller Unlock Primary Role
//=======================================

pub const ACCESS_CONTROLLER_UNLOCK_PRIMARY_ROLE_IDENT: &str = "unlock_primary_role";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor, ManifestSbor)]
pub struct AccessControllerUnlockPrimaryRoleInput;

pub type AccessControllerUnlockPrimaryRoleManifestInput = AccessControllerUnlockPrimaryRoleInput;

pub type AccessControllerUnlockPrimaryRoleOutput = ();

//=======================================
// Access Controller Stop Timed Recovery
//=======================================

pub const ACCESS_CONTROLLER_STOP_TIMED_RECOVERY_IDENT: &str = "stop_timed_recovery";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor, ManifestSbor)]
pub struct AccessControllerStopTimedRecoveryInput {
    pub rule_set: RuleSet,
    pub timed_recovery_delay_in_minutes: Option<u32>,
}

pub type AccessControllerStopTimedRecoveryManifestInput = AccessControllerStopTimedRecoveryInput;

pub type AccessControllerStopTimedRecoveryOutput = ();

//========================================
// Access Controller Mint Recovery Badges
//========================================

pub const ACCESS_CONTROLLER_MINT_RECOVERY_BADGES_IDENT: &str = "mint_recovery_badges";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor, ManifestSbor)]
pub struct AccessControllerMintRecoveryBadgesInput {
    pub non_fungible_local_ids: IndexSet<NonFungibleLocalId>,
}

pub type AccessControllerMintRecoveryBadgesManifestInput = AccessControllerMintRecoveryBadgesInput;

pub type AccessControllerMintRecoveryBadgesOutput = Bucket;

// region:bottlenose

//==================
// Lock Recovery Fee
//==================

pub const ACCESS_CONTROLLER_LOCK_RECOVERY_FEE_IDENT: &str = "lock_recovery_fee";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor, ManifestSbor)]
pub struct AccessControllerLockRecoveryFeeInput {
    pub amount: Decimal,
}

pub type AccessControllerLockRecoveryFeeManifestInput = AccessControllerLockRecoveryFeeInput;

pub type AccessControllerLockRecoveryFeeOutput = ();

//========================
// Withdraw Recovery Fee
//=======================

pub const ACCESS_CONTROLLER_WITHDRAW_RECOVERY_FEE_IDENT: &str = "withdraw_recovery_fee";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor, ManifestSbor)]
pub struct AccessControllerWithdrawRecoveryFeeInput {
    pub amount: Decimal,
}

pub type AccessControllerWithdrawRecoveryFeeManifestInput =
    AccessControllerWithdrawRecoveryFeeInput;

pub type AccessControllerWithdrawRecoveryFeeOutput = Bucket;

//=========================
// Contribute Recovery Fee
//=========================

pub const ACCESS_CONTROLLER_CONTRIBUTE_RECOVERY_FEE_IDENT: &str = "contribute_recovery_fee";

#[derive(Debug, Eq, PartialEq, ScryptoSbor)]
pub struct AccessControllerContributeRecoveryFeeInput {
    pub bucket: Bucket,
}

#[derive(Debug, Clone, Eq, PartialEq, ManifestSbor)]
pub struct AccessControllerContributeRecoveryFeeManifestInput {
    pub bucket: ManifestBucket,
}

pub type AccessControllerContributeRecoveryFeeOutput = ();

// endregion:bottlenose
