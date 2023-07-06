use core::time::Duration;
use criterion::{criterion_group, criterion_main, Criterion};
use radix_engine::vm::NativeVmV1;
use radix_engine::{
    transaction::{ExecutionConfig, FeeReserveConfig, TransactionReceipt},
    types::*,
};
#[cfg(feature = "rocksdb")]
use scrypto_unit::BasicRocksdbTestRunner;
#[cfg(not(feature = "rocksdb"))]
use scrypto_unit::{TestRunner, TestRunnerBuilder};
#[cfg(feature = "rocksdb")]
use std::path::PathBuf;
use transaction::{
    prelude::*,
    validation::{NotarizedTransactionValidator, TransactionValidator, ValidationConfig},
};

fn bench_radiswap(c: &mut Criterion) {
    #[cfg(feature = "rocksdb")]
    let mut test_runner = {
        std::fs::remove_dir_all("/tmp/radiswap").unwrap();
        BasicRocksdbTestRunner::new(PathBuf::from("/tmp/radiswap"), false)
    };
    #[cfg(not(feature = "rocksdb"))]
    let mut test_runner = TestRunnerBuilder::new().without_trace().build();

    // Scrypto developer
    let (pk1, _, _) = test_runner.new_allocated_account();
    // Radiswap operator
    let (pk2, _, account2) = test_runner.new_allocated_account();
    // Radiswap user
    let (pk3, sk3, account3) = test_runner.new_allocated_account();

    // Publish package
    let package_address = test_runner.publish_package(
        include_bytes!("../../assets/radiswap.wasm").to_vec(),
        manifest_decode(include_bytes!("../../assets/radiswap.rpd")).unwrap(),
        btreemap!(),
        OwnerRole::Updatable(rule!(require(NonFungibleGlobalId::from_public_key(&pk1)))),
    );

    #[cfg(feature = "rocksdb")]
    for i in 0..100_000 {
        if i % 100 == 0 {
            println!("{}/{}", i, 100_000);
        }
        test_runner.publish_package(
            include_bytes!("../../assets/radiswap.wasm").to_vec(),
            manifest_decode(include_bytes!("../../assets/radiswap.rpd")).unwrap(),
            btreemap!(),
            OwnerRole::Updatable(rule!(require(NonFungibleGlobalId::from_public_key(&pk1)))),
        );
    }

    // Instantiate Radiswap
    let btc = test_runner.create_fungible_resource(1_000_000.into(), 18, account2);
    let eth = test_runner.create_fungible_resource(1_000_000.into(), 18, account2);
    let component_address: ComponentAddress = test_runner
        .execute_manifest(
            ManifestBuilder::new()
                .lock_standard_test_fee(account2)
                .call_function(
                    package_address,
                    "Radiswap",
                    "new",
                    manifest_args!(OwnerRole::None, btc, eth),
                )
                .try_deposit_batch_or_abort(account2)
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
                .lock_standard_test_fee(account2)
                .withdraw_from_account(account2, btc, btc_init_amount)
                .withdraw_from_account(account2, eth, eth_init_amount)
                .take_all_from_worktop(btc, "liquidity_part_1")
                .take_all_from_worktop(eth, "liquidity_part_2")
                .with_name_lookup(|builder, lookup| {
                    let bucket1 = lookup.bucket("liquidity_part_1");
                    let bucket2 = lookup.bucket("liquidity_part_2");
                    builder.call_method(
                        component_address,
                        "add_liquidity",
                        manifest_args!(bucket1, bucket2),
                    )
                })
                .try_deposit_batch_or_abort(account2)
                .build(),
            vec![NonFungibleGlobalId::from_public_key(&pk2)],
        )
        .expect_commit(true);

    // Transfer `10,000 BTC` from `account2` to `account3`
    let btc_amount = Decimal::from(10_000);
    test_runner
        .execute_manifest(
            ManifestBuilder::new()
                .lock_standard_test_fee(account2)
                .withdraw_from_account(account2, btc, btc_amount)
                .try_deposit_batch_or_abort(account3)
                .build(),
            vec![NonFungibleGlobalId::from_public_key(&pk2)],
        )
        .expect_commit_success();

    // Swap 1 BTC into ETH
    let btc_to_swap = Decimal::from(1);
    let manifest = ManifestBuilder::new()
        .lock_standard_test_fee(account3)
        .withdraw_from_account(account3, btc, btc_to_swap)
        .take_all_from_worktop(btc, "to_trade")
        .with_name_lookup(|builder, lookup| {
            let to_trade_bucket = lookup.bucket("to_trade");
            builder.call_method(component_address, "swap", manifest_args!(to_trade_bucket))
        })
        .try_deposit_batch_or_abort(account3)
        .build();

    // Drain the faucet
    for _ in 0..100 {
        test_runner
            .execute_manifest(
                ManifestBuilder::new()
                    .lock_fee_from_faucet()
                    .get_free_xrd_from_faucet()
                    .try_deposit_batch_or_abort(account3)
                    .build(),
                vec![],
            )
            .expect_commit_success();
    }

    let transaction_payload = TransactionBuilder::new()
        .header(TransactionHeaderV1 {
            network_id: NetworkDefinition::simulator().id,
            start_epoch_inclusive: Epoch::zero(),
            end_epoch_exclusive: Epoch::of(100),
            nonce: 0,
            notary_public_key: pk3.clone().into(),
            notary_is_signatory: true,
            tip_percentage: 5,
        })
        .manifest(manifest.clone())
        .notarize(&sk3)
        .build()
        .to_payload_bytes()
        .unwrap();

    // To profile with flamegraph, run
    // ```
    // sudo cargo flamegraph --bench radiswap --features flamegraph
    // ```
    let mut nonce = 100u32;
    #[cfg(feature = "flamegraph")]
    for _ in 0..1000 {
        do_swap(&mut test_runner, &transaction_payload, nonce);
        nonce += 1;
    }
    #[cfg(not(feature = "flamegraph"))]
    c.bench_function("Radiswap::run", |b| {
        b.iter(|| {
            do_swap(&mut test_runner, &transaction_payload, nonce);
            nonce += 1;
        })
    });
}

#[cfg(feature = "rocksdb")]
type TestRunnerType = BasicRocksdbTestRunner;
#[cfg(not(feature = "rocksdb"))]
type TestRunnerType = TestRunner<NativeVmV1>;

fn do_swap(
    test_runner: &mut TestRunnerType,
    transaction_payload: &[u8],
    nonce: u32,
) -> TransactionReceipt {
    // Validate
    let validated = NotarizedTransactionValidator::new(ValidationConfig::simulator())
        .validate_from_payload_bytes(transaction_payload)
        .unwrap();

    let mut executable = validated.get_executable();

    // Execute & commit
    executable.overwrite_intent_hash(hash(nonce.to_le_bytes()));
    let receipt = test_runner.execute_transaction(
        executable,
        FeeReserveConfig::default(),
        ExecutionConfig::for_notarized_transaction(),
    );
    receipt.expect_commit_success();

    receipt
}

criterion_group!(
    name = radiswap;
    // Reduce number of iterations by reducing the benchmark duration.
    // This is to avoid VaultError(LockFeeInsufficientBalance) error
    config = Criterion::default().measurement_time(Duration::from_millis(2000));
    targets = bench_radiswap
);
criterion_main!(radiswap);
