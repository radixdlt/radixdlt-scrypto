use radix_engine::ledger::TypedInMemorySubstateStore;
use radix_engine::types::*;
use scrypto_unit::*;
use transaction::builder::ManifestBuilder;

#[test]
fn stored_component_addresses_in_non_globalized_component_are_invokable() {
    // Arrange
    let mut store = TypedInMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);
    let package = test_runner.compile_and_publish("./tests/stored_external_component");

    // Act
    let manifest = ManifestBuilder::new(&NetworkDefinition::simulator())
        .lock_fee(10.into(), SYS_FAUCET_COMPONENT)
        .call_function(package, "ExternalComponent", "create_and_call", args!())
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    receipt.expect_commit_success();
}

#[test]
fn stored_component_addresses_are_invokable() {
    // Arrange
    let mut store = TypedInMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);
    let (public_key, _, _) = test_runner.new_account();
    let package = test_runner.compile_and_publish("./tests/stored_external_component");
    let manifest1 = ManifestBuilder::new(&NetworkDefinition::simulator())
        .lock_fee(10.into(), SYS_FAUCET_COMPONENT)
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
    let manifest2 = ManifestBuilder::new(&NetworkDefinition::simulator())
        .lock_fee(10.into(), SYS_FAUCET_COMPONENT)
        .call_method(component0, "func", args!())
        .build();
    let receipt2 = test_runner.execute_manifest(
        manifest2,
        vec![NonFungibleAddress::from_public_key(&public_key)],
    );

    // Assert
    receipt2.expect_commit_success();

    // Act
    let manifest2 = ManifestBuilder::new(&NetworkDefinition::simulator())
        .lock_fee(10.into(), SYS_FAUCET_COMPONENT)
        .call_method(component1, "func", args!())
        .build();
    let receipt2 = test_runner.execute_manifest(
        manifest2,
        vec![NonFungibleAddress::from_public_key(&public_key)],
    );

    // Assert
    receipt2.expect_commit_success();
}
