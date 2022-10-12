use radix_engine::ledger::TypedInMemorySubstateStore;
use radix_engine::types::*;
use scrypto_unit::*;
use std::ops::Add;
use transaction::builder::ManifestBuilder;

#[test]
fn test_trace_resource_transfers() {
    // Arrange
    let mut store = TypedInMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);
    let (public_key, _, account) = test_runner.new_account();
    let package_address = test_runner.compile_and_publish("./tests/execution_trace");
    let transfer_amount = 10u8;

    // Act
    let manifest = ManifestBuilder::new(&NetworkDefinition::simulator())
        .lock_fee(10.into(), account)
        .call_scrypto_function(
            package_address,
            "ExecutionTraceTest",
            "transfer_resource_between_two_components",
            args!(transfer_amount),
        )
        .build();
    let receipt = test_runner.execute_manifest(
        manifest,
        vec![NonFungibleAddress::from_public_key(&public_key)],
    );

    // Assert
    let output = receipt.expect_commit_success();
    let (resource_address, source_component, target_component): (
        ResourceAddress,
        ComponentAddress,
        ComponentAddress,
    ) = scrypto_decode(&output.get(1).unwrap()[..]).unwrap();

    let account_component_id: ComponentId = test_runner.deref_component(account).unwrap().into();
    let source_component_id: ComponentId = test_runner
        .deref_component(source_component)
        .unwrap()
        .into();
    let target_component_id: ComponentId = test_runner
        .deref_component(target_component)
        .unwrap()
        .into();

    /* There should be three resource changes: withdrawal from the source vault,
    deposit to the target vault and withdrawal for the fee */
    assert_eq!(3, receipt.expect_commit().resource_changes.len());

    let fee_summary = &receipt.execution.fee_summary;

    let fee_resource_address = fee_summary.payments.first().unwrap().1.resource_address();

    let total_fee_paid = fee_summary.burned.add(fee_summary.tipped);

    // Source vault withdrawal
    assert!(receipt
        .expect_commit()
        .resource_changes
        .iter()
        .any(|r| r.resource_address == resource_address
            && r.component_id == source_component_id
            && r.amount == -Decimal::from(transfer_amount)));

    // Target vault deposit
    assert!(receipt
        .expect_commit()
        .resource_changes
        .iter()
        .any(|r| r.resource_address == resource_address
            && r.component_id == target_component_id
            && r.amount == Decimal::from(transfer_amount)));

    // Fee withdrawal
    assert!(receipt
        .expect_commit()
        .resource_changes
        .iter()
        .any(|r| r.resource_address == fee_resource_address
            && r.component_id == account_component_id
            && r.amount == -Decimal::from(total_fee_paid)));
}

#[test]
fn test_trace_fee_payments() {
    // Arrange
    let mut store = TypedInMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);
    let package_address = test_runner.compile_and_publish("./tests/execution_trace");

    // Prepare the component that will pay the fee
    let manifest_prepare = ManifestBuilder::new(&NetworkDefinition::simulator())
        .lock_fee(10.into(), SYS_FAUCET_COMPONENT)
        .call_method(SYS_FAUCET_COMPONENT, "free", args!())
        .call_scrypto_function(
            package_address,
            "ExecutionTraceTest",
            "create_and_fund_a_component",
            args!(Expression::entire_worktop()),
        )
        .clear_auth_zone()
        .build();

    let funded_component = test_runner
        .execute_manifest(manifest_prepare, vec![])
        .new_component_addresses()
        .into_iter()
        .nth(0)
        .unwrap()
        .clone();

    let funded_component_id: ComponentId = test_runner
        .deref_component(funded_component)
        .unwrap()
        .into();

    // Act
    let manifest = ManifestBuilder::new(&NetworkDefinition::simulator())
        .lock_fee(10.into(), SYS_FAUCET_COMPONENT)
        .call_method(
            funded_component.clone(),
            "test_lock_contingent_fee",
            args!(),
        )
        .clear_auth_zone()
        .build();

    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    let _ = receipt.expect_commit_success();
    let resource_changes = &receipt.expect_commit().resource_changes;
    let fee_summary = &receipt.execution.fee_summary;
    let total_fee_paid = fee_summary.burned.add(fee_summary.tipped);

    assert_eq!(1, resource_changes.len());
    assert!(resource_changes
        .iter()
        .any(|r| r.component_id == funded_component_id && r.amount == -total_fee_paid));
}
