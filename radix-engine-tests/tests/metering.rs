use radix_engine::transaction::TransactionReceipt;
use radix_engine::types::*;
use radix_engine_constants::DEFAULT_MAX_INVOKE_INPUT_SIZE;
use radix_engine_interface::blueprints::package::PackageDefinition;
use scrypto_unit::*;
use transaction::builder::*;
use utils::ContextualDisplay;

// For WASM-specific metering tests, see `wasm_metering.rs`.

#[cfg(feature = "std")]
fn execute_with_time_logging(
    test_runner: &mut TestRunner,
    manifest: TransactionManifestV1,
    proofs: Vec<NonFungibleGlobalId>,
) -> (TransactionReceipt, u32) {
    let start = std::time::Instant::now();
    let receipt = test_runner.execute_manifest(manifest, proofs);
    let duration = start.elapsed();
    println!(
        "Time elapsed is: {:?} - NOTE: this is a very bad measure. Use benchmarks instead.",
        duration
    );
    (receipt, duration.as_millis().try_into().unwrap())
}

#[cfg(feature = "alloc")]
fn execute_with_time_logging(
    test_runner: &mut TestRunner,
    manifest: TransactionManifestV1,
    proofs: Vec<NonFungibleGlobalId>,
) -> (TransactionReceipt, u32) {
    let receipt = test_runner.execute_manifest(manifest, proofs);
    (receipt, 0)
}

#[test]
fn test_basic_transfer() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let (public_key1, _, account1) = test_runner.new_allocated_account();
    let (_, _, account2) = test_runner.new_allocated_account();

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee(account1, 10u32.into())
        .withdraw_from_account(account1, RADIX_TOKEN, 100u32.into())
        .call_method(
            account2,
            "try_deposit_batch_or_abort",
            manifest_args!(ManifestExpression::EntireWorktop),
        )
        .build();

    let (receipt, _) = execute_with_time_logging(
        &mut test_runner,
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key1)],
    );
    let commit_result = receipt.expect_commit(true);

    // Assert
    // NOTE: If this test fails, it should print out the actual fee table in the error logs.
    // Or you can run just this test with the below:
    // cargo test -p radix-engine-tests --test metering -- test_basic_transfer
    assert_eq!(
        commit_result.fee_summary.execution_cost_sum,
        0
        + 897 /* AllocateNodeId */
        + 1417 /* CreateNode */
        + 5476 /* DropLock */
        + 1365 /* DropNode */
        + 735425 /* Invoke */
        + 336814 /* LockSubstate */
        + 8344 /* ReadSubstate */
        + 57500 /* RunNative */
        + 75000 /* RunSystem */
        + 50000 /* TxBaseCost */
        + 1345 /* TxPayloadCost */
        + 100000 /* TxSignatureVerification */
        + 697 /* WriteSubstate */
    );
}

