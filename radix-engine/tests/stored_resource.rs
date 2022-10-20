use radix_engine::ledger::TypedInMemorySubstateStore;
use radix_engine::types::*;
use scrypto_unit::*;
use transaction::builder::ManifestBuilder;

#[test]
fn stored_resource_is_invokeable() {
    // Arrange
    let mut store = TypedInMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);
    let package = test_runner.compile_and_publish("./tests/stored_resource");
    let manifest = ManifestBuilder::new(&NetworkDefinition::simulator())
        .lock_fee(10.into(), SYS_FAUCET_COMPONENT)
        .call_function(package, "StoredResource", "create", args!())
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);
    receipt.expect_commit_success();
    let component = receipt
        .expect_commit()
        .entity_changes
        .new_component_addresses[0];

    // Act
    let manifest2 = ManifestBuilder::new(&NetworkDefinition::simulator())
        .lock_fee(10.into(), SYS_FAUCET_COMPONENT)
        .call_method(component, "total_supply", args!())
        .build();
    let receipt = test_runner.execute_manifest(manifest2, vec![]);

    // Assert
    receipt.expect_commit_success();
}
