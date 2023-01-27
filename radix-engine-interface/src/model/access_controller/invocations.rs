use sbor::rust::fmt::Debug;

use crate::api::types::BucketId;
use crate::api::wasm::*;
use crate::api::*;
use crate::model::*;
use crate::*;

//=================================
// Access Controller Create Global
//=================================

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct AccessControllerCreateGlobalInvocation {
    pub controlled_asset: BucketId,
    pub rule_set: RuleSet,
    pub timed_recovery_delay_in_minutes: Option<u32>,
}

impl Invocation for AccessControllerCreateGlobalInvocation {
    type Output = ComponentAddress;
}

impl SerializableInvocation for AccessControllerCreateGlobalInvocation {
    type ScryptoOutput = ComponentAddress;
}

impl Into<CallTableInvocation> for AccessControllerCreateGlobalInvocation {
    fn into(self) -> CallTableInvocation {
        NativeInvocation::AccessController(AccessControllerInvocation::CreateGlobal(self)).into()
    }
}

//================================
// Access Controller Create Proof
//================================

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct AccessControllerCreateProofMethodArgs;

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct AccessControllerCreateProofInvocation {
    pub receiver: ComponentAddress,
}

impl Invocation for AccessControllerCreateProofInvocation {
    type Output = Proof;
}

impl SerializableInvocation for AccessControllerCreateProofInvocation {
    type ScryptoOutput = Proof;
}

impl Into<CallTableInvocation> for AccessControllerCreateProofInvocation {
    fn into(self) -> CallTableInvocation {
        NativeInvocation::AccessController(AccessControllerInvocation::CreateProof(self)).into()
    }
}

