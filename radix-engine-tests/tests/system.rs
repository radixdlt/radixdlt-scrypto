mod package_loader;

use package_loader::PackageLoader;
use radix_engine::types::*;
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
    receipt.expect_commit_failure();
}
