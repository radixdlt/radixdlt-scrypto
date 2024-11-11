use radix_common::prelude::*;
use radix_engine::errors::{KernelError, RuntimeError, SystemError};
use radix_engine_interface::blueprints::account::ACCOUNT_BLUEPRINT;
use radix_engine_interface::blueprints::transaction_processor::TRANSACTION_PROCESSOR_BLUEPRINT;
use radix_engine_tests::common::*;
use radix_substate_store_queries::typed_substate_layout::PACKAGE_BLUEPRINT;
use scrypto_test::prelude::*;

#[test]
fn get_global_address_in_local_in_function_should_fail() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let package_address = ledger.publish_package_simple(PackageLoader::get("address"));
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package_address,
            "CalledComponent",
            "create",
            manifest_args!(),
        )
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![]);
    let called_component = receipt.expect_commit(true).new_component_addresses()[0];

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package_address,
            "MyComponent",
            "get_global_address_in_local",
            manifest_args!(called_component),
        )
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![]);

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
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let package_address = ledger.publish_package_simple(PackageLoader::get("address"));
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package_address,
            "CalledComponent",
            "create",
            manifest_args!(),
        )
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![]);
    let called_component = receipt.expect_commit(true).new_component_addresses()[0];
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package_address,
            "MyComponent",
            "create",
            manifest_args!(called_component),
        )
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![]);
    receipt.expect_commit_success();
    let component = receipt.expect_commit(true).new_component_addresses()[0];

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_method(
            component,
            "get_global_address_in_local_of_parent_method",
            manifest_args!(called_component),
        )
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![]);

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
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let package_address = ledger.publish_package_simple(PackageLoader::get("address"));
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package_address,
            "CalledComponent",
            "create",
            manifest_args!(),
        )
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![]);
    let called_component = receipt.expect_commit(true).new_component_addresses()[0];
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package_address,
            "MyComponent",
            "create",
            manifest_args!(called_component),
        )
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![]);
    receipt.expect_commit_success();
    let component = receipt.expect_commit(true).new_component_addresses()[0];

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_method(component, "get_global_address_in_parent", manifest_args!())
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![]);
    let get_global_address_component: ComponentAddress = receipt.expect_commit(true).output(1);

    // Assert
    receipt.expect_commit_success();
    assert_eq!(component, get_global_address_component)
}

#[test]
fn get_global_address_in_child_should_succeed() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let package_address = ledger.publish_package_simple(PackageLoader::get("address"));
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package_address,
            "CalledComponent",
            "create",
            manifest_args!(),
        )
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![]);
    let called_component = receipt.expect_commit(true).new_component_addresses()[0];
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package_address,
            "MyComponent",
            "create",
            manifest_args!(called_component),
        )
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![]);
    receipt.expect_commit_success();
    let component = receipt.expect_commit(true).new_component_addresses()[0];

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_method(component, "get_global_address_in_owned", manifest_args!())
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![]);
    let get_global_address_component: ComponentAddress = receipt.expect_commit(true).output(1);

    // Assert
    receipt.expect_commit_success();
    assert_eq!(component, get_global_address_component)
}

