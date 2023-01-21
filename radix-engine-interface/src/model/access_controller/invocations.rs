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

//=====================================
// Access Controller Initiate Recovery
//=====================================

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct AccessControllerInitiateRecoveryMethodArgs {
    pub rule_set: RuleSet,
    pub role: Role,
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct AccessControllerInitiateRecoveryExecutable {
    pub receiver: RENodeId,
    pub rule_set: RuleSet,
    pub role: Role,
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct AccessControllerInitiateRecoveryInvocation {
    pub receiver: ComponentAddress,
    pub rule_set: RuleSet,
    pub role: Role,
}

impl Invocation for AccessControllerInitiateRecoveryInvocation {
    type Output = ();
}

impl SerializableInvocation for AccessControllerInitiateRecoveryInvocation {
    type ScryptoOutput = ();
}

impl Into<CallTableInvocation> for AccessControllerInitiateRecoveryInvocation {
    fn into(self) -> CallTableInvocation {
        NativeInvocation::AccessController(AccessControllerInvocation::InitiateRecovery(self))
            .into()
    }
}

//=================================
// Access Controller Quick Confirm
//=================================

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct AccessControllerQuickConfirmRecoveryMethodArgs {
    pub rule_set: RuleSet,
    pub proposer: Role,
    pub role: Role,
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct AccessControllerQuickConfirmRecoveryExecutable {
    pub receiver: RENodeId,
    pub rule_set: RuleSet,
    pub proposer: Role,
    pub role: Role,
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct AccessControllerQuickConfirmRecoveryInvocation {
    pub receiver: ComponentAddress,
    pub rule_set: RuleSet,
    pub proposer: Role,
    pub role: Role,
}

impl Invocation for AccessControllerQuickConfirmRecoveryInvocation {
    type Output = ();
}

impl SerializableInvocation for AccessControllerQuickConfirmRecoveryInvocation {
    type ScryptoOutput = ();
}

impl Into<CallTableInvocation> for AccessControllerQuickConfirmRecoveryInvocation {
    fn into(self) -> CallTableInvocation {
        NativeInvocation::AccessController(AccessControllerInvocation::QuickConfirmRecovery(self))
            .into()
    }
}

//=================================
// Access Controller Timed Confirm
//=================================

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct AccessControllerTimedConfirmRecoveryMethodArgs {
    pub rule_set: RuleSet,
    pub role: Role,
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct AccessControllerTimedConfirmRecoveryExecutable {
    pub receiver: RENodeId,
    pub rule_set: RuleSet,
    pub role: Role,
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct AccessControllerTimedConfirmRecoveryInvocation {
    pub receiver: ComponentAddress,
    pub rule_set: RuleSet,
    pub role: Role,
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

//===========================================
// Access Controller Cancel Recovery Attempt
//===========================================

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct AccessControllerCancelRecoveryAttemptMethodArgs {
    pub rule_set: RuleSet,
    pub role: Role,
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct AccessControllerCancelRecoveryAttemptExecutable {
    pub receiver: RENodeId,
    pub rule_set: RuleSet,
    pub role: Role,
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct AccessControllerCancelRecoveryAttemptInvocation {
    pub receiver: ComponentAddress,
    pub rule_set: RuleSet,
    pub role: Role,
}

impl Invocation for AccessControllerCancelRecoveryAttemptInvocation {
    type Output = ();
}

impl SerializableInvocation for AccessControllerCancelRecoveryAttemptInvocation {
    type ScryptoOutput = ();
}

impl Into<CallTableInvocation> for AccessControllerCancelRecoveryAttemptInvocation {
    fn into(self) -> CallTableInvocation {
        NativeInvocation::AccessController(AccessControllerInvocation::CancelRecoveryAttempt(self))
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
