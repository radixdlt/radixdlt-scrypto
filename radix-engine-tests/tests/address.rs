use radix_engine::errors::{RuntimeError, SystemError};
use radix_engine::types::*;
use radix_engine_interface::blueprints::transaction_processor::TRANSACTION_PROCESSOR_BLUEPRINT;
use scrypto_unit::*;
use transaction::builder::ManifestBuilder;

#[test]
fn get_global_address_in_local_in_function_should_fail() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let package_address = test_runner.compile_and_publish("./tests/blueprints/address");
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 10.into())
        .call_function(
            package_address,
            "CalledComponent",
            "create",
            manifest_args!(),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);
    let called_component = receipt.expect_commit(true).new_component_addresses()[0];

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 10.into())
        .call_function(
            package_address,
            "MyComponent",
            "get_global_address_in_local",
            manifest_args!(called_component),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::SystemError(SystemError::GlobalAddressDoesNotExist)
        )
    });
}

#[test]
fn get_global_address_in_local_in_method_should_fail() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let package_address = test_runner.compile_and_publish("./tests/blueprints/address");
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 10.into())
        .call_function(
            package_address,
            "CalledComponent",
            "create",
            manifest_args!(),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);
    let called_component = receipt.expect_commit(true).new_component_addresses()[0];
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 10.into())
        .call_function(
            package_address,
            "MyComponent",
            "create",
            manifest_args!(called_component),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);
    receipt.expect_commit_success();
    let component = receipt.expect_commit(true).new_component_addresses()[0];

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 10.into())
        .call_method(
            component,
            "get_global_address_in_local_of_parent_method",
            manifest_args!(called_component),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::SystemError(SystemError::GlobalAddressDoesNotExist)
        )
    });
}

#[test]
fn get_global_address_in_parent_should_succeed() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let package_address = test_runner.compile_and_publish("./tests/blueprints/address");
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 10.into())
        .call_function(
            package_address,
            "CalledComponent",
            "create",
            manifest_args!(),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);
    let called_component = receipt.expect_commit(true).new_component_addresses()[0];
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 10.into())
        .call_function(
            package_address,
            "MyComponent",
            "create",
            manifest_args!(called_component),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);
    receipt.expect_commit_success();
    let component = receipt.expect_commit(true).new_component_addresses()[0];

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 10.into())
        .call_method(component, "get_global_address_in_parent", manifest_args!())
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);
    let get_global_address_component: ComponentAddress = receipt.expect_commit(true).output(1);

    // Assert
    receipt.expect_commit_success();
    assert_eq!(component, get_global_address_component)
}

#[test]
fn get_global_address_in_child_should_succeed() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let package_address = test_runner.compile_and_publish("./tests/blueprints/address");
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 10.into())
        .call_function(
            package_address,
            "CalledComponent",
            "create",
            manifest_args!(),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);
    let called_component = receipt.expect_commit(true).new_component_addresses()[0];
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 10.into())
        .call_function(
            package_address,
            "MyComponent",
            "create",
            manifest_args!(called_component),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);
    receipt.expect_commit_success();
    let component = receipt.expect_commit(true).new_component_addresses()[0];

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 10.into())
        .call_method(component, "get_global_address_in_owned", manifest_args!())
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);
    let get_global_address_component: ComponentAddress = receipt.expect_commit(true).output(1);

    // Assert
    receipt.expect_commit_success();
    assert_eq!(component, get_global_address_component)
}

