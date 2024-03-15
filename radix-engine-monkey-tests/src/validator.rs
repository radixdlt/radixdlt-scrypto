use crate::{SystemTestFuzzer, ValidatorMeta};
use radix_common::constants::XRD;
use radix_common::data::manifest::ManifestArgs;
use radix_common::manifest_args;
use radix_common::prelude::{ComponentAddress, NonFungibleLocalId, VALIDATOR_OWNER_BADGE};
use radix_engine_interface::blueprints::consensus_manager::{
    ValidatorGetRedemptionValueInput, VALIDATOR_CLAIM_XRD_IDENT,
    VALIDATOR_FINISH_UNLOCK_OWNER_STAKE_UNITS_IDENT, VALIDATOR_GET_REDEMPTION_VALUE_IDENT,
    VALIDATOR_LOCK_OWNER_STAKE_UNITS_IDENT, VALIDATOR_REGISTER_IDENT, VALIDATOR_STAKE_IDENT,
    VALIDATOR_START_UNLOCK_OWNER_STAKE_UNITS_IDENT, VALIDATOR_UNSTAKE_IDENT,
    VALIDATOR_UPDATE_FEE_IDENT,
};
use radix_engine_interface::prelude::*;
use radix_rust::btreeset;
use radix_transactions::builder::ManifestBuilder;

#[repr(u8)]
#[derive(Copy, Clone, Debug, FromRepr, Ord, PartialOrd, Eq, PartialEq)]
pub enum ValidatorFuzzAction {
    GetRedemptionValue,
    Stake,
    Unstake,
    Claim,
    UpdateFee,
    LockOwnerStake,
    StartUnlockOwnerStake,
    FinishUnlockOwnerStake,
    Register,
}

