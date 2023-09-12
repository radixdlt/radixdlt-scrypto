mod package_loader;

use package_loader::PackageLoader;
use radix_engine::{
    errors::{RuntimeError, SystemError},
    types::*,
};
use radix_engine_queries::typed_substate_layout::PACKAGE_BLUEPRINT;
use scrypto_unit::*;
use transaction::prelude::*;

#[test]
fn test_handle_mismatch() {
    // Arrange
    let mut test_runner = TestRunnerBuilder::new().without_trace().build();
    let package_address = test_runner.publish_package_simple(PackageLoader::get("system"));
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package_address,
            "HandleMismatchTest",
            "new",
            manifest_args!(),
        )
        .build();
    let component_address = test_runner
        .execute_manifest(manifest, vec![])
        .expect_commit_success()
        .new_component_addresses()[0];

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_method(
            component_address,
            "treat_field_handle_as_kv_store_handle",
            manifest_args!(),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);
    println!("{:?}", receipt);

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::SystemError(SystemError::NotAKeyValueEntryWriteHandle)
        )
    });
}

#[test]
fn test_put_address_reservation_into_component_state() {
    // Arrange
    let mut test_runner = TestRunnerBuilder::new().without_trace().build();
    let package_address = test_runner.publish_package_simple(PackageLoader::get("system"));
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .allocate_global_address(PACKAGE_PACKAGE, PACKAGE_BLUEPRINT, "reservation", "address")
        .call_function_with_name_lookup(
            package_address,
            "AddressReservationTest",
            "put_address_reservation_into_component_state",
            |lookup| manifest_args!(lookup.address_reservation("reservation")),
        )
        .build();
    test_runner
        .execute_manifest(manifest, vec![])
        .expect_commit_failure();
}

#[test]
fn test_put_address_reservation_into_kv_store() {
    // Arrange
    let mut test_runner = TestRunnerBuilder::new().without_trace().build();
    let package_address = test_runner.publish_package_simple(PackageLoader::get("system"));
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .allocate_global_address(PACKAGE_PACKAGE, PACKAGE_BLUEPRINT, "reservation", "address")
        .call_function_with_name_lookup(
            package_address,
            "AddressReservationTest",
            "put_address_reservation_into_kv_store",
            |lookup| manifest_args!(lookup.address_reservation("reservation")),
        )
        .build();
    test_runner
        .execute_manifest(manifest, vec![])
        .expect_commit_failure();
}
