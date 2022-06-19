#[rustfmt::skip]
pub mod test_runner;

use crate::test_runner::TestRunner;
use radix_engine::engine::RuntimeError;
use scrypto::prelude::*;
use scrypto::to_struct;
use transaction::builder::ManifestBuilder;

#[test]
fn should_be_able_to_read_component_info() {
    // Arrange
    let mut test_runner = TestRunner::new(true);
    let package_address = test_runner.extract_and_publish_package("data_access");

    // Act
    let manifest = ManifestBuilder::new()
        .call_function(
            package_address,
            "DataAccess",
            "create_component_and_read_info",
            to_struct!(),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_success();
}

#[test]
fn should_not_be_able_to_write_component_info() {
    // Arrange
    let mut test_runner = TestRunner::new(true);
    let package_address = test_runner.extract_and_publish_package("data_access");

    // Act
    let manifest = ManifestBuilder::new()
        .call_function(
            package_address,
            "DataAccess",
            "create_component_and_write_info",
            to_struct!(),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_err(|e| matches!(e, RuntimeError::InvalidDataWrite));
}
