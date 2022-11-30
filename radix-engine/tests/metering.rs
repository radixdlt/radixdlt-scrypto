use radix_engine::ledger::TypedInMemorySubstateStore;
use radix_engine::types::*;
use radix_engine_interface::core::NetworkDefinition;
use radix_engine_interface::data::*;
use radix_engine_interface::model::FromPublicKey;
use scrypto_unit::*;
use transaction::builder::ManifestBuilder;

#[test]
fn test_loop() {
    // Arrange
    let mut store = TypedInMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);

    // Act
    let code = wat2wasm(&include_str!("wasm/loop.wat").replace("${n}", "2000"));
    let package_address =
        test_runner.publish_package(code, generate_single_function_abi("Test", "f", Type::Unit));
    let manifest = ManifestBuilder::new(&NetworkDefinition::simulator())
        .lock_fee(FAUCET_COMPONENT, 10.into())
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
    let code = wat2wasm(&include_str!("wasm/loop.wat").replace("${n}", "70000000"));
    let package_address =
        test_runner.publish_package(code, generate_single_function_abi("Test", "f", Type::Unit));
    let manifest = ManifestBuilder::new(&NetworkDefinition::simulator())
        .lock_fee(FAUCET_COMPONENT, 45.into())
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
    let code = wat2wasm(&include_str!("wasm/recursion.wat").replace("${n}", "256"));
    let package_address =
        test_runner.publish_package(code, generate_single_function_abi("Test", "f", Type::Unit));
    let manifest = ManifestBuilder::new(&NetworkDefinition::simulator())
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
    let mut store = TypedInMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);

    // Act
    let code = wat2wasm(&include_str!("wasm/recursion.wat").replace("${n}", "257"));
    let package_address =
        test_runner.publish_package(code, generate_single_function_abi("Test", "f", Type::Unit));
    let manifest = ManifestBuilder::new(&NetworkDefinition::simulator())
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
    let mut store = TypedInMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);

    // Act
    let code = wat2wasm(&include_str!("wasm/memory.wat").replace("${n}", "100"));
    let package_address =
        test_runner.publish_package(code, generate_single_function_abi("Test", "f", Type::Unit));
    let manifest = ManifestBuilder::new(&NetworkDefinition::simulator())
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
    let mut store = TypedInMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);

    // Act
    let code = wat2wasm(&include_str!("wasm/memory.wat").replace("${n}", "100000"));
    let package_address =
        test_runner.publish_package(code, generate_single_function_abi("Test", "f", Type::Unit));
    let manifest = ManifestBuilder::new(&NetworkDefinition::simulator())
        .lock_fee(FAUCET_COMPONENT, 10.into())
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
    let (public_key1, _, account1) = test_runner.new_allocated_account();
    let (_, _, account2) = test_runner.new_allocated_account();

    // Act
    let manifest = ManifestBuilder::new(&NetworkDefinition::simulator())
        .lock_fee(account1, 10u32.into())
        .withdraw_from_account_by_amount(account1, 100u32.into(), RADIX_TOKEN)
        .call_method(
            account2,
            "deposit_batch",
            args!(Expression::entire_worktop()),
        )
        .build();
    let receipt = test_runner.execute_manifest(
        manifest,
        vec![NonFungibleAddress::from_public_key(&public_key1)],
    );
    receipt.expect_commit_success();

    // Assert
    // NOTE: If this test fails, it should print out the actual fee table in the error logs.
    // Or you can run just this test with the below:
    // (cd radix-engine && cargo test --test metering -- test_basic_transfer)
    assert_eq!(
        10000 /* base_fee */
        + 0 /* blobs */
        + 2000 /* create_node */
        + 990 /* decode_manifest */
        + 6600 /* drop_lock */
        + 2000 /* drop_node */
        + 800  /* emit_event */
        + 0 /* instantiate_wasm */
        + 1460 /* invoke */
        + 8400 /* lock_substate */
        + 3000 /* read_owned_nodes */
        + 28500 /* read_substate */
        + 1000 /* run_native_function */
        + 2200 /* run_native_method */
        + 323082 /* run_wasm */
        + 330 /* verify_manifest */
        + 3750 /* verify_signatures */
        + 17000, /* write_substate */
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
    let manifest = ManifestBuilder::new(&NetworkDefinition::simulator())
        .lock_fee(FAUCET_COMPONENT, 100.into())
        .publish_package_no_owner(code, HashMap::new())
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);
    receipt.expect_commit_success();

    // Assert
    assert!(
        receipt.execution.fee_summary.cost_unit_consumed > 4000000
            && receipt.execution.fee_summary.cost_unit_consumed < 5000000
    );
}

#[test]
fn should_be_able_run_large_manifest() {
    // Arrange
    let mut store = TypedInMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);
    let (public_key, _, account) = test_runner.new_allocated_account();

    // Act
    let mut builder = ManifestBuilder::new(&NetworkDefinition::simulator());
    builder.lock_fee(account, 100u32.into());
    builder.withdraw_from_account_by_amount(account, 100u32.into(), RADIX_TOKEN);
    for _ in 0..500 {
        builder.take_from_worktop_by_amount(1.into(), RADIX_TOKEN, |builder, bid| {
            builder.return_to_worktop(bid)
        });
    }
    builder.call_method(
        account,
        "deposit_batch",
        args!(Expression::entire_worktop()),
    );
    let manifest = builder.build();
    let receipt = test_runner.execute_manifest(
        manifest,
        vec![NonFungibleAddress::from_public_key(&public_key)],
    );

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn should_be_able_invoke_account_balance_50_times() {
    // Arrange
    let mut store = TypedInMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);
    let (public_key, _, account) = test_runner.new_allocated_account();

    // Act
    let mut builder = ManifestBuilder::new(&NetworkDefinition::simulator());
    builder.lock_fee(account, 100u32.into());
    for _ in 0..50 {
        builder.call_method(account, "balance", args!(RADIX_TOKEN));
    }
    let manifest = builder.build();
    let receipt = test_runner.execute_manifest(
        manifest,
        vec![NonFungibleAddress::from_public_key(&public_key)],
    );

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn should_be_able_to_generate_5_proofs_and_then_lock_fee() {
    // Arrange
    let mut store = TypedInMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);
    let (public_key, _, account) = test_runner.new_allocated_account();
    let resource_address = test_runner.create_fungible_resource(100.into(), 0, account);

    // Act
    let mut builder = ManifestBuilder::new(&NetworkDefinition::simulator());
    for _ in 0..5 {
        builder.create_proof_from_account_by_amount(account, 1.into(), resource_address);
    }
    builder.lock_fee(account, 100u32.into());
    let manifest = builder.build();
    let receipt = test_runner.execute_manifest(
        manifest,
        vec![NonFungibleAddress::from_public_key(&public_key)],
    );

    // Assert
    receipt.expect_commit_success();
}
