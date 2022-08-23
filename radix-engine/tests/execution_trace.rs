use radix_engine::ledger::TypedInMemorySubstateStore;
use scrypto::core::NetworkDefinition;
use scrypto::prelude::*;
use scrypto_unit::*;
use transaction::builder::ManifestBuilder;

#[test]
fn test_trace_resource_transfers() {
    // Arrange
    let mut store = TypedInMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);
    let (public_key, _, account) = test_runner.new_account();
    let package_address = test_runner.extract_and_publish_package("execution_trace");
    let transfer_amount = 10u8;

    // Act
    let manifest = ManifestBuilder::new(&NetworkDefinition::local_simulator())
        .lock_fee(10.into(), account)
        .call_function(
            package_address,
            "ExecutionTraceTest",
            "transfer_resource_between_two_components",
            args!(transfer_amount),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![public_key]);

    // Assert
    let output = receipt.expect_success();
    let (resource_address, source_component, target_component): (
        ResourceAddress,
        ComponentAddress,
        ComponentAddress,
    ) = scrypto_decode(&output.get(1).unwrap()[..]).unwrap();
    /* There should be two resource changes, one for source component and one for target */
    assert_eq!(2, receipt.expect_commit().resource_changes.len());
    assert!(receipt
        .expect_commit()
        .resource_changes
        .iter()
        .any(|r| r.resource_address == resource_address
            && r.component_address == source_component
            && r.amount == -Decimal::from(transfer_amount)));
    assert!(receipt
        .expect_commit()
        .resource_changes
        .iter()
        .any(|r| r.resource_address == resource_address
            && r.component_address == target_component
            && r.amount == Decimal::from(transfer_amount)));
}
