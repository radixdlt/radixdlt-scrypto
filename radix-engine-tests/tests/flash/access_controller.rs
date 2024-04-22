use radix_common::prelude::*;
use radix_engine::blueprints::access_controller::v1::*;
use radix_substate_store_queries::typed_substate_layout::*;

/// The state of the access controller changes with the bottlenose protocol update where we're
/// adding a new XRD vault to the state. This test ensures that we don't have any regression from
/// the refactoring and that the package definition for the v1.0 access controller package remains
/// the same.
#[test]
fn access_controller_package_definition_v1_0_matches_expected() {
    // Arrange
    let expected_package_definition = manifest_decode::<PackageDefinition>(include_bytes!(
        "../../assets/access_controller_v1_0_package_definition.rpd"
    ))
    .unwrap();

    // Act
    let package_definition = AccessControllerV1NativePackage::definition();

    // Assert
    assert_eq!(package_definition, expected_package_definition);
}
