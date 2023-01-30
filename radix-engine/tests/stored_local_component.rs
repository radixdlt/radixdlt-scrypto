use radix_engine::types::*;
use scrypto_unit::*;
use transaction::builder::ManifestBuilder;

#[test]
#[ignore = "FIXME: Node root-ness property is not tracked properly"]
fn should_not_be_able_call_owned_components_directly() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let package_address = test_runner.compile_and_publish("./tests/blueprints/local_component");
    let manifest = ManifestBuilder::new()
        .lock_fee(FAUCET_COMPONENT, 10.into())
        .call_function(
            package_address,
            "StoredSecret",
            "new_global",
            args!(34567u32),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);
    receipt.expect_commit_success();
    let component_address = receipt
        .expect_commit()
        .entity_changes
        .new_component_addresses[1];

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee(FAUCET_COMPONENT, 10.into())
        .call_method(component_address, "get_secret", args!())
        .build();

    // Assert
    let receipt = test_runner.execute_manifest(manifest, vec![]);
    receipt.expect_rejection();
}

#[test]
fn should_be_able_to_call_read_method_on_a_stored_component_in_owned_component() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let package_address = test_runner.compile_and_publish("./tests/blueprints/local_component");

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee(FAUCET_COMPONENT, 10.into())
        .call_function(
            package_address,
            "StoredSecret",
            "call_read_on_stored_component_in_owned_component",
            args!(),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn should_be_able_to_call_write_method_on_a_stored_component_in_owned_component() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let package_address = test_runner.compile_and_publish("./tests/blueprints/local_component");

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee(FAUCET_COMPONENT, 10.into())
        .call_function(
            package_address,
            "StoredSecret",
            "call_write_on_stored_component_in_owned_component",
            args!(),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn should_be_able_to_call_read_method_on_a_stored_component_in_global_component() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let package_address = test_runner.compile_and_publish("./tests/blueprints/local_component");
    let manifest = ManifestBuilder::new()
        .lock_fee(FAUCET_COMPONENT, 10.into())
        .call_function(
            package_address,
            "StoredSecret",
            "new_global",
            args!(34567u32),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);
    receipt.expect_commit_success();
    let component_address = receipt
        .expect_commit()
        .entity_changes
        .new_component_addresses[0];

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee(FAUCET_COMPONENT, 10.into())
        .call_method(component_address, "parent_get_secret", args!())
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_commit_success();
    let rtn: u32 = receipt.output(1);
    assert_eq!(rtn, 34567u32);
}

#[test]
fn should_be_able_to_call_write_method_on_a_stored_component_in_global_component() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let package_address = test_runner.compile_and_publish("./tests/blueprints/local_component");
    let manifest = ManifestBuilder::new()
        .lock_fee(FAUCET_COMPONENT, 10.into())
        .call_function(
            package_address,
            "StoredSecret",
            "new_global",
            args!(34567u32),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);
    receipt.expect_commit_success();
    let component_address = receipt
        .expect_commit()
        .entity_changes
        .new_component_addresses[0];

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee(FAUCET_COMPONENT, 10.into())
        .call_method(component_address, "parent_set_secret", args!(8888u32))
        .call_method(component_address, "parent_get_secret", args!())
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_commit_success();
    let rtn: u32 = receipt.output(2);
    assert_eq!(rtn, 8888u32);
}

#[test]
fn should_be_able_to_call_read_method_on_a_kv_stored_component_in_owned_component() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let package_address = test_runner.compile_and_publish("./tests/blueprints/local_component");

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee(FAUCET_COMPONENT, 10.into())
        .call_function(
            package_address,
            "StoredKVLocal",
            "call_read_on_stored_component_in_owned_component",
            args!(),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn should_be_able_to_call_write_method_on_a_kv_stored_component_in_owned_component() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let package_address = test_runner.compile_and_publish("./tests/blueprints/local_component");

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee(FAUCET_COMPONENT, 10.into())
        .call_function(
            package_address,
            "StoredKVLocal",
            "call_write_on_stored_component_in_owned_component",
            args!(),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn should_be_able_to_call_read_method_on_a_kv_stored_component_in_global_component() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let package_address = test_runner.compile_and_publish("./tests/blueprints/local_component");
    let manifest = ManifestBuilder::new()
        .lock_fee(FAUCET_COMPONENT, 10.into())
        .call_function(
            package_address,
            "StoredKVLocal",
            "new_global",
            args!(34567u32),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);
    receipt.expect_commit_success();
    let component_address = receipt
        .expect_commit()
        .entity_changes
        .new_component_addresses[0];

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee(FAUCET_COMPONENT, 10.into())
        .call_method(component_address, "parent_get_secret", args!())
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_commit_success();
    let rtn: u32 = receipt.output(1);
    assert_eq!(rtn, 34567u32);
}

#[test]
fn should_be_able_to_call_write_method_on_a_kv_stored_component_in_global_component() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let package_address = test_runner.compile_and_publish("./tests/blueprints/local_component");
    let manifest = ManifestBuilder::new()
        .lock_fee(FAUCET_COMPONENT, 10.into())
        .call_function(
            package_address,
            "StoredKVLocal",
            "new_global",
            args!(34567u32),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);
    receipt.expect_commit_success();
    let component_address = receipt
        .expect_commit()
        .entity_changes
        .new_component_addresses[0];

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee(FAUCET_COMPONENT, 10.into())
        .call_method(component_address, "parent_set_secret", args!(8888u32))
        .call_method(component_address, "parent_get_secret", args!())
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_commit_success();
    let rtn: u32 = receipt.output(2);
    assert_eq!(rtn, 8888u32);
}
