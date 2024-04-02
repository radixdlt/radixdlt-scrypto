use radix_common::prelude::*;
use radix_engine::blueprints::package::PackageError;
use radix_engine::errors::ApplicationError;
use radix_engine::errors::RuntimeError;
use radix_engine::updates::state_updates::generate_bls128_and_keccak256_state_updates;
use radix_engine::updates::ProtocolUpdateEntry;
use radix_engine::updates::ProtocolUpdates;
use radix_engine_tests::common::PackageLoader;
use radix_engine_tests::common::*;
use radix_substate_store_interface::db_key_mapper::SpreadPrefixKeyMapper;
use scrypto_test::prelude::*;
use scrypto_test::prelude::{CustomGenesis, LedgerSimulatorBuilder};

#[test]
fn publishing_crypto_utils_without_state_flash_should_fail() {
    run_flash_test(false, false);
}

#[test]
fn publishing_crypto_utils_with_state_flash_should_succeed() {
    run_flash_test(true, true);
}

fn run_flash_test(flash_substates: bool, expect_success: bool) {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new()
        .with_custom_protocol_updates(ProtocolUpdates::none())
        .with_custom_genesis(CustomGenesis::default(
            Epoch::of(1),
            CustomGenesis::default_consensus_manager_config(),
        ))
        .build();
    if flash_substates {
        let state_updates = generate_bls128_and_keccak256_state_updates();
        let db_updates = state_updates.create_database_updates::<SpreadPrefixKeyMapper>();
        ledger.substate_db_mut().commit(&db_updates);
    }

    // Act
    let receipt = ledger.try_publish_package(PackageLoader::get("crypto_scrypto"));

    // Assert
    if expect_success {
        receipt.expect_commit_success();
    } else {
        receipt.expect_specific_failure(|e| {
            matches!(
                e,
                RuntimeError::ApplicationError(ApplicationError::PackageError(
                    PackageError::InvalidWasm(..)
                ))
            )
        });
    }
}

#[test]
fn publishing_crypto_utils_using_test_environment_without_state_flash_should_fail() {
    run_flash_test_test_environment(false, false);
}

#[test]
fn publishing_crypto_utils_using_test_environment_with_state_flash_should_succeed() {
    run_flash_test_test_environment(true, true);
}

fn run_flash_test_test_environment(enable_bls: bool, expect_success: bool) {
    // Arrange
    let test_env_builder = TestEnvironmentBuilder::new();

    let mut test_env = if enable_bls {
        test_env_builder.protocol_updates(
            ProtocolUpdates::none().and(ProtocolUpdateEntry::Bls12381AndKeccak256),
        )
    } else {
        test_env_builder.protocol_updates(ProtocolUpdates::none())
    }
    .build();

    // Act
    let result =
        PackageFactory::compile_and_publish(path_local_blueprint!("crypto_scrypto"), &mut test_env);

    // Assert
    if expect_success {
        let _package_address = result.unwrap();
    } else {
        let err = result.unwrap_err();
        assert!(matches!(
            err,
            RuntimeError::ApplicationError(ApplicationError::PackageError(
                PackageError::InvalidWasm(..)
            ))
        ));
    }
}
