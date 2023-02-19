use radix_engine::transaction::TransactionReceipt;
use radix_engine::types::*;
use radix_engine_interface::blueprints::resource::FromPublicKey;
use radix_engine_interface::blueprints::resource::*;
use scrypto_unit::*;
use transaction::builder::ManifestBuilder;
use transaction::model::TransactionManifest;

// For WASM-specific metering tests, see `wasm_metering.rs`.

#[cfg(feature = "std")]
fn execute_with_time_logging(
    test_runner: &mut TestRunner,
    manifest: TransactionManifest,
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
    manifest: TransactionManifest,
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
            "deposit_batch",
            args!(ManifestExpression::EntireWorktop),
        )
        .build();

    let (receipt, _) = execute_with_time_logging(
        &mut test_runner,
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key1)],
    );

    receipt.expect_commit_success();

    // Assert
    // NOTE: If this test fails, it should print out the actual fee table in the error logs.
    // Or you can run just this test with the below:
    // (cd radix-engine && cargo test --test metering -- test_basic_transfer)
    assert_eq!(
        15000 /* CreateNode */
        + 65500 /* DropLock */
        + 12500 /* DropNode */
        + 0 /* InstantiateWasm */
        + 6500 /* Invoke */
        + 101000 /* LockSubstate */
        + 76500 /* ReadSubstate */
        + 62500 /* RunPrecompiled */
        + 0 /* RunWasm */
        + 50000 /* TxBaseCost */
        + 1320 /* TxPayloadCost */
        + 100000 /* TxSignatureVerification */
        + 18500, /* WriteSubstate */
        receipt.execution.fee_summary.total_cost_units_consumed
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
        scrypto_decode(include_bytes!("../../assets/radiswap.abi")).unwrap(),
        btreemap!(
            "Radiswap".to_owned() => RoyaltyConfigBuilder::new()
                .add_rule("instantiate_pool", 5)
                .add_rule("add_liquidity", 1)
                .add_rule("remove_liquidity", 1)
                .add_rule("swap", 2)
                .default(0),
        ),
        btreemap!(),
        package_access_rules_from_owner_badge(&NonFungibleGlobalId::from_public_key(&pk1)),
    );

    // Instantiate radiswap
    let btc = test_runner.create_fungible_resource(1_000_000.into(), 18, account2);
    let eth = test_runner.create_fungible_resource(1_000_000.into(), 18, account2);
    let btc_init_amount = Decimal::from(500_000);
    let eth_init_amount = Decimal::from(300_000);
    let fee_amount = dec!("0.01");
    let (component_address, _) = test_runner
        .execute_manifest(
            ManifestBuilder::new()
                .lock_fee(account2, 10u32.into())
                .withdraw_from_account(account2, btc, btc_init_amount)
                .withdraw_from_account(account2, eth, eth_init_amount)
                .take_from_worktop(btc, |builder, bucket1| {
                    builder.take_from_worktop(eth, |builder, bucket2| {
                        builder.call_function(
                            package_address,
                            "Radiswap",
                            "instantiate_pool",
                            args!(
                                bucket1,
                                bucket2,
                                dec!("1000"),
                                "LP__ETH",
                                "LP token for /ETH swap",
                                "https://www.radiswap.com",
                                fee_amount
                            ),
                        )
                    })
                })
                .call_method(
                    account2,
                    "deposit_batch",
                    args!(ManifestExpression::EntireWorktop),
                )
                .build(),
            vec![NonFungibleGlobalId::from_public_key(&pk2)],
        )
        .output::<(ComponentAddress, Own)>(5);

    // Transfer `10,000 BTC` from `account2` to `account3`
    let btc_amount = Decimal::from(10_000);
    test_runner
        .execute_manifest(
            ManifestBuilder::new()
                .lock_fee(account2, 10u32.into())
                .withdraw_from_account(account2, btc, btc_amount)
                .call_method(
                    account3,
                    "deposit_batch",
                    args!(ManifestExpression::EntireWorktop),
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
            .take_from_worktop(btc, |builder, bucket| {
                builder.call_method(component_address, "swap", args!(bucket))
            })
            .call_method(
                account3,
                "deposit_batch",
                args!(ManifestExpression::EntireWorktop),
            )
            .build(),
        vec![NonFungibleGlobalId::from_public_key(&pk3)],
    );
    let remaining_btc = test_runner.account_balance(account3, btc).unwrap();
    let eth_received = test_runner.account_balance(account3, eth).unwrap();
    assert_eq!(remaining_btc, btc_amount - btc_to_swap);
    assert_eq!(
        eth_received,
        eth_init_amount
            - (btc_init_amount * eth_init_amount)
                / (btc_init_amount + (btc_to_swap - btc_to_swap * fee_amount))
    );

    // NOTE: If this test fails, it should print out the actual fee table in the error logs.
    // Or you can run just this test with the below:
    // (cd radix-engine && cargo test --test metering -- test_radiswap)
    assert_eq!(
        25000 /* CreateNode */
        + 189000 /* DropLock */
        + 17500 /* DropNode */
        + 19000 /* Invoke */
        + 296000 /* LockSubstate */
        + 230000 /* ReadSubstate */
        + 162500 /* RunPrecompiled */
        + 1616710 /* RunWasm */
        + 50000 /* TxBaseCost */
        + 1705 /* TxPayloadCost */
        + 100000 /* TxSignatureVerification */
        + 48000 /* WriteSubstate */
        + 2, /* royalty in cost units */
        receipt.execution.fee_summary.total_cost_units_consumed
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
        "i".repeat(4 * 1024 * 1024)
    ));
    assert_eq!(4194343, code.len());
    let manifest = ManifestBuilder::new()
        .lock_fee(FAUCET_COMPONENT, 100.into())
        .publish_package(
            code,
            BTreeMap::new(),
            BTreeMap::new(),
            BTreeMap::new(),
            AccessRules::new(),
        )
        .build();

    let (receipt, _) = execute_with_time_logging(&mut test_runner, manifest, vec![]);

    receipt.expect_commit_success();

    // Assert
    assert!(
        receipt.execution.fee_summary.total_cost_units_consumed > 20000000
            && receipt.execution.fee_summary.total_cost_units_consumed < 30000000
    );
}