//================================================
// Access Controller Initiate Recovery As Primary
//================================================

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct AccessControllerInitiateRecoveryAsPrimaryMethodArgs {
    pub rule_set: RuleSet,
    pub timed_recovery_delay_in_minutes: Option<u32>,
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct AccessControllerInitiateRecoveryAsPrimaryInvocation {
    pub receiver: ComponentAddress,
    pub proposal: RecoveryProposal,
}

impl Invocation for AccessControllerInitiateRecoveryAsPrimaryInvocation {
    type Output = ();
}

impl SerializableInvocation for AccessControllerInitiateRecoveryAsPrimaryInvocation {
    type ScryptoOutput = ();
}

impl Into<CallTableInvocation> for AccessControllerInitiateRecoveryAsPrimaryInvocation {
    fn into(self) -> CallTableInvocation {
        NativeInvocation::AccessController(AccessControllerInvocation::InitiateRecoveryAsPrimary(
            self,
        ))
        .into()
    }
}

//=================================================
// Access Controller Initiate Recovery As Recovery
//=================================================

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct AccessControllerInitiateRecoveryAsRecoveryMethodArgs {
    pub rule_set: RuleSet,
    pub timed_recovery_delay_in_minutes: Option<u32>,
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct AccessControllerInitiateRecoveryAsRecoveryInvocation {
    pub receiver: ComponentAddress,
    pub proposal: RecoveryProposal,
}

impl Invocation for AccessControllerInitiateRecoveryAsRecoveryInvocation {
    type Output = ();
}

impl SerializableInvocation for AccessControllerInitiateRecoveryAsRecoveryInvocation {
    type ScryptoOutput = ();
}

impl Into<CallTableInvocation> for AccessControllerInitiateRecoveryAsRecoveryInvocation {
    fn into(self) -> CallTableInvocation {
        NativeInvocation::AccessController(AccessControllerInvocation::InitiateRecoveryAsRecovery(
            self,
        ))
        .into()
    }
}

//=======================================================
// Access Controller Quick Confirm Primary Role Recovery
//=======================================================

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct AccessControllerQuickConfirmPrimaryRoleRecoveryProposalMethodArgs {
    pub rule_set: RuleSet,
    pub timed_recovery_delay_in_minutes: Option<u32>,
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct AccessControllerQuickConfirmPrimaryRoleRecoveryProposalInvocation {
    pub receiver: ComponentAddress,
    pub proposal_to_confirm: RecoveryProposal,
}

impl Invocation for AccessControllerQuickConfirmPrimaryRoleRecoveryProposalInvocation {
    type Output = ();
}

impl SerializableInvocation for AccessControllerQuickConfirmPrimaryRoleRecoveryProposalInvocation {
    type ScryptoOutput = ();
}

impl Into<CallTableInvocation>
    for AccessControllerQuickConfirmPrimaryRoleRecoveryProposalInvocation
{
    fn into(self) -> CallTableInvocation {
        NativeInvocation::AccessController(
            AccessControllerInvocation::QuickConfirmPrimaryRoleRecoveryProposal(self),
        )
        .into()
    }
}

//========================================================
// Access Controller Quick Confirm Recovery Role Recovery
//========================================================

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct AccessControllerQuickConfirmRecoveryRoleRecoveryProposalMethodArgs {
    pub rule_set: RuleSet,
    pub timed_recovery_delay_in_minutes: Option<u32>,
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct AccessControllerQuickConfirmRecoveryRoleRecoveryProposalInvocation {
    pub receiver: ComponentAddress,
    pub proposal_to_confirm: RecoveryProposal,
}

impl Invocation for AccessControllerQuickConfirmRecoveryRoleRecoveryProposalInvocation {
    type Output = ();
}

impl SerializableInvocation for AccessControllerQuickConfirmRecoveryRoleRecoveryProposalInvocation {
    type ScryptoOutput = ();
}

impl Into<CallTableInvocation>
    for AccessControllerQuickConfirmRecoveryRoleRecoveryProposalInvocation
{
    fn into(self) -> CallTableInvocation {
        NativeInvocation::AccessController(
            AccessControllerInvocation::QuickConfirmRecoveryRoleRecoveryProposal(self),
        )
        .into()
    }
}

//=================================
// Access Controller Timed Confirm
//=================================

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct AccessControllerTimedConfirmRecoveryMethodArgs {
    pub rule_set: RuleSet,
    pub timed_recovery_delay_in_minutes: Option<u32>,
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct AccessControllerTimedConfirmRecoveryInvocation {
    pub receiver: ComponentAddress,
    pub proposal_to_confirm: RecoveryProposal,
}

impl Invocation for AccessControllerTimedConfirmRecoveryInvocation {
    type Output = ();
}

impl SerializableInvocation for AccessControllerTimedConfirmRecoveryInvocation {
    type ScryptoOutput = ();
}

impl Into<CallTableInvocation> for AccessControllerTimedConfirmRecoveryInvocation {
    fn into(self) -> CallTableInvocation {
        NativeInvocation::AccessController(AccessControllerInvocation::TimedConfirmRecovery(self))
            .into()
    }
}

//=========================================================
// Access Controller Cancel Primary Role Recovery Proposal
//=========================================================

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct AccessControllerCancelPrimaryRoleRecoveryProposalMethodArgs;

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct AccessControllerCancelPrimaryRoleRecoveryProposalInvocation {
    pub receiver: ComponentAddress,
}

impl Invocation for AccessControllerCancelPrimaryRoleRecoveryProposalInvocation {
    type Output = ();
}

impl SerializableInvocation for AccessControllerCancelPrimaryRoleRecoveryProposalInvocation {
    type ScryptoOutput = ();
}

impl Into<CallTableInvocation> for AccessControllerCancelPrimaryRoleRecoveryProposalInvocation {
    fn into(self) -> CallTableInvocation {
        NativeInvocation::AccessController(
            AccessControllerInvocation::CancelPrimaryRoleRecoveryProposal(self),
        )
        .into()
    }
}

//==========================================================
// Access Controller Cancel Recovery Role Recovery Proposal
//==========================================================

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct AccessControllerCancelRecoveryRoleRecoveryProposalMethodArgs;

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct AccessControllerCancelRecoveryRoleRecoveryProposalInvocation {
    pub receiver: ComponentAddress,
}

impl Invocation for AccessControllerCancelRecoveryRoleRecoveryProposalInvocation {
    type Output = ();
}

impl SerializableInvocation for AccessControllerCancelRecoveryRoleRecoveryProposalInvocation {
    type ScryptoOutput = ();
}

impl Into<CallTableInvocation> for AccessControllerCancelRecoveryRoleRecoveryProposalInvocation {
    fn into(self) -> CallTableInvocation {
        NativeInvocation::AccessController(
            AccessControllerInvocation::CancelRecoveryRoleRecoveryProposal(self),
        )
        .into()
    }
}

//=====================================
// Access Controller Lock Primary Role
//=====================================

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct AccessControllerLockPrimaryRoleMethodArgs;

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct AccessControllerLockPrimaryRoleInvocation {
    pub receiver: ComponentAddress,
}

impl Invocation for AccessControllerLockPrimaryRoleInvocation {
    type Output = ();
}

impl SerializableInvocation for AccessControllerLockPrimaryRoleInvocation {
    type ScryptoOutput = ();
}

impl Into<CallTableInvocation> for AccessControllerLockPrimaryRoleInvocation {
    fn into(self) -> CallTableInvocation {
        NativeInvocation::AccessController(AccessControllerInvocation::LockPrimaryRole(self)).into()
    }
}

//=======================================
// Access Controller Unlock Primary Role
//=======================================

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct AccessControllerUnlockPrimaryRoleMethodArgs;

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct AccessControllerUnlockPrimaryRoleInvocation {
    pub receiver: ComponentAddress,
}

impl Invocation for AccessControllerUnlockPrimaryRoleInvocation {
    type Output = ();
}

impl SerializableInvocation for AccessControllerUnlockPrimaryRoleInvocation {
    type ScryptoOutput = ();
}

impl Into<CallTableInvocation> for AccessControllerUnlockPrimaryRoleInvocation {
    fn into(self) -> CallTableInvocation {
        NativeInvocation::AccessController(AccessControllerInvocation::UnlockPrimaryRole(self))
            .into()
    }
}

//=======================================
// Access Controller Stop Timed Recovery
//=======================================

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct AccessControllerStopTimedRecoveryMethodArgs {
    pub rule_set: RuleSet,
    pub timed_recovery_delay_in_minutes: Option<u32>,
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct AccessControllerStopTimedRecoveryInvocation {
    pub receiver: ComponentAddress,
    pub proposal: RecoveryProposal,
}

impl Invocation for AccessControllerStopTimedRecoveryInvocation {
    type Output = ();
}

impl SerializableInvocation for AccessControllerStopTimedRecoveryInvocation {
    type ScryptoOutput = ();
}

impl Into<CallTableInvocation> for AccessControllerStopTimedRecoveryInvocation {
    fn into(self) -> CallTableInvocation {
        NativeInvocation::AccessController(AccessControllerInvocation::StopTimedRecovery(self))
            .into()
    }
}