impl ValidatorFuzzAction {
    pub fn add_to_manifest(
        &self,
        _uuid: u64,
        builder: ManifestBuilder,
        fuzzer: &mut SystemTestFuzzer,
        meta: &Vec<ValidatorMeta>,
        _account_address: ComponentAddress,
    ) -> (ManifestBuilder, bool) {
        match self {
            ValidatorFuzzAction::GetRedemptionValue => {
                let next_validator = fuzzer.next(0usize..meta.len());
                let meta = meta[next_validator];
                let amount_of_stake_units = fuzzer.next_amount();

                let builder = builder.call_method(
                    meta.validator_address,
                    VALIDATOR_GET_REDEMPTION_VALUE_IDENT,
                    ValidatorGetRedemptionValueInput {
                        amount_of_stake_units,
                    },
                );
                (builder, amount_of_stake_units.is_zero())
            }
            ValidatorFuzzAction::Stake => {
                let next_validator = fuzzer.next(0usize..meta.len());
                let meta = meta[next_validator];
                let amount_to_stake = fuzzer.next_amount();

                let builder = builder
                    .withdraw_from_account(meta.account_address, XRD, amount_to_stake)
                    .take_all_from_worktop(XRD, "xrd")
                    .with_bucket("xrd", |builder, bucket| {
                        builder.call_method(
                            meta.validator_address,
                            VALIDATOR_STAKE_IDENT,
                            manifest_args!(bucket),
                        )
                    });
                (builder, amount_to_stake.is_zero())
            }
            ValidatorFuzzAction::Unstake => {
                let next_validator = fuzzer.next(0usize..meta.len());
                let meta = meta[next_validator];
                let amount = fuzzer.next_amount();

                let builder = builder
                    .withdraw_from_account(meta.account_address, meta.stake_unit_resource, amount)
                    .take_all_from_worktop(meta.stake_unit_resource, "stake_units")
                    .with_bucket("stake_units", |builder, bucket| {
                        builder.call_method(
                            meta.validator_address,
                            VALIDATOR_UNSTAKE_IDENT,
                            manifest_args!(bucket),
                        )
                    });
                (builder, amount.is_zero())
            }
            ValidatorFuzzAction::Claim => {
                let next_validator = fuzzer.next(0usize..meta.len());
                let meta = meta[next_validator];
                let amount = fuzzer.next_amount();

                let builder = builder
                    .withdraw_from_account(meta.account_address, meta.claim_resource, amount)
                    .take_all_from_worktop(meta.claim_resource, "claim_resource")
                    .with_bucket("claim_resource", |builder, bucket| {
                        builder.call_method(
                            meta.validator_address,
                            VALIDATOR_CLAIM_XRD_IDENT,
                            manifest_args!(bucket),
                        )
                    });
                (builder, false)
            }
            ValidatorFuzzAction::UpdateFee => {
                let next_validator = fuzzer.next(0usize..meta.len());
                let meta = meta[next_validator];
                let fee_factor = fuzzer.next_amount();

                let builder = builder
                    .create_proof_from_account_of_non_fungibles(
                        meta.account_address,
                        VALIDATOR_OWNER_BADGE,
                        btreeset!(
                            NonFungibleLocalId::bytes(meta.validator_address.as_node_id().0)
                                .unwrap()
                        ),
                    )
                    .call_method(
                        meta.validator_address,
                        VALIDATOR_UPDATE_FEE_IDENT,
                        manifest_args!(fee_factor),
                    );
                (builder, fee_factor.is_zero())
            }
            ValidatorFuzzAction::LockOwnerStake => {
                let next_validator = fuzzer.next(0usize..meta.len());
                let meta = meta[next_validator];
                let amount = fuzzer.next_amount();

                let builder = builder
                    .withdraw_from_account(meta.account_address, meta.stake_unit_resource, amount)
                    .create_proof_from_account_of_non_fungibles(
                        meta.account_address,
                        VALIDATOR_OWNER_BADGE,
                        btreeset!(
                            NonFungibleLocalId::bytes(meta.validator_address.as_node_id().0)
                                .unwrap()
                        ),
                    )
                    .take_all_from_worktop(meta.stake_unit_resource, "stake_units")
                    .with_bucket("stake_units", |builder, bucket| {
                        builder.call_method(
                            meta.validator_address,
                            VALIDATOR_LOCK_OWNER_STAKE_UNITS_IDENT,
                            manifest_args!(bucket),
                        )
                    });
                (builder, amount.is_zero())
            }
            ValidatorFuzzAction::StartUnlockOwnerStake => {
                let next_validator = fuzzer.next(0usize..meta.len());
                let meta = meta[next_validator];
                let amount = fuzzer.next_amount();

                let builder = builder
                    .create_proof_from_account_of_non_fungibles(
                        meta.account_address,
                        VALIDATOR_OWNER_BADGE,
                        btreeset!(
                            NonFungibleLocalId::bytes(meta.validator_address.as_node_id().0)
                                .unwrap()
                        ),
                    )
                    .call_method(
                        meta.validator_address,
                        VALIDATOR_START_UNLOCK_OWNER_STAKE_UNITS_IDENT,
                        manifest_args!(amount),
                    );

                (builder, amount.is_zero())
            }
            ValidatorFuzzAction::FinishUnlockOwnerStake => {
                let next_validator = fuzzer.next(0usize..meta.len());
                let meta = meta[next_validator];

                let builder = builder
                    .create_proof_from_account_of_non_fungibles(
                        meta.account_address,
                        VALIDATOR_OWNER_BADGE,
                        btreeset!(
                            NonFungibleLocalId::bytes(meta.validator_address.as_node_id().0)
                                .unwrap()
                        ),
                    )
                    .call_method(
                        meta.validator_address,
                        VALIDATOR_FINISH_UNLOCK_OWNER_STAKE_UNITS_IDENT,
                        manifest_args!(),
                    );
                (builder, false)
            }
            ValidatorFuzzAction::Register => {
                let next_validator = fuzzer.next(0usize..meta.len());
                let meta = meta[next_validator];

                let builder = builder
                    .create_proof_from_account_of_non_fungibles(
                        meta.account_address,
                        VALIDATOR_OWNER_BADGE,
                        btreeset!(
                            NonFungibleLocalId::bytes(meta.validator_address.as_node_id().0)
                                .unwrap()
                        ),
                    )
                    .call_method(
                        meta.validator_address,
                        VALIDATOR_REGISTER_IDENT,
                        manifest_args!(),
                    );
                (builder, false)
            }
        }
    }
}
