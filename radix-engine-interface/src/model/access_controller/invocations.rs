use sbor::rust::fmt::Debug;

use crate::api::types::BucketId;
use crate::api::types::RENodeId;
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
    pub timed_recovery_delay_in_hours: u16,
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
pub struct AccessControllerCreateProofMethodArgs {}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct AccessControllerCreateProofExecutable {
    pub receiver: RENodeId,
}

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

//=========================================
// Access Controller Update Timed Recovery
//=========================================

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct AccessControllerUpdateTimedRecoveryDelayMethodArgs {
    pub timed_recovery_delay_in_hours: u16,
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct AccessControllerUpdateTimedRecoveryDelayExecutable {
    pub receiver: RENodeId,
    pub timed_recovery_delay_in_hours: u16,
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct AccessControllerUpdateTimedRecoveryDelayInvocation {
    pub receiver: ComponentAddress,
    pub timed_recovery_delay_in_hours: u16,
}

impl Invocation for AccessControllerUpdateTimedRecoveryDelayInvocation {
    type Output = ();
}

impl SerializableInvocation for AccessControllerUpdateTimedRecoveryDelayInvocation {
    type ScryptoOutput = ();
}

impl Into<CallTableInvocation> for AccessControllerUpdateTimedRecoveryDelayInvocation {
    fn into(self) -> CallTableInvocation {
        NativeInvocation::AccessController(AccessControllerInvocation::UpdateTimedRecoveryDelay(
            self,
        ))
        .into()
    }
}

//================================================
// Access Controller Initiate Recovery As Primary
//================================================

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct AccessControllerInitiateRecoveryAsPrimaryMethodArgs {
    pub rule_set: RuleSet,
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct AccessControllerInitiateRecoveryAsPrimaryExecutable {
    pub receiver: RENodeId,
    pub rule_set: RuleSet,
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct AccessControllerInitiateRecoveryAsPrimaryInvocation {
    pub receiver: ComponentAddress,
    pub rule_set: RuleSet,
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
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct AccessControllerInitiateRecoveryAsRecoveryExecutable {
    pub receiver: RENodeId,
    pub rule_set: RuleSet,
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct AccessControllerInitiateRecoveryAsRecoveryInvocation {
    pub receiver: ComponentAddress,
    pub rule_set: RuleSet,
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

//=====================================================
// Access Controller Initiate Recovery As Confirmation
//=====================================================

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct AccessControllerInitiateRecoveryAsConfirmationMethodArgs {
    pub rule_set: RuleSet,
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct AccessControllerInitiateRecoveryAsConfirmationExecutable {
    pub receiver: RENodeId,
    pub rule_set: RuleSet,
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct AccessControllerInitiateRecoveryAsConfirmationInvocation {
    pub receiver: ComponentAddress,
    pub rule_set: RuleSet,
}

impl Invocation for AccessControllerInitiateRecoveryAsConfirmationInvocation {
    type Output = ();
}

impl SerializableInvocation for AccessControllerInitiateRecoveryAsConfirmationInvocation {
    type ScryptoOutput = ();
}

impl Into<CallTableInvocation> for AccessControllerInitiateRecoveryAsConfirmationInvocation {
    fn into(self) -> CallTableInvocation {
        NativeInvocation::AccessController(
            AccessControllerInvocation::InitiateRecoveryAsConfirmation(self),
        )
        .into()
    }
}

//============================================
// Access Controller Quick Confirm As Primary
//============================================

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct AccessControllerQuickConfirmRecoveryAsPrimaryMethodArgs {
    pub rule_set: RuleSet,
    pub proposer: Role,
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct AccessControllerQuickConfirmRecoveryAsPrimaryExecutable {
    pub receiver: RENodeId,
    pub rule_set: RuleSet,
    pub proposer: Role,
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct AccessControllerQuickConfirmRecoveryAsPrimaryInvocation {
    pub receiver: ComponentAddress,
    pub rule_set: RuleSet,
    pub proposer: Role,
}

impl Invocation for AccessControllerQuickConfirmRecoveryAsPrimaryInvocation {
    type Output = ();
}

impl SerializableInvocation for AccessControllerQuickConfirmRecoveryAsPrimaryInvocation {
    type ScryptoOutput = ();
}

impl Into<CallTableInvocation> for AccessControllerQuickConfirmRecoveryAsPrimaryInvocation {
    fn into(self) -> CallTableInvocation {
        NativeInvocation::AccessController(
            AccessControllerInvocation::QuickConfirmRecoveryAsPrimary(self),
        )
        .into()
    }
}

//=============================================
// Access Controller Quick Confirm As Recovery
//=============================================

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct AccessControllerQuickConfirmRecoveryAsRecoveryMethodArgs {
    pub rule_set: RuleSet,
    pub proposer: Role,
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct AccessControllerQuickConfirmRecoveryAsRecoveryExecutable {
    pub receiver: RENodeId,
    pub rule_set: RuleSet,
    pub proposer: Role,
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct AccessControllerQuickConfirmRecoveryAsRecoveryInvocation {
    pub receiver: ComponentAddress,
    pub rule_set: RuleSet,
    pub proposer: Role,
}

impl Invocation for AccessControllerQuickConfirmRecoveryAsRecoveryInvocation {
    type Output = ();
}

impl SerializableInvocation for AccessControllerQuickConfirmRecoveryAsRecoveryInvocation {
    type ScryptoOutput = ();
}

impl Into<CallTableInvocation> for AccessControllerQuickConfirmRecoveryAsRecoveryInvocation {
    fn into(self) -> CallTableInvocation {
        NativeInvocation::AccessController(
            AccessControllerInvocation::QuickConfirmRecoveryAsRecovery(self),
        )
        .into()
    }
}

//=================================================
// Access Controller Quick Confirm As Confirmation
//=================================================

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct AccessControllerQuickConfirmRecoveryAsConfirmationMethodArgs {
    pub rule_set: RuleSet,
    pub proposer: Role,
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct AccessControllerQuickConfirmRecoveryAsConfirmationExecutable {
    pub receiver: RENodeId,
    pub rule_set: RuleSet,
    pub proposer: Role,
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct AccessControllerQuickConfirmRecoveryAsConfirmationInvocation {
    pub receiver: ComponentAddress,
    pub rule_set: RuleSet,
    pub proposer: Role,
}

impl Invocation for AccessControllerQuickConfirmRecoveryAsConfirmationInvocation {
    type Output = ();
}

impl SerializableInvocation for AccessControllerQuickConfirmRecoveryAsConfirmationInvocation {
    type ScryptoOutput = ();
}

impl Into<CallTableInvocation> for AccessControllerQuickConfirmRecoveryAsConfirmationInvocation {
    fn into(self) -> CallTableInvocation {
        NativeInvocation::AccessController(
            AccessControllerInvocation::QuickConfirmRecoveryAsConfirmation(self),
        )
        .into()
    }
}

//============================================
// Access Controller Timed Confirm As Primary
//============================================

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct AccessControllerTimedConfirmRecoveryAsPrimaryMethodArgs {
    pub rule_set: RuleSet,
    pub proposer: Role,
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct AccessControllerTimedConfirmRecoveryAsPrimaryExecutable {
    pub receiver: RENodeId,
    pub rule_set: RuleSet,
    pub proposer: Role,
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct AccessControllerTimedConfirmRecoveryAsPrimaryInvocation {
    pub receiver: ComponentAddress,
    pub rule_set: RuleSet,
    pub proposer: Role,
}

impl Invocation for AccessControllerTimedConfirmRecoveryAsPrimaryInvocation {
    type Output = ();
}

impl SerializableInvocation for AccessControllerTimedConfirmRecoveryAsPrimaryInvocation {
    type ScryptoOutput = ();
}

impl Into<CallTableInvocation> for AccessControllerTimedConfirmRecoveryAsPrimaryInvocation {
    fn into(self) -> CallTableInvocation {
        NativeInvocation::AccessController(
            AccessControllerInvocation::TimedConfirmRecoveryAsPrimary(self),
        )
        .into()
    }
}

//=============================================
// Access Controller Timed Confirm As Recovery
//=============================================

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct AccessControllerTimedConfirmRecoveryAsRecoveryMethodArgs {
    pub rule_set: RuleSet,
    pub proposer: Role,
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct AccessControllerTimedConfirmRecoveryAsRecoveryExecutable {
    pub receiver: RENodeId,
    pub rule_set: RuleSet,
    pub proposer: Role,
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct AccessControllerTimedConfirmRecoveryAsRecoveryInvocation {
    pub receiver: ComponentAddress,
    pub rule_set: RuleSet,
    pub proposer: Role,
}

impl Invocation for AccessControllerTimedConfirmRecoveryAsRecoveryInvocation {
    type Output = ();
}

impl SerializableInvocation for AccessControllerTimedConfirmRecoveryAsRecoveryInvocation {
    type ScryptoOutput = ();
}

impl Into<CallTableInvocation> for AccessControllerTimedConfirmRecoveryAsRecoveryInvocation {
    fn into(self) -> CallTableInvocation {
        NativeInvocation::AccessController(
            AccessControllerInvocation::TimedConfirmRecoveryAsRecovery(self),
        )
        .into()
    }
}

//=================================================
// Access Controller Timed Confirm As Confirmation
//=================================================

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct AccessControllerTimedConfirmRecoveryAsConfirmationMethodArgs {
    pub rule_set: RuleSet,
    pub proposer: Role,
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct AccessControllerTimedConfirmRecoveryAsConfirmationExecutable {
    pub receiver: RENodeId,
    pub rule_set: RuleSet,
    pub proposer: Role,
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct AccessControllerTimedConfirmRecoveryAsConfirmationInvocation {
    pub receiver: ComponentAddress,
    pub rule_set: RuleSet,
    pub proposer: Role,
}

impl Invocation for AccessControllerTimedConfirmRecoveryAsConfirmationInvocation {
    type Output = ();
}

impl SerializableInvocation for AccessControllerTimedConfirmRecoveryAsConfirmationInvocation {
    type ScryptoOutput = ();
}

impl Into<CallTableInvocation> for AccessControllerTimedConfirmRecoveryAsConfirmationInvocation {
    fn into(self) -> CallTableInvocation {
        NativeInvocation::AccessController(
            AccessControllerInvocation::TimedConfirmRecoveryAsConfirmation(self),
        )
        .into()
    }
}

//======================================================
// Access Controller Cancel Recovery Attempt As Primary
//======================================================

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct AccessControllerCancelRecoveryAttemptAsPrimaryMethodArgs {
    pub rule_set: RuleSet,
    pub proposer: Role,
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct AccessControllerCancelRecoveryAttemptAsPrimaryExecutable {
    pub receiver: RENodeId,
    pub rule_set: RuleSet,
    pub proposer: Role,
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct AccessControllerCancelRecoveryAttemptAsPrimaryInvocation {
    pub receiver: ComponentAddress,
    pub rule_set: RuleSet,
    pub proposer: Role,
}

impl Invocation for AccessControllerCancelRecoveryAttemptAsPrimaryInvocation {
    type Output = ();
}

impl SerializableInvocation for AccessControllerCancelRecoveryAttemptAsPrimaryInvocation {
    type ScryptoOutput = ();
}

impl Into<CallTableInvocation> for AccessControllerCancelRecoveryAttemptAsPrimaryInvocation {
    fn into(self) -> CallTableInvocation {
        NativeInvocation::AccessController(
            AccessControllerInvocation::CancelRecoveryAttemptAsPrimary(self),
        )
        .into()
    }
}

//=======================================================
// Access Controller Cancel Recovery Attempt As Recovery
//=======================================================

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct AccessControllerCancelRecoveryAttemptAsRecoveryMethodArgs {
    pub rule_set: RuleSet,
    pub proposer: Role,
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct AccessControllerCancelRecoveryAttemptAsRecoveryExecutable {
    pub receiver: RENodeId,
    pub rule_set: RuleSet,
    pub proposer: Role,
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct AccessControllerCancelRecoveryAttemptAsRecoveryInvocation {
    pub receiver: ComponentAddress,
    pub rule_set: RuleSet,
    pub proposer: Role,
}

impl Invocation for AccessControllerCancelRecoveryAttemptAsRecoveryInvocation {
    type Output = ();
}

impl SerializableInvocation for AccessControllerCancelRecoveryAttemptAsRecoveryInvocation {
    type ScryptoOutput = ();
}

impl Into<CallTableInvocation> for AccessControllerCancelRecoveryAttemptAsRecoveryInvocation {
    fn into(self) -> CallTableInvocation {
        NativeInvocation::AccessController(
            AccessControllerInvocation::CancelRecoveryAttemptAsRecovery(self),
        )
        .into()
    }
}

//===========================================================
// Access Controller Cancel Recovery Attempt As Confirmation
//===========================================================

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct AccessControllerCancelRecoveryAttemptAsConfirmationMethodArgs {
    pub rule_set: RuleSet,
    pub proposer: Role,
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct AccessControllerCancelRecoveryAttemptAsConfirmationExecutable {
    pub receiver: RENodeId,
    pub rule_set: RuleSet,
    pub proposer: Role,
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct AccessControllerCancelRecoveryAttemptAsConfirmationInvocation {
    pub receiver: ComponentAddress,
    pub rule_set: RuleSet,
    pub proposer: Role,
}

impl Invocation for AccessControllerCancelRecoveryAttemptAsConfirmationInvocation {
    type Output = ();
}

impl SerializableInvocation for AccessControllerCancelRecoveryAttemptAsConfirmationInvocation {
    type ScryptoOutput = ();
}

impl Into<CallTableInvocation> for AccessControllerCancelRecoveryAttemptAsConfirmationInvocation {
    fn into(self) -> CallTableInvocation {
        NativeInvocation::AccessController(
            AccessControllerInvocation::CancelRecoveryAttemptAsConfirmation(self),
        )
        .into()
    }
}

//=====================================
// Access Controller Lock Primary Role
//=====================================

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct AccessControllerLockPrimaryRoleMethodArgs {}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct AccessControllerLockPrimaryRoleExecutable {
    pub receiver: RENodeId,
}

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
pub struct AccessControllerUnlockPrimaryRoleMethodArgs {}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct AccessControllerUnlockPrimaryRoleExecutable {
    pub receiver: RENodeId,
}

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
