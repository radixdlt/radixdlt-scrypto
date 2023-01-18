use sbor::rust::fmt::Debug;

use crate::api::api::*;
use crate::api::types::BucketId;
use crate::model::*;
use crate::wasm::*;
use crate::*;

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct AccessControllerCreateGlobalInvocation {
    pub controlled_asset: BucketId,
    pub primary_role: AccessRule,
    pub recovery_role: AccessRule,
    pub confirmation_role: AccessRule,
    pub timed_recovery_delay_in_hours: u64,
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

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct AccessControllerUpdateTimedRecoveryDelayMethodArgs {
    pub timed_recovery_delay_in_hours: u64,
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct AccessControllerUpdateTimedRecoveryDelayInvocation {
    pub receiver: ComponentAddress,
    pub timed_recovery_delay_in_hours: u64,
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

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct AccessControllerInitiateRecoveryMethodArgs {
    pub proposer: Role,
    pub proposed_primary_role: AccessRule,
    pub proposed_recovery_role: AccessRule,
    pub proposed_confirmation_role: AccessRule,
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct AccessControllerInitiateRecoveryInvocation {
    pub receiver: ComponentAddress,
    pub proposer: Role,
    pub proposed_primary_role: AccessRule,
    pub proposed_recovery_role: AccessRule,
    pub proposed_confirmation_role: AccessRule,
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

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct AccessControllerQuickConfirmRecoveryMethodArgs {
    pub proposer: (Role, Role),
    pub proposed_primary_role: AccessRule,
    pub proposed_recovery_role: AccessRule,
    pub proposed_confirmation_role: AccessRule,
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct AccessControllerQuickConfirmRecoveryInvocation {
    pub receiver: ComponentAddress,
    pub proposer: (Role, Role),
    pub proposed_primary_role: AccessRule,
    pub proposed_recovery_role: AccessRule,
    pub proposed_confirmation_role: AccessRule,
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

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct AccessControllerTimedConfirmRecoveryMethodArgs {
    pub proposer: Role,
    pub proposed_primary_role: AccessRule,
    pub proposed_recovery_role: AccessRule,
    pub proposed_confirmation_role: AccessRule,
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct AccessControllerTimedConfirmRecoveryInvocation {
    pub receiver: ComponentAddress,
    pub proposer: Role,
    pub proposed_primary_role: AccessRule,
    pub proposed_recovery_role: AccessRule,
    pub proposed_confirmation_role: AccessRule,
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

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct AccessControllerCancelRecoveryMethodArgs {
    pub proposer: Role,
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct AccessControllerCancelRecoveryInvocation {
    pub receiver: ComponentAddress,
    pub proposer: Role,
}

impl Invocation for AccessControllerCancelRecoveryInvocation {
    type Output = ();
}

impl SerializableInvocation for AccessControllerCancelRecoveryInvocation {
    type ScryptoOutput = ();
}

impl Into<CallTableInvocation> for AccessControllerCancelRecoveryInvocation {
    fn into(self) -> CallTableInvocation {
        NativeInvocation::AccessController(AccessControllerInvocation::CancelRecovery(self)).into()
    }
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct AccessControllerLockPrimaryRoleMethodArgs {
    pub receiver: ComponentAddress,
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

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct AccessControllerUnlockPrimaryRoleMethodArgs {
    pub receiver: ComponentAddress,
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
