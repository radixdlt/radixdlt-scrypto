use radix_engine::types::*;
use radix_engine_interface::model::FromPublicKey;
use scrypto_unit::*;
use transaction::builder::ManifestBuilder;

#[test]
fn stored_component_addresses_in_non_globalized_component_are_invokable() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let package = test_runner.compile_and_publish("./tests/blueprints/stored_external_component");

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee(FAUCET_COMPONENT, 10.into())
        .call_function(package, "ExternalComponent", "create_and_call", args!())
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    receipt.expect_commit_success();
}

#[test]
fn stored_component_addresses_are_invokable() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let (public_key, _, _) = test_runner.new_allocated_account();
    let package = test_runner.compile_and_publish("./tests/blueprints/stored_external_component");
    let manifest1 = ManifestBuilder::new()
        .lock_fee(FAUCET_COMPONENT, 10.into())
        .call_function(package, "ExternalComponent", "create", args!())
        .build();
    let receipt1 = test_runner.execute_manifest(manifest1, vec![]);
    receipt1.expect_commit_success();
    let component0 = receipt1
        .expect_commit()
        .entity_changes
        .new_component_addresses[0];
    let component1 = receipt1
        .expect_commit()
        .entity_changes
        .new_component_addresses[1];

    // Act
    let manifest2 = ManifestBuilder::new()
        .lock_fee(FAUCET_COMPONENT, 10.into())
        .call_method(component0, "func", args!())
        .build();
    let receipt2 = test_runner.execute_manifest(
        manifest2,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    // Assert
    receipt2.expect_commit_success();

    // Act
    let manifest2 = ManifestBuilder::new()
        .lock_fee(FAUCET_COMPONENT, 10.into())
        .call_method(component1, "func", args!())
        .build();
    let receipt2 = test_runner.execute_manifest(
        manifest2,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    // Assert
    receipt2.expect_commit_success();
}
