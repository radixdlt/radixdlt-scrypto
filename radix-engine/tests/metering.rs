use radix_engine::ledger::TypedInMemorySubstateStore;
use radix_engine::wasm::InvokeError;
use scrypto::args;
use scrypto::core::NetworkDefinition;
use scrypto::prelude::{Package, RADIX_TOKEN, SYSTEM_COMPONENT};
use scrypto_unit::*;
use transaction::builder::ManifestBuilder;

#[test]
fn test_loop() {
    // Arrange
    let mut store = TypedInMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);

    // Act
    let code = wat2wasm(&include_str!("wasm/loop.wat").replace("${n}", "2000"));
    let package = Package {
        code,
        blueprints: abi_single_fn_any_input_void_output("Test", "f"),
    };
    let package_address = test_runner.publish_package(package);
    let manifest = ManifestBuilder::new(NetworkDefinition::local_simulator())
        .lock_fee(10.into(), SYSTEM_COMPONENT)
        .call_function(package_address, "Test", "f", args!())
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_success();
}

#[test]
fn test_loop_out_of_cost_unit() {
    // Arrange
    let mut store = TypedInMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);

    // Act
    let code = wat2wasm(&include_str!("wasm/loop.wat").replace("${n}", "7000000"));
    let package = Package {
        code,
        blueprints: abi_single_fn_any_input_void_output("Test", "f"),
    };
    let package_address = test_runner.publish_package(package);
    let manifest = ManifestBuilder::new(NetworkDefinition::local_simulator())
        .lock_fee(45.into(), SYSTEM_COMPONENT)
        .call_function(package_address, "Test", "f", args!())
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    expect_invoke_error(&receipt, |err| matches!(err, InvokeError::CostingError(..)));
}

#[test]
fn test_recursion() {
    // Arrange
    let mut store = TypedInMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);

    // Act
    // In this test case, each call frame costs 4 stack units
    let code = wat2wasm(&include_str!("wasm/recursion.wat").replace("${n}", "128"));
    let package = Package {
        code,
        blueprints: abi_single_fn_any_input_void_output("Test", "f"),
    };
    let package_address = test_runner.publish_package(package);
    let manifest = ManifestBuilder::new(NetworkDefinition::local_simulator())
        .lock_fee(10.into(), SYSTEM_COMPONENT)
        .call_function(package_address, "Test", "f", args!())
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_success();
}

#[test]
fn test_recursion_stack_overflow() {
    // Arrange
    let mut store = TypedInMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);

    // Act
    let code = wat2wasm(&include_str!("wasm/recursion.wat").replace("${n}", "129"));
    let package = Package {
        code,
        blueprints: abi_single_fn_any_input_void_output("Test", "f"),
    };
    let package_address = test_runner.publish_package(package);
    let manifest = ManifestBuilder::new(NetworkDefinition::local_simulator())
        .lock_fee(10.into(), SYSTEM_COMPONENT)
        .call_function(package_address, "Test", "f", args!())
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    expect_invoke_error(&receipt, |err| matches!(err, InvokeError::WasmError(..)));
}

#[test]
fn test_grow_memory() {
    // Arrange
    let mut store = TypedInMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);

    // Act
    let code = wat2wasm(&include_str!("wasm/memory.wat").replace("${n}", "100"));
    let package = Package {
        code,
        blueprints: abi_single_fn_any_input_void_output("Test", "f"),
    };
    let package_address = test_runner.publish_package(package);
    let manifest = ManifestBuilder::new(NetworkDefinition::local_simulator())
        .lock_fee(10.into(), SYSTEM_COMPONENT)
        .call_function(package_address, "Test", "f", args!())
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_success();
}

#[test]
fn test_grow_memory_out_of_cost_unit() {
    // Arrange
    let mut store = TypedInMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);

    // Act
    let code = wat2wasm(&include_str!("wasm/memory.wat").replace("${n}", "100000"));
    let package = Package {
        code,
        blueprints: abi_single_fn_any_input_void_output("Test", "f"),
    };
    let package_address = test_runner.publish_package(package);
    let manifest = ManifestBuilder::new(NetworkDefinition::local_simulator())
        .lock_fee(10.into(), SYSTEM_COMPONENT)
        .call_function(package_address, "Test", "f", args!())
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    expect_invoke_error(&receipt, |err| matches!(err, InvokeError::CostingError(..)));
}

#[test]
fn test_basic_transfer() {
    // Arrange
    let mut store = TypedInMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);
    let (public_key1, _, account1) = test_runner.new_account();
    let (_, _, account2) = test_runner.new_account();

    // Act
    let manifest = ManifestBuilder::new(NetworkDefinition::local_simulator())
        .lock_fee(10.into(), account1)
        .withdraw_from_account_by_amount(100.into(), RADIX_TOKEN, account1)
        .call_method_with_all_resources(account2, "deposit_batch")
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![public_key1]);

    // Assert

    // NOTE: If this test fails, it should print out the actual fee table in the error logs.
    // Or you can run just this test with the below:
    // (cd radix-engine && cargo test test_basic_transfer -- --exact)
    assert_eq!(
        10000 /* base_fee */
        + 3300 /* borrow_substate */
        + 1500 /* create_node */
        + 1938 /* decode_transaction */
        + 1000 /* drop_node */
        + 603822 /* instantiate_wasm */
        + 1895 /* invoke_function */
        + 2215 /* invoke_method */
        + 5000 /* read_substate */
        + 600 /* return_substate */
        + 1000 /* run_function */
        + 5200 /* run_method */
        + 262304 /* run_wasm */
        + 646 /* verify_manifest */
        + 3750 /* verify_signatures */
        + 3000, /* write_substate */
        receipt.expect_executed().fee_summary.cost_unit_consumed
    );
}
