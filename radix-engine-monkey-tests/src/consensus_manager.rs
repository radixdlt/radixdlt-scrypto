use crate::{SystemTestFuzzer, ValidatorMeta};
use radix_common::constants::{CONSENSUS_MANAGER, XRD};
use radix_common::prelude::ComponentAddress;
use radix_common::prelude::*;
use radix_engine_interface::blueprints::consensus_manager::{
    ConsensusManagerCreateValidatorManifestInput, CONSENSUS_MANAGER_CREATE_VALIDATOR_IDENT,
};
use radix_engine_interface::types::FromRepr;
use radix_transactions::builder::ManifestBuilder;

#[repr(u8)]
#[derive(Copy, Clone, Debug, FromRepr, Ord, PartialOrd, Eq, PartialEq)]
pub enum ConsensusManagerFuzzAction {
    CreateValidator,
}

impl ConsensusManagerFuzzAction {
    pub fn add_to_manifest(
        &self,
        uuid: u64,
        builder: ManifestBuilder,
        fuzzer: &mut SystemTestFuzzer,
        _validators: &Vec<ValidatorMeta>,
        account_address: ComponentAddress,
    ) -> (ManifestBuilder, bool) {
        match self {
            ConsensusManagerFuzzAction::CreateValidator => {
                let amount = fuzzer.next_amount();
                let fee_factor = fuzzer.next_amount();
                let key = Secp256k1PrivateKey::from_u64(1u64).unwrap().public_key();

                let bucket_name = format!("stake{}", uuid);

                let builder = builder
                    .withdraw_from_account(account_address, XRD, amount)
                    .take_all_from_worktop(XRD, bucket_name.as_str())
                    .with_bucket(bucket_name.as_str(), |builder, bucket| {
                        builder.call_method(
                            CONSENSUS_MANAGER,
                            CONSENSUS_MANAGER_CREATE_VALIDATOR_IDENT,
                            ConsensusManagerCreateValidatorManifestInput {
                                key,
                                fee_factor,
                                xrd_payment: bucket,
                            },
                        )
                    });

                (builder, false)
            }
        }
    }
}
