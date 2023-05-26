use core::time::Duration;
use criterion::{criterion_group, criterion_main, Criterion};
use radix_engine::{transaction::TransactionReceipt, types::*};
use scrypto_unit::TestRunner;
use transaction::{
    builder::{ManifestBuilder, TransactionBuilder},
    model::{TransactionHeaderV1, TransactionPayloadEncode},
    validation::{NotarizedTransactionValidator, TransactionValidator, ValidationConfig},
};

#[allow(unused_variables)]
fn bench_radiswap(c: &mut Criterion) {
    let mut test_runner = TestRunner::builder().without_trace().build();

    // Scrypto developer
    let (pk1, _, _) = test_runner.new_allocated_account();
    // Radiswap operator
    let (pk2, _, account2) = test_runner.new_allocated_account();
    // Radiswap user
    let (pk3, sk3, account3) = test_runner.new_allocated_account();

    // Publish package
    let package_address = test_runner.publish_package(
        include_bytes!("../../assets/radiswap.wasm").to_vec(),
        manifest_decode(include_bytes!("../../assets/radiswap.schema")).unwrap(),
        btreemap!(
            "Radiswap".to_owned() => RoyaltyConfigBuilder::new()
                .add_rule("instantiate_pool", 5)
                .add_rule("add_liquidity", 1)
                .add_rule("remove_liquidity", 1)
                .add_rule("swap", 2)
                .default(0),
        ),
        btreemap!(),
        AuthorityRules::new_with_owner_authority(&NonFungibleGlobalId::from_public_key(&pk1)),
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

    // Swap 1 BTC into ETH
    let btc_to_swap = Decimal::from(1);
    let manifest = ManifestBuilder::new()
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
        .build();

    let transaction_payload = TransactionBuilder::new()
        .header(TransactionHeaderV1 {
            network_id: NetworkDefinition::simulator().id,
            start_epoch_inclusive: 0,
            end_epoch_exclusive: 100,
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

fn do_swap(
    test_runner: &mut TestRunner,
    transaction_payload: &[u8],
    nonce: u32,
) -> TransactionReceipt {
    // Validate
    let validated = NotarizedTransactionValidator::new(ValidationConfig::default(
        NetworkDefinition::simulator().id,
    ))
    .check_length_decode_and_validate_from_slice(transaction_payload)
    .unwrap();

    let mut executable = validated.get_executable();

    // Execute & commit
    executable.overwrite_transaction_hash(hash(nonce.to_le_bytes()));
    let receipt = test_runner.execute_transaction(executable);
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
