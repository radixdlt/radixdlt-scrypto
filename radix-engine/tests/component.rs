use radix_engine::engine::{KernelError, RuntimeError, TrackError};
use radix_engine::ledger::TypedInMemorySubstateStore;
use radix_engine::types::*;
use scrypto::address::Bech32Decoder;
use scrypto::engine::types::SubstateId;
use scrypto_unit::*;
use transaction::builder::ManifestBuilder;

#[test]
fn test_component() {
    let mut store = TypedInMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);
    let (public_key, _, account) = test_runner.new_account();
    let package = test_runner.compile_and_publish("./tests/component");

    // Create component
    let manifest1 = ManifestBuilder::new(&NetworkDefinition::simulator())
        .lock_fee(10.into(), SYS_FAUCET_COMPONENT)
        .call_scrypto_function(package, "ComponentTest", "create_component", args!())
        .build();
    let receipt1 = test_runner.execute_manifest(manifest1, vec![]);
    receipt1.expect_commit_success();

    // Find the component address from receipt
    let component = receipt1
        .expect_commit()
        .entity_changes
        .new_component_addresses[0];

    // Call functions & methods
    let manifest2 = ManifestBuilder::new(&NetworkDefinition::simulator())
        .lock_fee(10.into(), SYS_FAUCET_COMPONENT)
        .call_scrypto_function(
            package,
            "ComponentTest",
            "get_component_info",
            args!(component),
        )
        .call_method(component, "get_component_state", args!())
        .call_method(component, "put_component_state", args!())
        .call_method(
            account,
            "deposit_batch",
            args!(Expression::entire_worktop()),
        )
        .build();
    let receipt2 = test_runner.execute_manifest(
        manifest2,
        vec![NonFungibleAddress::from_public_key(&public_key)],
    );
    receipt2.expect_commit_success();
}

#[test]
fn invalid_blueprint_name_should_cause_error() {
    // Arrange
    let mut store = TypedInMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);
    let package_address = test_runner.compile_and_publish("./tests/component");

    // Act
    let manifest = ManifestBuilder::new(&NetworkDefinition::simulator())
        .lock_fee(10.into(), SYS_FAUCET_COMPONENT)
        .call_scrypto_function(
            package_address,
            "NonExistentBlueprint",
            "create_component",
            args!(),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_specific_failure(|e| {
        if let RuntimeError::KernelError(KernelError::BlueprintNotFound(addr, blueprint)) = e {
            addr.eq(&package_address) && blueprint.eq("NonExistentBlueprint")
        } else {
            false
        }
    });
}

#[test]
fn reentrancy_should_not_be_possible() {
    // Arrange
    let mut store = TypedInMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);
    let package_address = test_runner.compile_and_publish("./tests/component");
    let manifest = ManifestBuilder::new(&NetworkDefinition::simulator())
        .lock_fee(10u32.into(), SYS_FAUCET_COMPONENT)
        .call_scrypto_function(package_address, "ReentrantComponent", "new", args!())
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);
    receipt.expect_commit_success();
    let component_address = receipt
        .expect_commit()
        .entity_changes
        .new_component_addresses[0];

    // Act
    let manifest = ManifestBuilder::new(&NetworkDefinition::simulator())
        .lock_fee(10u32.into(), SYS_FAUCET_COMPONENT)
        .call_method(component_address, "call_self", args!())
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_specific_failure(|e| {
        if let RuntimeError::KernelError(KernelError::TrackError(TrackError::NotAvailable(
            substate_id,
        ))) = e
        {
            substate_id.eq(&SubstateId(
                RENodeId::Component(component_address),
                SubstateOffset::Component(ComponentOffset::Info),
            ))
        } else {
            false
        }
    });
}

#[test]
fn missing_component_address_in_manifest_should_cause_rejection() {
    // Arrange
    let mut store = TypedInMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);
    let _ = test_runner.compile_and_publish("./tests/component");
    let component_address = Bech32Decoder::new(&NetworkDefinition::simulator())
        .validate_and_decode_component_address(
            "component_sim1qgqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqph4dhmhs42ee03",
        )
        .unwrap();

    // Act
    let manifest = ManifestBuilder::new(&NetworkDefinition::simulator())
        .lock_fee(10.into(), SYS_FAUCET_COMPONENT)
        .call_method(component_address, "get_component_state", args!())
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_rejection();
}
