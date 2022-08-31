use radix_engine::ledger::TypedInMemorySubstateStore;
use radix_engine::types::*;
use scrypto::args;
use scrypto::core::NetworkDefinition;
use scrypto::prelude::{Expression, Package, RADIX_TOKEN, SYS_FAUCET_COMPONENT};
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
    let manifest = ManifestBuilder::new(&NetworkDefinition::local_simulator())
        .lock_fee(10.into(), SYS_FAUCET_COMPONENT)
        .call_function(package_address, "Test", "f", args!())
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_commit_success();
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
    let manifest = ManifestBuilder::new(&NetworkDefinition::local_simulator())
        .lock_fee(45.into(), SYS_FAUCET_COMPONENT)
        .call_function(package_address, "Test", "f", args!())
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_specific_failure(is_costing_error)
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
    let manifest = ManifestBuilder::new(&NetworkDefinition::local_simulator())
        .lock_fee(10.into(), SYS_FAUCET_COMPONENT)
        .call_function(package_address, "Test", "f", args!())
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_commit_success();
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
    let manifest = ManifestBuilder::new(&NetworkDefinition::local_simulator())
        .lock_fee(10.into(), SYS_FAUCET_COMPONENT)
        .call_function(package_address, "Test", "f", args!())
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_specific_failure(is_wasm_error)
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
    let manifest = ManifestBuilder::new(&NetworkDefinition::local_simulator())
        .lock_fee(10.into(), SYS_FAUCET_COMPONENT)
        .call_function(package_address, "Test", "f", args!())
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_commit_success();
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
    let manifest = ManifestBuilder::new(&NetworkDefinition::local_simulator())
        .lock_fee(10.into(), SYS_FAUCET_COMPONENT)
        .call_function(package_address, "Test", "f", args!())
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_specific_failure(is_costing_error)
}

#[test]
fn test_basic_transfer() {
    // Arrange
    let mut store = TypedInMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);
    let (public_key1, _, account1) = test_runner.new_account();
    let (_, _, account2) = test_runner.new_account();

    // Act
    let manifest = ManifestBuilder::new(&NetworkDefinition::local_simulator())
        .lock_fee(10.into(), account1)
        .withdraw_from_account_by_amount(100.into(), RADIX_TOKEN, account1)
        .call_method(
            account2,
            "deposit_batch",
            args!(Expression::entire_worktop()),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![public_key1]);
    receipt.expect_commit_success();

    // Assert

    // NOTE: If this test fails, it should print out the actual fee table in the error logs.
    // Or you can run just this test with the below:
    // (cd radix-engine && cargo test --test metering -- test_basic_transfer)
    assert_eq!(
        10000 /* base_fee */
        + 0 /* blobs */
        + 3300 /* borrow_substate */
        + 1500 /* create_node */
        + 1137 /* decode_manifest */
        + 1000 /* drop_node */
        + 616107 /* instantiate_wasm */
        + 1965 /* invoke_function */
        + 2215 /* invoke_method */
        + 5000 /* read_substate */
        + 600 /* return_substate */
        + 1000 /* run_function */
        + 5200 /* run_method */
        + 275043 /* run_wasm */
        + 379 /* verify_manifest */
        + 3750 /* verify_signatures */
        + 3000, /* write_substate */
        receipt.execution.fee_summary.cost_unit_consumed
    );
}

#[test]
fn test_publish_large_package() {
    // Arrange
    let mut store = TypedInMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);

    // Act
    let code = wat2wasm(&format!(
        r#"
            (module
                (data (i32.const 0) "{}")
                (memory $0 64)
                (export "memory" (memory $0))
            )
        "#,
        "i".repeat(4 * 1024 * 1024)
    ));
    assert_eq!(4194343, code.len());
    let package = Package {
        code,
        blueprints: HashMap::new(),
    };
    let manifest = ManifestBuilder::new(&NetworkDefinition::local_simulator())
        .lock_fee(100.into(), SYS_FAUCET_COMPONENT)
        .publish_package(package)
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);
    receipt.expect_commit_success();

    // Assert
    assert_eq!(4394312, receipt.execution.fee_summary.cost_unit_consumed);
}