#[test]
fn test_radiswap() {
    let mut test_runner = TestRunner::builder().build();

    // Scrypto developer
    let (pk1, _, _) = test_runner.new_allocated_account();
    // Radiswap operator
    let (pk2, _, account2) = test_runner.new_allocated_account();
    // Radiswap user
    let (pk3, _, account3) = test_runner.new_allocated_account();

    // Publish package
    let package_address = test_runner.publish_package(
        include_bytes!("../../assets/radiswap.wasm").to_vec(),
        manifest_decode(include_bytes!("../../assets/radiswap.schema")).unwrap(),
        btreemap!(),
        OwnerRole::Fixed(rule!(require(NonFungibleGlobalId::from_public_key(&pk1)))),
    );

    // Instantiate Radiswap
    let btc = test_runner.create_fungible_resource(1_000_000.into(), 18, account2);
    let eth = test_runner.create_fungible_resource(1_000_000.into(), 18, account2);
    let component_address: ComponentAddress = test_runner
        .execute_manifest(
            ManifestBuilder::new()
                .lock_fee(account2, 10u32.into())
                .call_function(package_address, "Radiswap", "new", manifest_args!(btc, eth))
                .call_method(
                    account2,
                    "try_deposit_batch_or_abort",
                    manifest_args!(ManifestExpression::EntireWorktop),
                )
                .build(),
            vec![NonFungibleGlobalId::from_public_key(&pk2)],
        )
        .expect_commit(true)
        .output(1);

    // Contributing an initial amount to radiswap
    let btc_init_amount = Decimal::from(500_000);
    let eth_init_amount = Decimal::from(300_000);
    test_runner
        .execute_manifest(
            ManifestBuilder::new()
                .lock_fee(account2, 10u32.into())
                .withdraw_from_account(account2, btc, btc_init_amount)
                .withdraw_from_account(account2, eth, eth_init_amount)
                .take_all_from_worktop(btc, |builder, bucket1| {
                    builder.take_all_from_worktop(eth, |builder, bucket2| {
                        builder.call_method(
                            component_address,
                            "add_liquidity",
                            manifest_args!(bucket1, bucket2),
                        )
                    })
                })
                .call_method(
                    account2,
                    "try_deposit_batch_or_abort",
                    manifest_args!(ManifestExpression::EntireWorktop),
                )
                .build(),
            vec![NonFungibleGlobalId::from_public_key(&pk2)],
        )
        .expect_commit(true);

    // Transfer `10,000 BTC` from `account2` to `account3`
    let btc_amount = Decimal::from(10_000);
    test_runner
        .execute_manifest(
            ManifestBuilder::new()
                .lock_fee(account2, 10u32.into())
                .withdraw_from_account(account2, btc, btc_amount)
                .call_method(
                    account3,
                    "try_deposit_batch_or_abort",
                    manifest_args!(ManifestExpression::EntireWorktop),
                )
                .build(),
            vec![NonFungibleGlobalId::from_public_key(&pk2)],
        )
        .expect_commit_success();
    assert_eq!(test_runner.account_balance(account3, btc), Some(btc_amount));

    // Swap 2,000 BTC into ETH
    let btc_to_swap = Decimal::from(2000);
    let receipt = test_runner.execute_manifest(
        ManifestBuilder::new()
            .lock_fee(account3, 10u32.into())
            .withdraw_from_account(account3, btc, btc_to_swap)
            .take_all_from_worktop(btc, |builder, bucket| {
                builder.call_method(component_address, "swap", manifest_args!(bucket))
            })
            .call_method(
                account3,
                "try_deposit_batch_or_abort",
                manifest_args!(ManifestExpression::EntireWorktop),
            )
            .build(),
        vec![NonFungibleGlobalId::from_public_key(&pk3)],
    );
    let remaining_btc = test_runner.account_balance(account3, btc).unwrap();
    let eth_received = test_runner.account_balance(account3, eth).unwrap();
    assert_eq!(remaining_btc, btc_amount - btc_to_swap);
    assert_eq!(eth_received, dec!("1195.219123505976095617"));
    let commit_result = receipt.expect_commit(true);

    // NOTE: If this test fails, it should print out the actual fee table in the error logs.
    // Or you can run just this test with the below:
    // cargo test -p radix-engine-tests --test metering -- test_radiswap
    assert_eq!(
        commit_result.fee_summary.execution_cost_sum,
        0
        + 2070 /* AllocateNodeId */
        + 3281 /* CreateNode */
        + 12765 /* DropLock */
        + 3045 /* DropNode */
        + 3107898 /* Invoke */
        + 4028062 /* LockSubstate */
        + 19376 /* ReadSubstate */
        + 122500 /* RunNative */
        + 200000 /* RunSystem */
        + 602350 /* RunWasm */
        + 50000 /* TxBaseCost */
        + 1765 /* TxPayloadCost */
        + 100000 /* TxSignatureVerification */
        + 2056 /* WriteSubstate */
    );

    assert_eq!(
        commit_result.fee_summary.total_execution_cost_xrd,
        dec!("0.8255168"),
    );
    assert_eq!(commit_result.fee_summary.total_royalty_cost_xrd, dec!("2"));
}

