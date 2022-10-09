use radix_engine::ledger::TypedInMemorySubstateStore;
use radix_engine::types::*;
use scrypto::args;
use scrypto_unit::*;
use transaction::builder::ManifestBuilder;

#[test]
fn test_loop() {
    // Arrange
    let mut store = TypedInMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);

    // Act
    let code = wat2wasm(&include_str!("wasm/loop.wat").replace("${n}", "2000"));
    let package_address = test_runner.publish_package(code, test_abi_any_in_void_out("Test", "f"));
    let manifest = ManifestBuilder::new(&NetworkDefinition::simulator())
        .lock_fee(10.into(), SYS_FAUCET_COMPONENT)
        .call_scrypto_function(package_address, "Test", "f", args!())
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
    let package_address = test_runner.publish_package(code, test_abi_any_in_void_out("Test", "f"));
    let manifest = ManifestBuilder::new(&NetworkDefinition::simulator())
        .lock_fee(45.into(), SYS_FAUCET_COMPONENT)
        .call_scrypto_function(package_address, "Test", "f", args!())
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
    let package_address = test_runner.publish_package(code, test_abi_any_in_void_out("Test", "f"));
    let manifest = ManifestBuilder::new(&NetworkDefinition::simulator())
        .lock_fee(10.into(), SYS_FAUCET_COMPONENT)
        .call_scrypto_function(package_address, "Test", "f", args!())
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
    let package_address = test_runner.publish_package(code, test_abi_any_in_void_out("Test", "f"));
    let manifest = ManifestBuilder::new(&NetworkDefinition::simulator())
        .lock_fee(10.into(), SYS_FAUCET_COMPONENT)
        .call_scrypto_function(package_address, "Test", "f", args!())
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
    let package_address = test_runner.publish_package(code, test_abi_any_in_void_out("Test", "f"));
    let manifest = ManifestBuilder::new(&NetworkDefinition::simulator())
        .lock_fee(10.into(), SYS_FAUCET_COMPONENT)
        .call_scrypto_function(package_address, "Test", "f", args!())
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
    let package_address = test_runner.publish_package(code, test_abi_any_in_void_out("Test", "f"));
    let manifest = ManifestBuilder::new(&NetworkDefinition::simulator())
        .lock_fee(10.into(), SYS_FAUCET_COMPONENT)
        .call_scrypto_function(package_address, "Test", "f", args!())
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
    let manifest = ManifestBuilder::new(&NetworkDefinition::simulator())
        .lock_fee(10u32.into(), account1)
        .withdraw_from_account_by_amount(100u32.into(), RADIX_TOKEN, account1)
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
        + 4400 /* borrow_node */
        + 7500 /* create_node */
        + 1698 /* decode_manifest */
        + 700 /* drop_lock */
        + 1000 /* drop_node */
        + 0 /* instantiate_wasm */
        + 2215 /* invoke_function */
        + 700 /* lock_substate */
        + 100 /* read_owned_nodes */
        + 3500 /* read_substate */
        + 5200 /* run_function */
        + 352580 /* run_wasm */
        + 566 /* verify_manifest */
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
    let manifest = ManifestBuilder::new(&NetworkDefinition::simulator())
        .lock_fee(100.into(), SYS_FAUCET_COMPONENT)
        .publish_package(code, HashMap::new())
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);
    receipt.expect_commit_success();

    // Assert
    assert_eq!(4285593, receipt.execution.fee_summary.cost_unit_consumed);
}

#[test]
fn should_be_able_run_large_manifest() {
    // Arrange
    let mut store = TypedInMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);
    let (public_key, _, account) = test_runner.new_account();

    // Act
    let mut builder = ManifestBuilder::new(&NetworkDefinition::simulator());
    builder.lock_fee(100u32.into(), account);
    builder.withdraw_from_account_by_amount(100u32.into(), RADIX_TOKEN, account);
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
    let (public_key, _, account) = test_runner.new_account();

    // Act
    let mut builder = ManifestBuilder::new(&NetworkDefinition::simulator());
    builder.lock_fee(100u32.into(), account);
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
    let (public_key, _, account) = test_runner.new_account();
    let resource_address = test_runner.create_fungible_resource(100.into(), 0, account);

    // Act
    let mut builder = ManifestBuilder::new(&NetworkDefinition::simulator());
    for _ in 0..5 {
        builder.create_proof_from_account_by_amount(1.into(), resource_address, account);
    }
    builder.lock_fee(100u32.into(), account);
    let manifest = builder.build();
    let receipt = test_runner.execute_manifest(
        manifest,
        vec![NonFungibleAddress::from_public_key(&public_key)],
    );

    // Assert
    receipt.expect_commit_success();
}
