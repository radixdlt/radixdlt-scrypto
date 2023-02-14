use radix_engine::{
    errors::{ModuleError, RuntimeError},
    system::kernel_modules::transaction_limits::TransactionLimitsError,
    types::*,
};
use radix_engine_interface::blueprints::resource::*;
use scrypto_unit::*;
use transaction::builder::ManifestBuilder;

#[test]
fn test_loop() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();

    // Act
    let code = wat2wasm(&include_str!("wasm/loop.wat").replace("${n}", "1000"));
    let package_address = test_runner.publish_package(
        code,
        generate_single_function_abi(
            "Test",
            "f",
            Type::Tuple {
                element_types: vec![],
            },
        ),
        BTreeMap::new(),
        BTreeMap::new(),
        AccessRules::new(),
    );
    let manifest = ManifestBuilder::new()
        .lock_fee(FAUCET_COMPONENT, 10.into())
        .call_function(package_address, "Test", "f", args!())
        .build();
    let receipt = test_runner.execute_manifest_with_cost_unit_limit(manifest, vec![], 15_000_000);

    // Assert
    receipt.expect_commit_success();
}

// TODO: investigate the case where cost_unit_limit < system_loan and transaction runs out of cost units.

#[test]
fn test_loop_out_of_cost_unit() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();

    // Act
    let code = wat2wasm(&include_str!("wasm/loop.wat").replace("${n}", "2000000"));
    let package_address = test_runner.publish_package(
        code,
        generate_single_function_abi(
            "Test",
            "f",
            Type::Tuple {
                element_types: vec![],
            },
        ),
        BTreeMap::new(),
        BTreeMap::new(),
        AccessRules::new(),
    );
    let manifest = ManifestBuilder::new()
        .lock_fee(FAUCET_COMPONENT, 450.into())
        .call_function(package_address, "Test", "f", args!())
        .build();
    let receipt = test_runner.execute_manifest_with_cost_unit_limit(manifest, vec![], 15_000_000);

    // Assert
    receipt.expect_specific_failure(is_costing_error)
}

#[test]
fn test_recursion() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();

    // Act
    // In this test case, each call frame costs 4 stack units
    let code = wat2wasm(&include_str!("wasm/recursion.wat").replace("${n}", "256"));
    let package_address = test_runner.publish_package(
        code,
        generate_single_function_abi(
            "Test",
            "f",
            Type::Tuple {
                element_types: vec![],
            },
        ),
        BTreeMap::new(),
        BTreeMap::new(),
        AccessRules::new(),
    );
    let manifest = ManifestBuilder::new()
        .lock_fee(FAUCET_COMPONENT, 10.into())
        .call_function(package_address, "Test", "f", args!())
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn test_recursion_stack_overflow() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();

    // Act
    let code = wat2wasm(&include_str!("wasm/recursion.wat").replace("${n}", "257"));
    let package_address = test_runner.publish_package(
        code,
        generate_single_function_abi(
            "Test",
            "f",
            Type::Tuple {
                element_types: vec![],
            },
        ),
        BTreeMap::new(),
        BTreeMap::new(),
        AccessRules::new(),
    );
    let manifest = ManifestBuilder::new()
        .lock_fee(FAUCET_COMPONENT, 10.into())
        .call_function(package_address, "Test", "f", args!())
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_specific_failure(is_wasm_error)
}

#[test]
fn test_grow_memory() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();

    // Act
    let code = wat2wasm(&include_str!("wasm/memory.wat").replace("${n}", "40"));
    let package_address = test_runner.publish_package(
        code,
        generate_single_function_abi(
            "Test",
            "f",
            Type::Tuple {
                element_types: vec![],
            },
        ),
        BTreeMap::new(),
        BTreeMap::new(),
        AccessRules::new(),
    );
    let manifest = ManifestBuilder::new()
        .lock_fee(FAUCET_COMPONENT, 10.into())
        .call_function(package_address, "Test", "f", args!())
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn test_grow_memory_out_of_cost_unit() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();

    // Act
    let code = wat2wasm(&include_str!("wasm/memory.wat").replace("${n}", "100000"));
    let package_address = test_runner.publish_package(
        code,
        generate_single_function_abi(
            "Test",
            "f",
            Type::Tuple {
                element_types: vec![],
            },
        ),
        BTreeMap::new(),
        BTreeMap::new(),
        AccessRules::new(),
    );
    let manifest = ManifestBuilder::new()
        .lock_fee(FAUCET_COMPONENT, 10.into())
        .call_function(package_address, "Test", "f", args!())
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_specific_failure(is_costing_error)
}

#[test]
fn test_max_instance_memory_exceeded() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();

    // Act
    let code = wat2wasm(&include_str!("wasm/memory.wat").replace("${n}", "1000"));
    let package_address = test_runner.publish_package(
        code,
        generate_single_function_abi(
            "Test",
            "f",
            Type::Tuple {
                element_types: vec![],
            },
        ),
        BTreeMap::new(),
        BTreeMap::new(),
        AccessRules::new(),
    );
    let manifest = ManifestBuilder::new()
        .lock_fee(FAUCET_COMPONENT, 10.into())
        .call_function(package_address, "Test", "f", args!())
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::ModuleError(ModuleError::TransactionLimitsError(
                TransactionLimitsError::MaxWasmInstanceMemoryExceeded(_)
            ))
        )
    })
}

#[test]
fn test_max_transaction_memory_exceeded() {
    // Test recursive instantiation of WASM with memory allocation of 1 MiB
    // for each call (+additional memory for transaction execution).

    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let package_address = test_runner.compile_and_publish("tests/blueprints/recursion");

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee(FAUCET_COMPONENT, 10.into())
        .call_function(
            package_address,
            "Caller",
            "recursive_with_memory",
            args!(8u32, 1024 * 1024 as usize),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::ModuleError(ModuleError::TransactionLimitsError(
                TransactionLimitsError::MaxWasmMemoryExceeded(_)
            ))
        )
    })
}
