#[rustfmt::skip]
pub mod test_runner;

use crate::test_runner::TestRunner;
use scrypto::prelude::*;
use scrypto::to_struct;
use transaction::builder::ManifestBuilder;

#[test]
fn local_component_should_return_correct_info() {
    // Arrange
    let mut test_runner = TestRunner::new(true);
    let package_address = test_runner.extract_and_publish_package("component");

    // Act
    let manifest = ManifestBuilder::new()
        .call_function(
            package_address,
            "LocalComponent",
            "check_info_of_local_component",
            to_struct!(package_address, "LocalComponent".to_string()),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_success();
}
