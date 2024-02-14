use radix_engine::blueprints::package::PackageError;
use radix_engine::errors::ApplicationError;
use radix_engine::errors::RuntimeError;
use radix_engine::utils::generate_vm_boot_scrypto_version_state_updates;
use radix_engine_common::prelude::*;
use radix_engine_tests::common::PackageLoader;
use radix_engine_tests::common::*;
use scrypto_test::prelude::*;
use scrypto_test::prelude::{CustomGenesis, TestRunnerBuilder};
use substate_store_interface::db_key_mapper::SpreadPrefixKeyMapper;
use substate_store_interface::interface::CommittableSubstateDatabase;

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
    let mut test_runner = TestRunnerBuilder::new()
        .without_crypto_utils_update()
        .with_custom_genesis(CustomGenesis::default(
            Epoch::of(1),
            CustomGenesis::default_consensus_manager_config(),
        ))
        .build();
    if flash_substates {
        let state_updates =
            generate_vm_boot_scrypto_version_state_updates(ScryptoVmVersion::crypto_utils_added());
        let db_updates = state_updates.create_database_updates::<SpreadPrefixKeyMapper>();
        test_runner.substate_db_mut().commit(&db_updates);
    }

    // Act
    let receipt = test_runner.try_publish_package(PackageLoader::get("crypto_scrypto"));

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

fn run_flash_test_test_environment(flash_substates: bool, expect_success: bool) {
    // Arrange
    let test_env_builder = TestEnvironmentBuilder::new();

    let mut test_env = if flash_substates {
        test_env_builder.with_crypto_utils_protocol_update()
    } else {
        test_env_builder.without_crypto_utils_protocol_update()
    }
    .build();

    // Act
    let result =
        Package::compile_and_publish(path_local_blueprint!("crypto_scrypto"), &mut test_env);

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