#[test]
fn should_be_able_run_large_manifest() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let (public_key, _, account) = test_runner.new_allocated_account();

    // Act
    let mut builder = ManifestBuilder::new();
    builder.lock_fee(account, 100u32.into());
    builder.withdraw_from_account(account, RADIX_TOKEN, 100u32.into());
    for _ in 0..300 {
        builder.take_from_worktop_by_amount(1.into(), RADIX_TOKEN, |builder, bid| {
            builder.return_to_worktop(bid)
        });
    }
    builder.call_method(
        account,
        "deposit_batch",
        args!(ManifestExpression::EntireWorktop),
    );
    let manifest = builder.build();

    let (receipt, _) = execute_with_time_logging(
        &mut test_runner,
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    // Assert
    receipt.expect_commit_success();
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
        builder.create_proof_from_account_by_amount(account, resource_address, 1.into());
    }
    builder.lock_fee(account, 100u32.into());
    let manifest = builder.build();

    let (receipt, _) = execute_with_time_logging(
        &mut test_runner,
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    // Assert
    receipt.expect_commit_success();
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
            .take_from_worktop(RADIX_TOKEN, |builder, bucket_id| {
                builder.call_function(package_address, "Fee", "new", args!(bucket_id));
                builder
            })
            .build(),
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );
    let component_address = receipt1
        .expect_commit()
        .entity_changes
        .new_component_addresses[0];

    (test_runner, component_address)
}

#[test]
fn spin_loop_should_end_in_reasonable_amount_of_time() {
    let (mut test_runner, component_address) = setup_test_runner_with_fee_blueprint_component();

    let manifest = ManifestBuilder::new()
        // First, lock the fee so that the loan will be repaid
        .call_method(component_address, "lock_fee", args!(Decimal::from(10)))
        // Now spin-loop to wait for the fee loan to burn through
        .call_method(component_address, "spin_loop", args!())
        .build();

    let (receipt, _) = execute_with_time_logging(&mut test_runner, manifest, vec![]);

    // No assertion here - this is just a sanity-test
    println!("{:?}", receipt);
    receipt.expect_commit_failure();
}
