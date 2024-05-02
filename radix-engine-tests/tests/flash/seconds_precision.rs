use radix_common::constants::AuthAddresses;
use radix_common::constants::CONSENSUS_MANAGER;
use radix_common::prelude::*;
use radix_common::prelude::{manifest_args, Round};
use radix_common::types::Epoch;
use radix_engine::errors::{RuntimeError, SystemError};
use radix_engine::system::system_type_checker::TypeCheckError;
use radix_engine::updates::*;
use radix_engine_interface::blueprints::consensus_manager::{
    ConsensusManagerNextRoundInput, CONSENSUS_MANAGER_NEXT_ROUND_IDENT,
};
use radix_engine_tests::common::PackageLoader;
use radix_substate_store_interface::db_key_mapper::SpreadPrefixKeyMapper;
use radix_substate_store_interface::interface::CommittableSubstateDatabase;
use radix_transactions::builder::ManifestBuilder;
use scrypto_test::prelude::{CustomGenesis, LedgerSimulatorBuilder};

#[test]
fn get_current_time_rounded_to_seconds_without_state_flash_should_fail() {
    run_flash_test(false, false);
}

#[test]
fn get_current_time_rounded_to_seconds_with_state_flash_should_succeed() {
    run_flash_test(true, true);
}

fn run_flash_test(flash_substates: bool, expect_success: bool) {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new()
        .with_custom_genesis(CustomGenesis::default(
            Epoch::of(1),
            CustomGenesis::default_consensus_manager_config(),
        ))
        .with_protocol_version(ProtocolVersion::Genesis)
        .build();
    let package_address = ledger.publish_package_simple(PackageLoader::get("clock"));

    // Act
    if flash_substates {
        let anemone_protocol_update_batch_generator = AnemoneSettings::all_disabled()
            .enable(|item| &mut item.seconds_precision)
            .create_batch_generator();
        let mut i = 0;
        while let Some(batch) =
            anemone_protocol_update_batch_generator.generate_batch(ledger.substate_db(), i)
        {
            i += 1;
            for ProtocolUpdateTransactionDetails::FlashV1Transaction(
                FlashProtocolUpdateTransactionDetails { state_updates, .. },
            ) in batch.transactions
            {
                ledger
                    .substate_db_mut()
                    .commit(&state_updates.create_database_updates::<SpreadPrefixKeyMapper>())
            }
        }
    }

    let time_to_set_ms = 1669663688996;
    let expected_unix_time_rounded_to_seconds = time_to_set_ms / 1000;
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_method(
            CONSENSUS_MANAGER,
            CONSENSUS_MANAGER_NEXT_ROUND_IDENT,
            ConsensusManagerNextRoundInput::successful(Round::of(1), 0, time_to_set_ms),
        )
        .call_function(
            package_address,
            "ClockTest",
            "get_current_time_rounded_to_seconds",
            manifest_args![],
        )
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![AuthAddresses::validator_role()]);

    // Assert
    if expect_success {
        receipt.expect_commit_success();
        let current_unix_time_rounded_to_seconds: i64 = receipt.expect_commit(true).output(2);
        assert_eq!(
            current_unix_time_rounded_to_seconds,
            expected_unix_time_rounded_to_seconds,
        );
    } else {
        receipt.expect_specific_failure(|e| {
            matches!(
                e,
                RuntimeError::SystemError(SystemError::TypeCheckError(
                    TypeCheckError::BlueprintPayloadValidationError(..)
                ))
            )
        });
    }
}