fn test_call_component_address_protected_method(caller_child: bool, callee_child: bool) {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let package_address = test_runner.compile_and_publish("./tests/blueprints/address");
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 10.into())
        .call_function(
            package_address,
            "CalledComponent",
            "create",
            manifest_args!(),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);
    let called_component = receipt.expect_commit(true).new_component_addresses()[0];
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 10.into())
        .call_function(
            package_address,
            "MyComponent",
            "create",
            manifest_args!(called_component),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);
    receipt.expect_commit_success();
    let component = receipt.expect_commit(true).new_component_addresses()[0];

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 10.into())
        .call_method(
            component,
            "call_other_component",
            manifest_args!(caller_child, callee_child),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn call_component_address_protected_method_in_parent_to_parent_should_succeed() {
    test_call_component_address_protected_method(false, false);
}

#[test]
fn call_component_address_protected_method_in_child_to_parent_should_succeed() {
    test_call_component_address_protected_method(true, false);
}

#[test]
fn call_component_address_protected_method_in_parent_to_child_should_succeed() {
    test_call_component_address_protected_method(false, true);
}

#[test]
fn call_component_address_protected_method_in_child_to_child_should_succeed() {
    test_call_component_address_protected_method(false, false);
}

enum AssertAgainst {
    SelfPackage,
    TransactionProcessorPackage,
    SelfBlueprint,
    TransactionProcessorBlueprint,
}

fn test_assert(package: AssertAgainst, child: bool, should_succeed: bool) {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let package_address = test_runner.compile_and_publish("./tests/blueprints/address");
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 10.into())
        .call_function(
            package_address,
            "CalledComponent",
            "create",
            manifest_args!(),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);
    let called_component = receipt.expect_commit(true).new_component_addresses()[0];
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 10.into())
        .call_function(
            package_address,
            "MyComponent",
            "create",
            manifest_args!(called_component),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);
    receipt.expect_commit_success();
    let component = receipt.expect_commit(true).new_component_addresses()[0];

    let (method_name, args) = match package {
        AssertAgainst::SelfPackage => (
            "assert_check_on_package",
            manifest_args!(package_address, child),
        ),
        AssertAgainst::SelfBlueprint => {
            let blueprint = Blueprint::new(&package_address, "MyComponent");
            (
                "assert_check_on_global_blueprint_caller",
                manifest_args!(blueprint, child),
            )
        }
        AssertAgainst::TransactionProcessorPackage => (
            "assert_check_on_package",
            manifest_args!(TRANSACTION_PROCESSOR_PACKAGE, child),
        ),
        AssertAgainst::TransactionProcessorBlueprint => {
            let blueprint = Blueprint::new(
                &TRANSACTION_PROCESSOR_PACKAGE,
                TRANSACTION_PROCESSOR_BLUEPRINT,
            );
            (
                "assert_check_on_global_blueprint_caller",
                manifest_args!(blueprint, child),
            )
        }
    };

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 10.into())
        .call_method(component, method_name, args)
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    if should_succeed {
        receipt.expect_commit_success();
    } else {
        receipt.expect_specific_failure(|e| {
            matches!(
                e,
                RuntimeError::SystemError(SystemError::AssertAccessRuleFailed)
            )
        });
    }
}

#[test]
fn assert_self_package_on_parent_should_fail() {
    test_assert(AssertAgainst::SelfPackage, false, false);
}

#[test]
fn assert_self_blueprint_on_parent_should_fail() {
    test_assert(AssertAgainst::SelfBlueprint, false, false);
}

#[test]
fn assert_tx_processor_package_on_parent_should_succeed() {
    test_assert(AssertAgainst::TransactionProcessorPackage, false, true);
}

#[test]
fn assert_tx_processor_blueprint_on_parent_should_succeed() {
    test_assert(AssertAgainst::TransactionProcessorBlueprint, false, true);
}

#[test]
fn assert_self_package_on_child_should_succeed() {
    test_assert(AssertAgainst::SelfPackage, true, true);
}

#[test]
fn assert_tx_processor_package_on_child_should_fail() {
    test_assert(AssertAgainst::TransactionProcessorPackage, true, false);
}

#[test]
fn assert_self_blueprint_on_child_should_fail() {
    test_assert(AssertAgainst::SelfBlueprint, true, false);
}

#[test]
fn assert_tx_processor_blueprint_on_child_should_succeed() {
    test_assert(AssertAgainst::TransactionProcessorBlueprint, true, true);
}

