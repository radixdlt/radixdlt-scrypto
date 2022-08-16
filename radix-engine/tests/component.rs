use radix_engine::engine::RuntimeError;
use radix_engine::ledger::TypedInMemorySubstateStore;
use scrypto::address::Bech32Decoder;
use scrypto::core::Network;
use scrypto::engine::types::SubstateId;
use scrypto::prelude::*;
use scrypto::to_struct;
use scrypto_unit::*;
use transaction::builder::ManifestBuilder;

#[test]
fn test_component() {
    let mut store = TypedInMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);
    let (public_key, _, account) = test_runner.new_account();
    let package = test_runner.extract_and_publish_package("component");

    // Create component
    let manifest1 = ManifestBuilder::new(Network::LocalSimulator)
        .lock_fee(10.into(), SYSTEM_COMPONENT)
        .call_function(package, "ComponentTest", "create_component", to_struct!())
        .build();
    let receipt1 = test_runner.execute_manifest(manifest1, vec![]);
    receipt1.expect_success();

    // Find the component address from receipt
    let component = receipt1.new_component_addresses[0];

    // Call functions & methods
    let manifest2 = ManifestBuilder::new(Network::LocalSimulator)
        .lock_fee(10.into(), SYSTEM_COMPONENT)
        .call_function(
            package,
            "ComponentTest",
            "get_component_info",
            to_struct!(component),
        )
        .call_method(component, "get_component_state", to_struct!())
        .call_method(component, "put_component_state", to_struct!())
        .call_method_with_all_resources(account, "deposit_batch")
        .build();
    let receipt2 = test_runner.execute_manifest(manifest2, vec![public_key]);
    receipt2.expect_success();
}

#[test]
fn invalid_blueprint_name_should_cause_error() {
    // Arrange
    let mut store = TypedInMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);
    let package_address = test_runner.extract_and_publish_package("component");

    // Act
    let manifest = ManifestBuilder::new(Network::LocalSimulator)
        .lock_fee(10.into(), SYSTEM_COMPONENT)
        .call_function(
            package_address,
            "NonExistentBlueprint",
            "create_component",
            to_struct!(),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_failure(|e| {
        if let RuntimeError::BlueprintNotFound(addr, blueprint) = e {
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
    let package_address = test_runner.extract_and_publish_package("component");
    let manifest = ManifestBuilder::new(Network::LocalSimulator)
        .lock_fee(10.into(), SYSTEM_COMPONENT)
        .call_function(package_address, "ReentrantComponent", "new", to_struct!())
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);
    receipt.expect_success();
    let component_address = receipt.new_component_addresses[0];

    // Act
    let manifest = ManifestBuilder::new(Network::LocalSimulator)
        .lock_fee(10.into(), SYSTEM_COMPONENT)
        .call_method(component_address, "call_self", to_struct!())
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_failure(|e| {
        if let RuntimeError::Reentrancy(SubstateId::ComponentState(address)) = e {
            address.eq(&component_address)
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
    let _ = test_runner.extract_and_publish_package("component");
    let component_address = Bech32Decoder::new_from_network(&Network::LocalSimulator)
        .validate_and_decode_component_address(
            "component_sim1qgqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqph4dhmhs42ee03",
        )
        .unwrap();

    // Act
    let manifest = ManifestBuilder::new(Network::LocalSimulator)
        .lock_fee(10.into(), SYSTEM_COMPONENT)
        .call_method(component_address, "get_component_state", to_struct!())
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_rejection();
}
