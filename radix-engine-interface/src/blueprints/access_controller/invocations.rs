use crate::api::types::*;
use crate::blueprints::access_controller::*;
use crate::blueprints::resource::*;
use crate::*;
use sbor::rust::collections::BTreeMap;
use sbor::rust::fmt::Debug;
use scrypto_abi::BlueprintAbi;

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

#[derive(Debug, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
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

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct AccessControllerCreateProofInput {
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

    fn fn_identifier(&self) -> FnIdentifier {
        FnIdentifier::Native(NativeFn::AccessController(
            AccessControllerFn::InitiateRecoveryAsPrimary,
        ))
    }
}

impl SerializableInvocation for AccessControllerInitiateRecoveryAsPrimaryInvocation {
    type ScryptoOutput = ();

    fn native_fn() -> NativeFn {
        NativeFn::AccessController(AccessControllerFn::InitiateRecoveryAsPrimary)
    }
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

    fn fn_identifier(&self) -> FnIdentifier {
        FnIdentifier::Native(NativeFn::AccessController(
            AccessControllerFn::InitiateRecoveryAsRecovery,
        ))
    }
}

impl SerializableInvocation for AccessControllerInitiateRecoveryAsRecoveryInvocation {
    type ScryptoOutput = ();

    fn native_fn() -> NativeFn {
        NativeFn::AccessController(AccessControllerFn::InitiateRecoveryAsRecovery)
    }
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

    fn fn_identifier(&self) -> FnIdentifier {
        FnIdentifier::Native(NativeFn::AccessController(
            AccessControllerFn::QuickConfirmPrimaryRoleRecoveryProposal,
        ))
    }
}

impl SerializableInvocation for AccessControllerQuickConfirmPrimaryRoleRecoveryProposalInvocation {
    type ScryptoOutput = ();

    fn native_fn() -> NativeFn {
        NativeFn::AccessController(AccessControllerFn::QuickConfirmPrimaryRoleRecoveryProposal)
    }
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

    fn fn_identifier(&self) -> FnIdentifier {
        FnIdentifier::Native(NativeFn::AccessController(
            AccessControllerFn::QuickConfirmPrimaryRoleRecoveryProposal,
        ))
    }
}

impl SerializableInvocation for AccessControllerQuickConfirmRecoveryRoleRecoveryProposalInvocation {
    type ScryptoOutput = ();

    fn native_fn() -> NativeFn {
        NativeFn::AccessController(AccessControllerFn::QuickConfirmPrimaryRoleRecoveryProposal)
    }
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

    fn fn_identifier(&self) -> FnIdentifier {
        FnIdentifier::Native(NativeFn::AccessController(
            AccessControllerFn::TimedConfirmRecovery,
        ))
    }
}

impl SerializableInvocation for AccessControllerTimedConfirmRecoveryInvocation {
    type ScryptoOutput = ();

    fn native_fn() -> NativeFn {
        NativeFn::AccessController(AccessControllerFn::TimedConfirmRecovery)
    }
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

    fn fn_identifier(&self) -> FnIdentifier {
        FnIdentifier::Native(NativeFn::AccessController(
            AccessControllerFn::CancelPrimaryRoleRecoveryProposal,
        ))
    }
}

impl SerializableInvocation for AccessControllerCancelPrimaryRoleRecoveryProposalInvocation {
    type ScryptoOutput = ();

    fn native_fn() -> NativeFn {
        NativeFn::AccessController(AccessControllerFn::CancelPrimaryRoleRecoveryProposal)
    }
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

    fn fn_identifier(&self) -> FnIdentifier {
        FnIdentifier::Native(NativeFn::AccessController(
            AccessControllerFn::CancelRecoveryRoleRecoveryProposal,
        ))
    }
}

impl SerializableInvocation for AccessControllerCancelRecoveryRoleRecoveryProposalInvocation {
    type ScryptoOutput = ();

    fn native_fn() -> NativeFn {
        NativeFn::AccessController(AccessControllerFn::CancelRecoveryRoleRecoveryProposal)
    }
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

    fn fn_identifier(&self) -> FnIdentifier {
        FnIdentifier::Native(NativeFn::AccessController(
            AccessControllerFn::LockPrimaryRole,
        ))
    }
}

impl SerializableInvocation for AccessControllerLockPrimaryRoleInvocation {
    type ScryptoOutput = ();

    fn native_fn() -> NativeFn {
        NativeFn::AccessController(AccessControllerFn::LockPrimaryRole)
    }
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

    fn fn_identifier(&self) -> FnIdentifier {
        FnIdentifier::Native(NativeFn::AccessController(
            AccessControllerFn::UnlockPrimaryRole,
        ))
    }
}

impl SerializableInvocation for AccessControllerUnlockPrimaryRoleInvocation {
    type ScryptoOutput = ();

    fn native_fn() -> NativeFn {
        NativeFn::AccessController(AccessControllerFn::UnlockPrimaryRole)
    }
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

    fn fn_identifier(&self) -> FnIdentifier {
        FnIdentifier::Native(NativeFn::AccessController(
            AccessControllerFn::StopTimedRecovery,
        ))
    }
}

impl SerializableInvocation for AccessControllerStopTimedRecoveryInvocation {
    type ScryptoOutput = ();

    fn native_fn() -> NativeFn {
        NativeFn::AccessController(AccessControllerFn::StopTimedRecovery)
    }
}

impl Into<CallTableInvocation> for AccessControllerStopTimedRecoveryInvocation {
    fn into(self) -> CallTableInvocation {
        NativeInvocation::AccessController(AccessControllerInvocation::StopTimedRecovery(self))
            .into()
    }
}