#[test]
fn test_flash_loan() {
    let mut test_runner = TestRunner::builder().build();

    // Scrypto developer
    let (pk1, _, _) = test_runner.new_allocated_account();
    // Flash loan operator
    let (pk2, _, account2) = test_runner.new_allocated_account();
    // Flash loan user
    let (pk3, _, account3) = test_runner.new_allocated_account();

    // Publish package
    let package_address = test_runner.publish_package(
        include_bytes!("../../assets/flash_loan.wasm").to_vec(),
        manifest_decode(include_bytes!("../../assets/flash_loan.schema")).unwrap(),
        btreemap!(),
        OwnerRole::Fixed(rule!(require(NonFungibleGlobalId::from_public_key(&pk1)))),
    );

    // Instantiate flash_loan
    let xrd_init_amount = Decimal::from(100);
    let (component_address, promise_token_address) = test_runner
        .execute_manifest(
            ManifestBuilder::new()
                .lock_fee(account2, 10u32.into())
                .withdraw_from_account(account2, RADIX_TOKEN, xrd_init_amount)
                .take_all_from_worktop(RADIX_TOKEN, |builder, bucket1| {
                    builder.call_function(
                        package_address,
                        "BasicFlashLoan",
                        "instantiate_default",
                        manifest_args!(bucket1),
                    )
                })
                .call_method(
                    account2,
                    "try_deposit_batch_or_abort",
                    manifest_args!(ManifestExpression::EntireWorktop),
                )
                .build(),
            vec![NonFungibleGlobalId::from_public_key(&pk2)],
        )
        .expect_commit(true)
        .output::<(ComponentAddress, ResourceAddress)>(3);

    // Take loan
    let loan_amount = Decimal::from(50);
    let repay_amount = loan_amount * dec!("1.001");
    let old_balance = test_runner.account_balance(account3, RADIX_TOKEN).unwrap();
    let receipt = test_runner.execute_manifest(
        ManifestBuilder::new()
            .lock_fee(account3, 10u32.into())
            .call_method(component_address, "take_loan", manifest_args!(loan_amount))
            .withdraw_from_account(account3, RADIX_TOKEN, dec!(10))
            .take_from_worktop(RADIX_TOKEN, repay_amount, |builder, bucket1| {
                builder.take_all_from_worktop(promise_token_address, |builder, bucket2| {
                    builder.call_method(
                        component_address,
                        "repay_loan",
                        manifest_args!(bucket1, bucket2),
                    )
                })
            })
            .call_method(
                account3,
                "try_deposit_batch_or_abort",
                manifest_args!(ManifestExpression::EntireWorktop),
            )
            .build(),
        vec![NonFungibleGlobalId::from_public_key(&pk3)],
    );
    let commit_result = receipt.expect_commit(true);
    let new_balance = test_runner.account_balance(account3, RADIX_TOKEN).unwrap();
    assert!(test_runner
        .account_balance(account3, promise_token_address)
        .is_none());
    assert_eq!(
        old_balance - new_balance,
        commit_result.fee_summary.total_execution_cost_xrd
            + commit_result.fee_summary.total_royalty_cost_xrd
            + (repay_amount - loan_amount)
    );

    // NOTE: If this test fails, it should print out the actual fee table in the error logs.
    // Or you can run just this test with the below:
    // cargo test -p radix-engine-tests --test metering -- test_flash_loan
    assert_eq!(
        commit_result.fee_summary.execution_cost_sum,
        0
        + 3657 /* AllocateNodeId */
        + 5777 /* CreateNode */
        + 21201 /* DropLock */
        + 5565 /* DropNode */
        + 4091947 /* Invoke */
        + 7622317 /* LockSubstate */
        + 32760 /* ReadSubstate */
        + 192500 /* RunNative */
        + 287500 /* RunSystem */
        + 1188510 /* RunWasm */
        + 50000 /* TxBaseCost */
        + 2570 /* TxPayloadCost */
        + 100000 /* TxSignatureVerification */
        + 4967 /* WriteSubstate */
    );
}

