use crate::{SystemTestFuzzer, ValidatorMeta};
use radix_engine_common::prelude::ComponentAddress;
use radix_engine_interface::blueprints::access_controller::{ACCESS_CONTROLLER_INITIATE_RECOVERY_AS_PRIMARY_IDENT, AccessControllerInitiateRecoveryAsPrimaryInput};
use radix_engine_interface::types::FromRepr;
use transaction::builder::ManifestBuilder;

#[repr(u8)]
#[derive(Copy, Clone, Debug, FromRepr, Ord, PartialOrd, Eq, PartialEq)]
pub enum AccessControllerFuzzAction {
    InitiateRecoveryAsPrimary,
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
                let builder = builder.call_method(access_controller, ACCESS_CONTROLLER_INITIATE_RECOVERY_AS_PRIMARY_IDENT, AccessControllerInitiateRecoveryAsPrimaryInput {
                    rule_set: fuzzer.next_rule_set(),
                    timed_recovery_delay_in_minutes: None,
                });
                (builder, false)
            }
        }
    }
}