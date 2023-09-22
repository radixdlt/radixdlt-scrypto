use crate::{SystemTestFuzzer, ValidatorMeta};
use radix_engine_common::prelude::ComponentAddress;
use radix_engine_interface::blueprints::access_controller::{
    AccessControllerCreateProofInput, AccessControllerInitiateRecoveryAsPrimaryInput,
    AccessControllerQuickConfirmPrimaryRoleRecoveryProposalInput,
    ACCESS_CONTROLLER_CREATE_PROOF_IDENT, ACCESS_CONTROLLER_INITIATE_RECOVERY_AS_PRIMARY_IDENT,
    ACCESS_CONTROLLER_QUICK_CONFIRM_PRIMARY_ROLE_RECOVERY_PROPOSAL_IDENT,
};
use radix_engine_interface::types::FromRepr;
use transaction::builder::ManifestBuilder;

#[repr(u8)]
#[derive(Copy, Clone, Debug, FromRepr, Ord, PartialOrd, Eq, PartialEq)]
pub enum ProofFromAccountAction {
    CreateProofOfAmount,
}

impl ProofFromAccountAction {
    pub fn add_to_manifest(
        &self,
        _uuid: u64,
        builder: ManifestBuilder,
        fuzzer: &mut SystemTestFuzzer,
        _meta: &Vec<ValidatorMeta>,
        account: ComponentAddress,
    ) -> (ManifestBuilder, bool) {
        match self {
            ProofFromAccountAction::CreateProofOfAmount => {
                let builder = builder.create_proof_from_account_of_amount(
                    account,
                    fuzzer.next_resource(),
                    fuzzer.next_amount(),
                );
                (builder, false)
            }
        }
    }
}

#[repr(u8)]
#[derive(Copy, Clone, Debug, FromRepr, Ord, PartialOrd, Eq, PartialEq)]
pub enum AccessControllerFuzzAction {
    InitiateRecoveryAsPrimary,
    ConfirmRecoveryAsRecovery,
    CreateProof,
}

impl AccessControllerFuzzAction {
    pub fn add_to_manifest(
        &self,
        _uuid: u64,
        builder: ManifestBuilder,
        fuzzer: &mut SystemTestFuzzer,
        _meta: &Vec<ValidatorMeta>,
        access_controller: ComponentAddress,
    ) -> (ManifestBuilder, bool) {
        match self {
            AccessControllerFuzzAction::InitiateRecoveryAsPrimary => {
                let builder = builder.call_method(
                    access_controller,
                    ACCESS_CONTROLLER_INITIATE_RECOVERY_AS_PRIMARY_IDENT,
                    AccessControllerInitiateRecoveryAsPrimaryInput {
                        rule_set: fuzzer.next_rule_set(),
                        timed_recovery_delay_in_minutes: None,
                    },
                );
                (builder, false)
            }
            AccessControllerFuzzAction::ConfirmRecoveryAsRecovery => {
                let builder = builder.call_method(
                    access_controller,
                    ACCESS_CONTROLLER_QUICK_CONFIRM_PRIMARY_ROLE_RECOVERY_PROPOSAL_IDENT,
                    AccessControllerQuickConfirmPrimaryRoleRecoveryProposalInput {
                        rule_set: fuzzer.next_rule_set(),
                        timed_recovery_delay_in_minutes: None,
                    },
                );
                (builder, false)
            }
            AccessControllerFuzzAction::CreateProof => {
                let builder = builder.call_method(
                    access_controller,
                    ACCESS_CONTROLLER_CREATE_PROOF_IDENT,
                    AccessControllerCreateProofInput {},
                );
                (builder, false)
            }
        }
    }
}