#[test]
fn call_component_address_protected_method_in_parent_with_wrong_address_should_fail() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let package_address = test_runner.compile_and_publish("./tests/blueprints/address");
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 10.into())
        .call_function(
            package_address,
            "CalledComponent",
            "create",
            manifest_args!(),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);
    let called_component = receipt.expect_commit(true).new_component_addresses()[0];
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 10.into())
        .call_function(
            package_address,
            "MyComponent",
            "create",
            manifest_args!(called_component),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);
    receipt.expect_commit_success();
    let component = receipt.expect_commit(true).new_component_addresses()[0];

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 10.into())
        .call_method(
            component,
            "call_other_component_with_wrong_address",
            manifest_args!(),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::SystemError(SystemError::AssertAccessRuleFailed)
        )
    });
}

#[test]
fn can_instantiate_with_preallocated_address() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let package_address = test_runner.compile_and_publish("./tests/blueprints/address");
    // Act + Assert
    let manifest = ManifestBuilder::new()
        .call_function(
            package_address,
            "PreallocationComponent",
            "create_with_preallocated_address",
            manifest_args!(),
        )
        .build();
    let receipt = test_runner.execute_manifest_ignoring_fee(manifest, vec![]);
    receipt.expect_commit_success();
}

#[test]
fn errors_if_unused_preallocated_address() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let package_address = test_runner.compile_and_publish("./tests/blueprints/address");

    // Act + Assert 1
    let receipt = test_runner.execute_manifest_ignoring_fee(
        ManifestBuilder::new()
            .call_function(
                package_address,
                "PreallocationComponent",
                "create_with_unused_preallocated_address_1",
                manifest_args!(),
            )
            .build(),
        vec![],
    );
    receipt.expect_commit_failure();

    // Act + Assert 2
    let receipt = test_runner.execute_manifest_ignoring_fee(
        ManifestBuilder::new()
            .call_function(
                package_address,
                "PreallocationComponent",
                "create_with_unused_preallocated_address_2",
                manifest_args!(),
            )
            .build(),
        vec![],
    );
    receipt.expect_commit_failure();
}

#[test]
fn errors_if_assigns_same_address_to_two_components() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let package_address = test_runner.compile_and_publish("./tests/blueprints/address");

    // Act + Assert
    let receipt = test_runner.execute_manifest_ignoring_fee(
        ManifestBuilder::new()
            .call_function(
                package_address,
                "PreallocationComponent",
                "create_two_with_same_address",
                manifest_args!(),
            )
            .build(),
        vec![],
    );
    receipt.expect_commit_failure();
}

#[test]
fn various_preallocated_address_smuggling_scenarios_are_disallowed() {
    // NOTE - all of these scenarios shouldn't be possible - but with upcoming changes, the reason for their failure
    // might change. So I'll just assert that they're a failure without giving reasons.

    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let package_address = test_runner.compile_and_publish("./tests/blueprints/address");

    {
        let unused_address =
            radix_engine_common::types::component_address(EntityType::GlobalGenericComponent, 53);
        test_runner
            .call_function(
                package_address,
                "PreallocationSmugglerComponent",
                "create_empty_at_address",
                manifest_args!(unused_address),
            )
            .expect_not_success();
    }

    {
        let unused_address =
            radix_engine_common::types::component_address(EntityType::GlobalGenericComponent, 53);
        let unused_address_bytes = unused_address.as_node_id().0;
        test_runner
            .call_function(
                package_address,
                "PreallocationSmugglerComponent",
                "create_empty_at_address_bytes",
                manifest_args!(unused_address_bytes),
            )
            .expect_not_success();
    }

    {
        test_runner
            .call_function(
                package_address,
                "PreallocationSmugglerComponent",
                "create_with_smuggled_address",
                manifest_args!(),
            )
            .expect_not_success();
    }

    {
        let empty_smuggler = test_runner.construct_new(
            package_address,
            "PreallocationSmugglerComponent",
            "create_empty",
            manifest_args!(),
        );

        // Currently you're unable to smuggle a new address as the allocator requires the address is used
        test_runner
            .call_method(
                empty_smuggler,
                "allocate_and_smuggle_address",
                manifest_args!(),
            )
            .expect_not_success();
    }

    {
        let unused_address =
            radix_engine_common::types::component_address(EntityType::GlobalGenericComponent, 53);

        let empty_smuggler = test_runner.construct_new(
            package_address,
            "PreallocationSmugglerComponent",
            "create_empty",
            manifest_args!(),
        );

        // Currently you're unable to smuggle this address as the reference is simply invalid
        test_runner
            .call_method(
                empty_smuggler,
                "smuggle_given_address",
                manifest_args!(unused_address),
            )
            .expect_not_success();
    }
}