fn test_call_component_address_protected_method(caller_child: bool, callee_child: bool) {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let package_address = ledger.publish_package_simple(PackageLoader::get("address"));
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package_address,
            "CalledComponent",
            "create",
            manifest_args!(),
        )
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![]);
    let called_component = receipt.expect_commit(true).new_component_addresses()[0];
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package_address,
            "MyComponent",
            "create",
            manifest_args!(called_component),
        )
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![]);
    receipt.expect_commit_success();
    let component = receipt.expect_commit(true).new_component_addresses()[0];

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_method(
            component,
            "call_other_component",
            manifest_args!(caller_child, callee_child),
        )
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![]);

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
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let package_address = ledger.publish_package_simple(PackageLoader::get("address"));
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package_address,
            "CalledComponent",
            "create",
            manifest_args!(),
        )
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![]);
    let called_component = receipt.expect_commit(true).new_component_addresses()[0];
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package_address,
            "MyComponent",
            "create",
            manifest_args!(called_component),
        )
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![]);
    receipt.expect_commit_success();
    let component = receipt.expect_commit(true).new_component_addresses()[0];

    let (method_name, args) = match package {
        AssertAgainst::SelfPackage => (
            "assert_check_on_package",
            manifest_args!(package_address, child),
        ),
        AssertAgainst::SelfBlueprint => {
            let blueprint = BlueprintId::new(&package_address, "MyComponent");
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
            let blueprint = BlueprintId::new(
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
        .lock_fee_from_faucet()
        .call_method(component, method_name, args)
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![]);

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

/// Package actor badge will be different depending on whether the callee is global or internal
mod package_actor_badge {
    use super::{test_assert, AssertAgainst};

    #[test]
    fn assert_self_package_in_global_callee_should_fail() {
        test_assert(AssertAgainst::SelfPackage, false, false);
    }

    #[test]
    fn assert_self_package_in_internal_callee_should_succeed() {
        test_assert(AssertAgainst::SelfPackage, true, true);
    }

    #[test]
    fn assert_tx_processor_package_in_global_callee_should_succeed() {
        test_assert(AssertAgainst::TransactionProcessorPackage, false, true);
    }

    #[test]
    fn assert_tx_processor_package_in_internal_callee_should_fail() {
        test_assert(AssertAgainst::TransactionProcessorPackage, true, false);
    }
}

/// Global caller results should be the same whether the callee is global or internal
mod global_caller_actor_badge {
    use super::{test_assert, AssertAgainst};

    #[test]
    fn assert_self_blueprint_global_caller_in_global_callee_should_fail() {
        test_assert(AssertAgainst::SelfBlueprint, false, false);
    }

    #[test]
    fn assert_self_blueprint_global_caller_in_internal_callee_should_fail() {
        test_assert(AssertAgainst::SelfBlueprint, true, false);
    }

    #[test]
    fn assert_tx_processor_blueprint_global_caller_in_global_callee_should_succeed() {
        test_assert(AssertAgainst::TransactionProcessorBlueprint, false, true);
    }

    #[test]
    fn assert_tx_processor_blueprint_global_caller_in_internal_callee_should_succeed() {
        test_assert(AssertAgainst::TransactionProcessorBlueprint, true, true);
    }
}

#[test]
fn call_component_address_protected_method_in_parent_with_wrong_address_should_fail() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let package_address = ledger.publish_package_simple(PackageLoader::get("address"));
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package_address,
            "CalledComponent",
            "create",
            manifest_args!(),
        )
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![]);
    let called_component = receipt.expect_commit(true).new_component_addresses()[0];
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package_address,
            "MyComponent",
            "create",
            manifest_args!(called_component),
        )
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![]);
    receipt.expect_commit_success();
    let component = receipt.expect_commit(true).new_component_addresses()[0];

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_method(
            component,
            "call_other_component_with_wrong_address",
            manifest_args!(),
        )
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![]);

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
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let package_address = ledger.publish_package_simple(PackageLoader::get("address"));
    // Act + Assert
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package_address,
            "PreallocationComponent",
            "create_with_preallocated_address",
            manifest_args!(),
        )
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![]);
    receipt.expect_commit_success();
}

#[test]
fn errors_if_unused_preallocated_address() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let package_address = ledger.publish_package_simple(PackageLoader::get("address"));

    // Act + Assert 1
    let receipt = ledger.execute_manifest(
        ManifestBuilder::new()
            .lock_fee_from_faucet()
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
    let receipt = ledger.execute_manifest(
        ManifestBuilder::new()
            .lock_fee_from_faucet()
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
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let package_address = ledger.publish_package_simple(PackageLoader::get("address"));

    // Act + Assert
    let receipt = ledger.execute_manifest(
        ManifestBuilder::new()
            .lock_fee_from_faucet()
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
fn test_pass_named_global_addresses() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (public_key, _, _) = ledger.new_allocated_account();
    let package_address = ledger.publish_package_simple(PackageLoader::get("address"));

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .allocate_global_address(
            ACCOUNT_PACKAGE,
            ACCOUNT_BLUEPRINT,
            "account_address_reservation",
            "account_address",
        )
        .allocate_global_address(
            PACKAGE_PACKAGE,
            PACKAGE_BLUEPRINT,
            "package_address_reservation",
            "package_address",
        )
        .allocate_global_address(
            package_address,
            "Garbage",
            "component_address_reservation",
            "component_address",
        )
        .allocate_global_address(
            RESOURCE_PACKAGE,
            FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT,
            "resource_address_reservation",
            "resource_address",
        )
        .call_function_with_name_lookup(
            package_address,
            "ManifestGlobalAddresses",
            "accept_global_addresses",
            |lookup| {
                (
                    lookup.named_address("account_address"),
                    lookup.named_address("package_address"),
                    lookup.named_address("component_address"),
                    lookup.named_address("resource_address"),
                )
            },
        )
        .build_no_validate();
    let receipt = ledger.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    // Assert
    receipt.expect_specific_failure(|e| match e {
        // 4 address reservations are left
        RuntimeError::KernelError(KernelError::OrphanedNodes(nodes)) if nodes.len() == 4 => true,
        _ => false,
    })
}

#[test]
fn test_pass_static_global_addresses() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (public_key, _, _) = ledger.new_allocated_account();
    let package_address = ledger.publish_package_simple(PackageLoader::get("address"));

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package_address,
            "ManifestGlobalAddresses",
            "accept_global_addresses",
            manifest_args!(
                ManifestGlobalAddress::Static(FAUCET_COMPONENT.into()),
                ManifestPackageAddress::Static(RESOURCE_PACKAGE),
                ManifestComponentAddress::Static(CONSENSUS_MANAGER),
                ManifestResourceAddress::Static(XRD)
            ),
        )
        .build();
    let receipt = ledger.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    // Assert
    receipt.expect_commit_success();
}
