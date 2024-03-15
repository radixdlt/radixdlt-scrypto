use radix_common::prelude::*;
use radix_engine_interface::types::FromPublicKey;
use radix_engine_tests::common::*;
use scrypto_test::prelude::*;

#[test]
fn stored_component_addresses_in_non_globalized_component_are_invocable() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let package = ledger.publish_package_simple(PackageLoader::get("stored_external_component"));

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package,
            "ExternalComponent",
            "create_and_call",
            manifest_args!(),
        )
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![]);

    receipt.expect_commit_success();
}

#[test]
fn stored_component_addresses_are_invocable() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (public_key, _, _) = ledger.new_allocated_account();
    let package = ledger.publish_package_simple(PackageLoader::get("stored_external_component"));
    let manifest1 = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(package, "ExternalComponent", "create", manifest_args!())
        .build();
    let receipt1 = ledger.execute_manifest(manifest1, vec![]);
    receipt1.expect_commit_success();
    let component0 = receipt1.expect_commit(true).new_component_addresses()[0];
    let component1 = receipt1.expect_commit(true).new_component_addresses()[1];

    // Act
    let manifest2 = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_method(component0, "func", manifest_args!())
        .build();
    let receipt2 = ledger.execute_manifest(
        manifest2,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    // Assert
    receipt2.expect_commit_success();

    // Act
    let manifest2 = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_method(component1, "func", manifest_args!())
        .build();
    let receipt2 = ledger.execute_manifest(
        manifest2,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    // Assert
    receipt2.expect_commit_success();
}