#[test]
fn system_address_preallocation_smuggling_not_possible() {
    // NOTE - all of these scenarios shouldn't be possible - but with upcoming changes, the reason for their failure
    // might change. So I'll just assert that they're a failure without giving reasons.

    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let package_address = test_runner.compile_and_publish("./tests/blueprints/address");

    {
        let transaction_allocated_address =
            radix_engine_common::types::component_address(EntityType::GlobalGenericComponent, 53);
        let manifest = ManifestBuilder::new()
            .call_function(
                package_address,
                "PreallocationSmugglerComponent",
                "create_empty_at_address",
                manifest_args!(transaction_allocated_address),
            )
            .build();
        // Standard global preallocation not yet allowed because of reference issues
        test_runner
            .execute_system_transaction(
                manifest.instructions,
                btreeset!(transaction_allocated_address.into_node_id()),
            )
            .expect_not_success();
    }

    {
        let transaction_allocated_address =
            radix_engine_common::types::component_address(EntityType::GlobalGenericComponent, 53);
        let manifest = ManifestBuilder::new()
            .call_function(
                package_address,
                "PreallocationSmugglerComponent",
                "create_empty_at_address_bytes",
                manifest_args!(transaction_allocated_address.into_node_id().0),
            )
            .build();
        // Global preallocation with address-as-bytes-smuggling currently allowed
        test_runner
            .execute_system_transaction(
                manifest.instructions,
                btreeset!(transaction_allocated_address.into_node_id()),
            )
            .expect_commit_success();
    }

    {
        let transaction_allocated_address =
            radix_engine_common::types::component_address(EntityType::GlobalGenericComponent, 53);
        let manifest = ManifestBuilder::new()
            .call_function(
                package_address,
                "PreallocationSmugglerComponent",
                "create_with_smuggled_given_address_bytes",
                manifest_args!(transaction_allocated_address.into_node_id().0),
            )
            .build();
        // System transaction global preallocate address smuggling should not be allowed
        test_runner
            .execute_system_transaction(
                manifest.instructions,
                btreeset!(transaction_allocated_address.into_node_id()),
            )
            .expect_not_success();
    }

    {
        let transaction_allocated_address =
            radix_engine_common::types::component_address(EntityType::GlobalGenericComponent, 53);
        let manifest = ManifestBuilder::new()
            .call_function(
                package_address,
                "PreallocationSmugglerComponent",
                "create_with_smuggled_given_address",
                manifest_args!(transaction_allocated_address),
            )
            .build();
        // System transaction global preallocate address smuggling should not be allowed
        test_runner
            .execute_system_transaction(
                manifest.instructions,
                btreeset!(transaction_allocated_address.into_node_id()),
            )
            .expect_not_success();
    }

    {
        let transaction_allocated_address =
            radix_engine_common::types::component_address(EntityType::GlobalGenericComponent, 53);
        let manifest = ManifestBuilder::new()
            .call_function(
                package_address,
                "PreallocationSmugglerComponent",
                "create_empty",
                manifest_args!(),
            )
            .build();
        // We don't use the transaction allocated address - so it should be an error
        test_runner
            .execute_system_transaction(
                manifest.instructions,
                btreeset!(transaction_allocated_address.into_node_id()),
            )
            .expect_not_success();
    }
}
