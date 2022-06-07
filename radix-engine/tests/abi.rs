#[rustfmt::skip]
pub mod test_runner;

use crate::test_runner::TestRunner;
use radix_engine::engine::RuntimeError;
use scrypto::prelude::*;
use scrypto::to_struct;
use transaction::builder::ManifestBuilder;

#[test]
fn test_invalid_access_rule_methods() {
    // Arrange
    let mut test_runner = TestRunner::new(true);
    let package_address = test_runner.extract_and_publish_package("abi");

    // Act
    let manifest = ManifestBuilder::new().call_function(
        package_address,
        "AbiComponent",
        "create_invalid_abi_component",
        to_struct!(),
    ).build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    let error = receipt.result.expect_err("Should be an error.");
    if !matches!(error, RuntimeError::BlueprintFunctionDoesNotExist(_)) {
        panic!(
            "Should be an function does not exist but error was {}",
            error
        );
    }
}
