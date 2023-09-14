mod package_loader;

use package_loader::PackageLoader;
use radix_engine::errors::{ApplicationError, RuntimeError};
use radix_engine_queries::typed_substate_layout::RoleAssignmentError;
use scrypto::prelude::*;
use scrypto_unit::TestRunnerBuilder;
use transaction::prelude::ManifestBuilder;

#[test]
fn setting_reserved_role_on_role_assignment_before_attachment_fails() {
    // Arrange
    let mut test_runner = TestRunnerBuilder::new().build();

    let package_address =
        test_runner.publish_package_simple(PackageLoader::get("role-assignment-edge-cases"));

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package_address,
            "RoleAssignmentEdgeCases",
            "instantiate",
            manifest_args!(),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_specific_failure(|error| {
        matches!(
            error,
            RuntimeError::ApplicationError(ApplicationError::RoleAssignmentError(
                RoleAssignmentError::UsedReservedRole(..)
            ))
        )
    });
}
