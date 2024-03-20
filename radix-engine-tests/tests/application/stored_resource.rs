use radix_common::prelude::*;
use radix_engine_tests::common::*;
use scrypto_test::prelude::*;

#[test]
fn stored_resource_is_invokeable() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let package = ledger.publish_package_simple(PackageLoader::get("stored_resource"));
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(package, "StoredResource", "create", manifest_args!())
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![]);
    let component = receipt.expect_commit(true).new_component_addresses()[0];

    // Act
    let manifest2 = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_method(component, "total_supply", manifest_args!())
        .build();
    let receipt = ledger.execute_manifest(manifest2, vec![]);

    // Assert
    receipt.expect_commit_success();
}