#[test]
fn test_publish_large_package() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();

    // Act
    let code = wat2wasm(&format!(
        r#"
                (module
                    (data (i32.const 0) "{}")
                    (memory $0 64)
                    (export "memory" (memory $0))
                )
            "#,
        "i".repeat(DEFAULT_MAX_INVOKE_INPUT_SIZE - 1024)
    ));
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 100.into())
        .publish_package_advanced(
            code,
            PackageDefinition::default(),
            BTreeMap::new(),
            OwnerRole::None,
        )
        .build();

    let (receipt, _) = execute_with_time_logging(&mut test_runner, manifest, vec![]);

    receipt.expect_commit_success();
}

#[test]
fn should_be_able_run_large_manifest() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();

    // Act
    let (public_key, _, account) = test_runner.new_allocated_account();

    // Act
    let mut builder = ManifestBuilder::new();
    builder.lock_fee(account, 100u32.into());
    builder.withdraw_from_account(account, RADIX_TOKEN, 100u32.into());
    for _ in 0..40 {
        builder.take_from_worktop(RADIX_TOKEN, 1.into(), |builder, bid| {
            builder.return_to_worktop(bid)
        });
    }
    builder.call_method(
        account,
        "try_deposit_batch_or_abort",
        manifest_args!(ManifestExpression::EntireWorktop),
    );
    let manifest = builder.build();

    let (receipt, _) = execute_with_time_logging(
        &mut test_runner,
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    // Assert
    receipt.expect_commit(true);
}

#[test]
fn should_be_able_to_generate_5_proofs_and_then_lock_fee() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let (public_key, _, account) = test_runner.new_allocated_account();
    let resource_address = test_runner.create_fungible_resource(100.into(), 0, account);

    // Act
    let mut builder = ManifestBuilder::new();
    for _ in 0..5 {
        builder.create_proof_from_account_of_amount(account, resource_address, 1.into());
    }
    builder.lock_fee(account, 100u32.into());
    let manifest = builder.build();

    let (receipt, _) = execute_with_time_logging(
        &mut test_runner,
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    // Assert
    receipt.expect_commit(true);
}

fn setup_test_runner_with_fee_blueprint_component() -> (TestRunner, ComponentAddress) {
    // Basic setup
    let mut test_runner = TestRunner::builder().build();
    let (public_key, _, account) = test_runner.new_allocated_account();

    // Publish package and instantiate component
    let package_address = test_runner.compile_and_publish("./tests/blueprints/fee");
    let receipt1 = test_runner.execute_manifest(
        ManifestBuilder::new()
            .lock_fee(account, 10u32.into())
            .withdraw_from_account(account, RADIX_TOKEN, 10u32.into())
            .take_all_from_worktop(RADIX_TOKEN, |builder, bucket_id| {
                builder.call_function(package_address, "Fee", "new", manifest_args!(bucket_id));
                builder
            })
            .build(),
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );
    let commit_result = receipt1.expect_commit(true);
    let component_address = commit_result.new_component_addresses()[0];

    (test_runner, component_address)
}

#[test]
fn spin_loop_should_end_in_reasonable_amount_of_time() {
    let (mut test_runner, component_address) = setup_test_runner_with_fee_blueprint_component();

    let manifest = ManifestBuilder::new()
        // First, lock the fee so that the loan will be repaid
        .call_method(
            component_address,
            "lock_fee",
            manifest_args!(Decimal::from(10)),
        )
        // Now spin-loop to wait for the fee loan to burn through
        .call_method(component_address, "spin_loop", manifest_args!())
        .build();

    let (receipt, _) = execute_with_time_logging(&mut test_runner, manifest, vec![]);

    // No assertion here - this is just a sanity-test
    println!(
        "{}",
        receipt.display(&AddressBech32Encoder::for_simulator())
    );
    receipt.expect_commit_failure();
}
