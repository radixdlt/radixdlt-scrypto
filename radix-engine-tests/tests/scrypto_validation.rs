use radix_engine::blueprints::package::PackageError;
use radix_engine::errors::{ApplicationError, RuntimeError};
use radix_engine::types::*;
use radix_engine_interface::blueprints::package::PackageDefinition;
use scrypto_unit::*;
use transaction::prelude::*;

#[test]
fn cannot_create_more_than_1_substate_field_in_scrypto() {
    // Arrange
    let mut test_runner = TestRunnerBuilder::new().build();
    let code = wat2wasm(include_str!("wasm/basic_package.wat"));

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .publish_package(
            code,
            PackageDefinition::new_with_fields_test_definition("Test", 2, vec![]),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::ApplicationError(ApplicationError::PackageError(
                PackageError::WasmUnsupported(..)
            ))
        )
    });
}
