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
pub struct AccessControllerCreateProofMethodArgs {}

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
    pub rule_set: RuleSet,
    pub timed_recovery_delay_in_minutes: Option<u32>,
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
    pub rule_set: RuleSet,
    pub timed_recovery_delay_in_minutes: Option<u32>,
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

//============================================
// Access Controller Quick Confirm As Primary
//============================================

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct AccessControllerQuickConfirmRecoveryAsPrimaryMethodArgs {
    pub rule_set: RuleSet,
    pub proposer: Proposer,
    pub timed_recovery_delay_in_minutes: Option<u32>,
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct AccessControllerQuickConfirmRecoveryAsPrimaryInvocation {
    pub receiver: ComponentAddress,
    pub rule_set: RuleSet,
    pub proposer: Proposer,
    pub timed_recovery_delay_in_minutes: Option<u32>,
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
    pub proposer: Proposer,
    pub timed_recovery_delay_in_minutes: Option<u32>,
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct AccessControllerQuickConfirmRecoveryAsRecoveryInvocation {
    pub receiver: ComponentAddress,
    pub rule_set: RuleSet,
    pub proposer: Proposer,
    pub timed_recovery_delay_in_minutes: Option<u32>,
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
    pub proposer: Proposer,
    pub timed_recovery_delay_in_minutes: Option<u32>,
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct AccessControllerQuickConfirmRecoveryAsConfirmationInvocation {
    pub receiver: ComponentAddress,
    pub rule_set: RuleSet,
    pub proposer: Proposer,
    pub timed_recovery_delay_in_minutes: Option<u32>,
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

//=================================
// Access Controller Timed Confirm As Primary
//=================================

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct AccessControllerTimedConfirmRecoveryAsPrimaryMethodArgs {
    pub rule_set: RuleSet,
    pub timed_recovery_delay_in_minutes: Option<u32>,
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct AccessControllerTimedConfirmRecoveryAsPrimaryInvocation {
    pub receiver: ComponentAddress,
    pub rule_set: RuleSet,
    pub timed_recovery_delay_in_minutes: Option<u32>,
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

//============================================
// Access Controller Timed Confirm As Primary
//============================================

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct AccessControllerTimedConfirmRecoveryAsRecoveryMethodArgs {
    pub rule_set: RuleSet,
    pub timed_recovery_delay_in_minutes: Option<u32>,
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct AccessControllerTimedConfirmRecoveryAsRecoveryInvocation {
    pub receiver: ComponentAddress,
    pub rule_set: RuleSet,
    pub timed_recovery_delay_in_minutes: Option<u32>,
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

//======================================================
// Access Controller Cancel Recovery Attempt As Primary
//======================================================

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct AccessControllerCancelRecoveryAttemptAsPrimaryMethodArgs {
    pub rule_set: RuleSet,
    pub timed_recovery_delay_in_minutes: Option<u32>,
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct AccessControllerCancelRecoveryAttemptAsPrimaryInvocation {
    pub receiver: ComponentAddress,
    pub rule_set: RuleSet,
    pub timed_recovery_delay_in_minutes: Option<u32>,
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
    pub timed_recovery_delay_in_minutes: Option<u32>,
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct AccessControllerCancelRecoveryAttemptAsRecoveryInvocation {
    pub receiver: ComponentAddress,
    pub rule_set: RuleSet,
    pub timed_recovery_delay_in_minutes: Option<u32>,
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

//=====================================
// Access Controller Lock Primary Role
//=====================================

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct AccessControllerLockPrimaryRoleMethodArgs {}

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
